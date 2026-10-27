//! Residual edges use ordinary calls, real enum fields and existing drop exits.
use super::{
    Builder, Control, Error, Kind, LocalKind, NodeId, Operation, Place, Rvalue, Terminator, Type,
    ValueId, Work,
};
use crate::{
    body::{ir::ProjectionKind, scopes, storage::Range},
    sema::{
        members::{Name, Selection, VariantId},
        try_plans::{Kind as PlanKind, Plan},
    },
    types::{Primitive, TypeId},
};

impl Builder<'_> {
    fn try_plan(&self, node: NodeId) -> Result<Plan, Error> {
        let Some(Operation::Try { plan }) = self.fact(node)?.operation else {
            return Err(self.error(node, "Try expression lost its checked call plan"));
        };
        crate::sema::try_plans::checked(
            self.report,
            self.facts,
            self.source,
            crate::resolve::unit::NodeKey {
                node,
                ..self.source
            },
            plan,
        )
    }
    pub(super) fn enter_try_block(&mut self, node: NodeId, body: NodeId) -> Result<(), Error> {
        let plan = self.try_plan(node)?;
        if !matches!(plan.kind,PlanKind::Block {body:actual,..} if actual==body) {
            return Err(self.error(node, "Try block plan has a different actual body"));
        }
        let Kind::Block {
            statements, tail, ..
        } = self.kind(body)
        else {
            return Err(self.error(node, "Try block lost its actual parsed block"));
        };
        let merge = self.merge(node)?;
        let exit = self.new_block(node)?;
        self.scratch.controls.insert(
            node,
            Control {
                exit,
                repeat: None,
                merge,
                live_base: self.scratch.live.len(),
                scope: self.node_scope(node)?,
            },
        );
        self.scratch.work.push(Work::TryFinish { node, body });
        if let Some(tail) = tail {
            self.scratch.work.push(Work::Enter(tail));
        }
        self.children(statements);
        Ok(())
    }
    pub(super) fn finish_try_block(&mut self, node: NodeId, body: NodeId) -> Result<(), Error> {
        let control = *self
            .scratch
            .controls
            .get(&node)
            .ok_or_else(|| self.error(node, "Try block lost its active control target"))?;
        if self.current.is_some() {
            let PlanKind::Block { from_output, .. } = self.try_plan(node)?.kind else {
                return Err(self.error(node, "Try block lost its actual from_output call"));
            };
            let Kind::Block { tail, .. } = self.kind(body) else {
                return Err(self.error(node, "Try block body changed during construction"));
            };
            let output = if let Some(tail) = tail {
                self.value(tail)?
            } else {
                self.unit(body)?
            };
            // Preserve the real block's retained tail coercion before its call,
            // while keeping the call inside the block's storage/drop extent.
            self.save(body, output)?;
            let value = self.implicit_call(node, from_output, self.value(body)?)?;
            if let Some(merge) = control.merge {
                self.store(node, Place::Local(merge), value)?;
            }
            self.exit_scopes(node, self.boundary_range(body, scopes::Kind::Block)?)?;
            self.cleanup(node, control.live_base)?;
            self.terminate(node, Terminator::Jump(control.exit));
        }
        self.scratch.live.truncate(control.live_base);
        self.scratch.controls.remove(&node);
        self.switch(node, control.exit)?;
        if self.current.is_some() {
            let value = if let Some(merge) = control.merge {
                self.evaluate(node, self.ty(node)?, Rvalue::Read(Place::Local(merge)))?
            } else {
                self.unit(node)?
            };
            self.save(node, value)?;
        }
        Ok(())
    }
    pub(super) fn question_mark(
        &mut self,
        node: NodeId,
        operand: NodeId,
    ) -> Result<ValueId, Error> {
        let PlanKind::Branch {
            operand: actual,
            branch,
            residual,
            target,
            control_flow,
            continue_,
            break_,
        } = self.try_plan(node)?.kind
        else {
            return Err(self.error(node, "question mark lost its actual branch plan"));
        };
        if actual != operand {
            return Err(self.error(node, "question mark operand changed"));
        }
        let value = self.implicit_call(node, branch, self.value(operand)?)?;
        let base = self.scratch.live.len();
        let local = self.local(node, control_flow, LocalKind::Binding, None, false)?;
        self.store(node, Place::Local(local), value)?;
        let present = self.new_block(node)?;
        let absent = self.new_block(node)?;
        let test = self.evaluate(
            node,
            TypeId::primitive(Primitive::Bool),
            Rvalue::IsVariant {
                place: Place::Local(local),
                variant: continue_,
            },
        )?;
        self.terminate(
            node,
            Terminator::Branch {
                condition: test,
                yes: present,
                no: absent,
            },
        );
        self.switch(node, absent)?;
        let Some(Type::Function { parameters, .. }) = self.facts.types.kind(residual.signature)
        else {
            return Err(self.error(node, "Try residual call lost its actual signature"));
        };
        let Some([input]) = self.facts.types.list(parameters) else {
            return Err(self.error(node, "Try residual call changed its input arity"));
        };
        let value = self.flow_field(node, Place::Local(local), break_, *input)?;
        let value = self.implicit_call(node, residual, value)?;
        self.residual_exit(node, target, value)?;
        self.switch(node, present)?;
        let value = self.flow_field(node, Place::Local(local), continue_, self.ty(node)?)?;
        self.cleanup(node, base)?;
        self.scratch.live.truncate(base);
        Ok(value)
    }
    fn flow_field(
        &mut self,
        node: NodeId,
        base: Place,
        variant: VariantId,
        ty: TypeId,
    ) -> Result<ValueId, Error> {
        let field = self
            .facts
            .members
            .field(variant, Name::Index(0))
            .ok_or_else(|| self.error(node, "Try control-flow variant lost its real field"))?;
        let place = self.project(
            node,
            ty,
            base,
            ProjectionKind::Field(Selection::Nominal(field)),
        )?;
        self.evaluate(node, ty, Rvalue::Read(place))
    }
    fn residual_exit(
        &mut self,
        node: NodeId,
        target: Option<NodeId>,
        value: ValueId,
    ) -> Result<(), Error> {
        let Some(target) = target else {
            return self.return_value(node, value, self.node_scope(node)?);
        };
        let control = *self.scratch.controls.get(&target).ok_or_else(|| {
            self.error(
                node,
                "Try residual target is outside its actual lexical scope",
            )
        })?;
        if let Some(merge) = control.merge {
            self.store(node, Place::Local(merge), value)?;
        }
        self.exit_scopes(
            node,
            Range {
                inner: self.node_scope(node)?,
                outer: Some(control.scope),
            },
        )?;
        self.cleanup(node, control.live_base)?;
        self.terminate(node, Terminator::Jump(control.exit));
        Ok(())
    }
}
