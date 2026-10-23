//! For loops consume canonical call plans and use ordinary storage and transfer cleanup.
use super::{
    BlockId, Builder, Control, Error, LocalKind, NodeId, Place, Rvalue, Terminator, Type, TypeId,
    ValueId, ValueList, Work,
};
use crate::{
    body::{
        ir::{BorrowKind, ProjectionKind},
        scopes,
    },
    sema::{iteration::Call, members::Selection},
    types::Primitive,
};
impl Builder<'_> {
    fn for_plan(&self, node: NodeId) -> Result<crate::sema::iteration::ForPlan, Error> {
        let plan = self
            .facts
            .iteration
            .as_deref()
            .and_then(|r| {
                r.plans.get(&crate::resolve::unit::NodeKey {
                    file: self.source.file,
                    node,
                })
            })
            .copied()
            .ok_or_else(|| self.error(node, "missing checked iteration plan"))?;
        let canonical = self
            .facts
            .iteration
            .as_deref()
            .ok_or_else(|| self.error(node, "missing canonical iteration registry"))?
            .canonical;
        for (call, kind) in [
            (
                plan.into_iter,
                crate::support::iteration::Trait::IntoIterator,
            ),
            (plan.next, crate::support::iteration::Trait::Iterator),
        ] {
            if !matches!(self.facts.types.kind(call.callable), Some(Type::FunctionItem {item, ..}) if item.0 == canonical.get(kind).method.0)
            {
                return Err(self.error(node, "iteration plan has a different canonical method"));
            }
        }
        Ok(plan)
    }
    pub(super) fn initialize_for(&mut self, node: NodeId, base: usize) -> Result<(), Error> {
        if self.current.is_none() {
            return Ok(());
        }
        let crate::syntax::ast::Kind::For { iterator, body, .. } = self.kind(node) else {
            return Err(self.error(node, "invalid iteration CFG continuation"));
        };
        let plan = self.for_plan(node)?;
        let source = self.value(iterator)?;
        let iterator = self.iteration_call(node, plan.into_iter, source)?;
        let local = self.local(node, plan.iterator, LocalKind::Binding, None, true)?;
        let extent = self.boundary(node, scopes::Kind::Block)?;
        self.body.locals[local.0 as usize].scope = extent;
        self.store(node, Place::Local(local), iterator)?;
        let iteration_base = self.scratch.live.len();
        let header = self.new_block(node)?;
        let exit = self.new_block(node)?;
        let present = self.new_block(node)?;
        let absent = self.new_block(node)?;
        self.scratch.controls.insert(
            node,
            Control {
                exit,
                repeat: Some(header),
                merge: None,
                live_base: iteration_base,
                scope: extent,
            },
        );
        self.terminate(node, Terminator::Jump(header));
        self.switch(node, header)?;
        self.enter_for_item(node, plan, local, (exit, iteration_base), (present, absent))?;
        self.scratch.work.push(Work::ForFinish { node, exit, base });
        self.scratch.work.push(Work::ForRepeat {
            node,
            header,
            base: iteration_base,
        });
        self.scratch.work.push(Work::Enter(body));
        Ok(())
    }
    fn enter_for_item(
        &mut self,
        node: NodeId,
        plan: crate::sema::iteration::ForPlan,
        local: crate::body::ir::LocalId,
        exit: (BlockId, usize),
        branches: (BlockId, BlockId),
    ) -> Result<(), Error> {
        let crate::syntax::ast::Kind::For { pattern, .. } = self.kind(node) else {
            return Err(self.error(node, "missing iteration pattern source"));
        };
        let (exit, iteration_base) = exit;
        let (present, absent) = branches;
        let Some(Type::Function { parameters, .. }) = self.facts.types.kind(plan.next.signature)
        else {
            return Err(self.error(node, "iteration next is not callable"));
        };
        let [receiver] = self
            .facts
            .types
            .list(parameters)
            .ok_or_else(|| self.error(node, "missing iteration next receiver"))?
        else {
            return Err(self.error(node, "incorrect iteration next arity"));
        };
        let borrow = self.evaluate(
            node,
            *receiver,
            Rvalue::Borrow {
                place: Place::Local(local),
                kind: BorrowKind::Immediate,
            },
        )?;
        let next = self.iteration_call(node, plan.next, borrow)?;
        let option = self.local(node, plan.option, LocalKind::Binding, None, false)?;
        self.body.locals[option.0 as usize].scope = self.boundary(node, scopes::Kind::Iteration)?;
        self.store(node, Place::Local(option), next)?;
        let test = self.evaluate(
            node,
            TypeId::primitive(Primitive::Bool),
            Rvalue::IsVariant {
                place: Place::Local(option),
                variant: plan.some,
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
        if self.facts.members.variants[plan.none.0 as usize].owner
            != self.facts.members.variants[plan.some.0 as usize].owner
        {
            return Err(self.error(node, "iteration Option variants have different owners"));
        }
        self.repeat_or_exit_for(node, exit, iteration_base)?;
        self.switch(node, present)?;
        let [_] = self
            .facts
            .members
            .fields(plan.some)
            .ok_or_else(|| self.error(node, "missing canonical iteration Some field"))?
        else {
            return Err(self.error(node, "canonical iteration Some has incorrect fields"));
        };
        let field = self
            .facts
            .members
            .field(plan.some, crate::sema::members::Name::Index(0))
            .ok_or_else(|| self.error(node, "missing canonical iteration Some selection"))?;
        let place = self.project(
            node,
            plan.item,
            Place::Local(option),
            ProjectionKind::Field(Selection::Nominal(field)),
        )?;
        let pattern = self
            .pattern_root(pattern)
            .ok_or_else(|| self.error(node, "missing checked iteration pattern"))?;
        self.bind_pattern(pattern, Some(place))?;
        Ok(())
    }
    fn iteration_call(
        &mut self,
        node: NodeId,
        call: Call,
        argument: ValueId,
    ) -> Result<ValueId, Error> {
        let Some(Type::FunctionItem { item, .. }) = self.facts.types.kind(call.callable) else {
            return Err(self.error(node, "iteration call has no canonical method identity"));
        };
        let Some(Type::Function { result, .. }) = self.facts.types.kind(call.signature) else {
            return Err(self.error(node, "iteration call has no checked signature"));
        };
        let callee = self.evaluate(
            node,
            call.callable,
            Rvalue::Function(crate::resolve::model::DefId(item.0)),
        )?;
        let start = self.index(node, self.body.arguments.len())?;
        self.body.arguments.push(argument);
        let end = self.index(node, self.body.arguments.len())?;
        self.evaluate(
            node,
            result,
            Rvalue::Call {
                signature: call.signature,
                callee,
                arguments: ValueList { start, end },
            },
        )
    }
    fn repeat_or_exit_for(
        &mut self,
        node: NodeId,
        target: BlockId,
        base: usize,
    ) -> Result<(), Error> {
        self.exit_scopes(node, self.boundary_range(node, scopes::Kind::Iteration)?)?;
        self.cleanup(node, base)?;
        self.terminate(node, Terminator::Jump(target));
        Ok(())
    }
    pub(super) fn repeat_for(
        &mut self,
        node: NodeId,
        header: BlockId,
        base: usize,
    ) -> Result<(), Error> {
        self.repeat_or_exit_for(node, header, base)?;
        self.scratch.live.truncate(base);
        Ok(())
    }
    pub(super) fn finish_for(
        &mut self,
        node: NodeId,
        exit: BlockId,
        base: usize,
    ) -> Result<(), Error> {
        self.scratch.controls.remove(&node);
        self.switch(node, exit)?;
        self.exit_scopes(node, self.boundary_range(node, scopes::Kind::Block)?)?;
        self.cleanup(node, base)?;
        self.scratch.live.truncate(base);
        if self.current.is_some() {
            let unit = self.unit(node)?;
            self.save(node, unit)?;
        }
        Ok(())
    }
}
