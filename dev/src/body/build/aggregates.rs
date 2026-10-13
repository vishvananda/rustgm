//! Flat place construction preserves projections and aggregate evaluation order.
use super::{
    Action, Builder, DefKind, Error, Kind, List, NodeId, Operation, Place, Rvalue, Target, Type,
    TypeId, ValueId, ValueList, Work,
};
use crate::{
    body::ir::{Projection, ProjectionId, ProjectionKind},
    sema::members::{FieldId, Selection},
};

pub(super) fn constructors(
    report: &crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
    program: &mut super::Program,
) -> Result<(), Error> {
    use super::{Local, LocalId, LocalKind, Owner, Statement, Value};
    for (index, ty) in facts.definitions.iter().enumerate() {
        let Some(Type::FunctionItem {
            item, signature, ..
        }) = ty.and_then(|ty| facts.types.kind(ty))
        else {
            continue;
        };
        let definition = crate::resolve::model::DefId(item.0);
        let Some(variant_id) = facts.members.constructors.get(&definition).copied() else {
            continue;
        };
        let variant = facts.members.variants[variant_id.0 as usize];
        if item.0 as usize != index {
            continue;
        }
        let offset = report
            .package
            .unit(variant.node.file)
            .configured
            .ast
            .node(variant.node.node)
            .span
            .start;
        let error = |message| Error::at(variant.node.file, offset, message);
        let Some(Type::Function { result, .. }) = facts.types.kind(signature) else {
            return Err(error("missing constructor signature"));
        };
        let mut body = super::Body::new(
            Owner::Function {
                definition,
                signature,
            },
            variant.node,
            result,
        );
        body.storage = Some(crate::body::storage::Facts::new(
            crate::body::scopes::Facts::synthetic(variant.node.node),
        ));
        for (ordinal, field) in facts
            .members
            .fields(variant_id)
            .ok_or_else(|| error("missing constructor fields"))?
            .iter()
            .enumerate()
        {
            let ordinal =
                u32::try_from(ordinal).map_err(|_| error("too many constructor fields"))?;
            let local = LocalId(ordinal);
            body.locals.push(Local {
                scope: crate::body::scopes::Id(0),
                storage_parent: ordinal.checked_sub(1).map(LocalId),
                ty: field.ty,
                kind: LocalKind::Parameter(ordinal),
                definition: None,
                mutable: false,
                source: field.node.node,
            });
            body.statements.push(Statement {
                source: field.node.node,
                action: Action::StorageLive(local),
            });
            let value = ValueId(ordinal);
            body.values.push(Value {
                ty: field.ty,
                kind: Rvalue::Read(Place::Local(local)),
            });
            body.arguments.push(value);
            body.statements.push(Statement {
                source: field.node.node,
                action: Action::Evaluate(value),
            });
        }
        finish_constructor(&mut body, variant_id).map_err(error)?;
        program.bodies.push(body);
    }
    Ok(())
}

fn finish_constructor(
    body: &mut super::Body,
    variant_id: crate::sema::members::VariantId,
) -> Result<(), &'static str> {
    use super::{Block, LocalId, Statement, Terminator, Value};
    let count =
        u32::try_from(body.arguments.len()).map_err(|_| "too many constructor arguments")?;
    let value = ValueId(count);
    body.values.push(Value {
        ty: body.result,
        kind: Rvalue::Aggregate {
            variant: Some(variant_id),
            fields: ValueList {
                start: 0,
                end: count,
            },
        },
    });
    body.statements.push(Statement {
        source: body.source.node,
        action: Action::Evaluate(value),
    });
    let before =
        u32::try_from(body.statements.len()).map_err(|_| "too many constructor statements")?;
    body.storage
        .as_mut()
        .ok_or("missing constructor storage")?
        .exits
        .push(crate::body::storage::Exit {
            location: crate::body::storage::Location::At {
                block: super::BlockId(0),
                before,
            },
            end: crate::body::storage::End::Scopes(crate::body::storage::Range {
                inner: crate::body::scopes::Id(0),
                outer: None,
            }),
        });
    if let Some(last) = count.checked_sub(1).map(LocalId) {
        body.statements.push(Statement {
            source: body.source.node,
            action: Action::end_storage(LocalId(0), last),
        });
    }
    let end =
        u32::try_from(body.statements.len()).map_err(|_| "too many constructor statements")?;
    body.blocks.push(Block {
        start: 0,
        end,
        source: body.source.node,
        terminator: Terminator::Return(value),
    });
    Ok(())
}

impl Builder<'_> {
    pub(super) fn projected_syntax(&self, mut node: NodeId) -> bool {
        while let Kind::Group(value) | Kind::Attributed { value, .. } = self.kind(node) {
            node = value;
        }
        matches!(
            self.kind(node),
            Kind::Field { .. }
                | Kind::Index { .. }
                | Kind::Unary {
                    op: crate::syntax::ast::Unary::Dereference,
                    ..
                }
        )
    }
    pub(super) fn pop_place(&mut self, node: NodeId) -> Result<Option<Place>, Error> {
        self.scratch
            .places
            .pop()
            .ok_or_else(|| self.error(node, "place construction stack underflow"))
    }
    pub(super) fn discard_unreachable_place(&mut self, node: NodeId) -> Result<(), Error> {
        if matches!(self.kind(node), Kind::Struct { base: Some(_), .. })
            || matches!(
                self.fact(node)?.operation,
                Some(Operation::AssignPlace { .. } | Operation::Borrow)
            )
        {
            self.pop_place(node)?;
        }
        Ok(())
    }
    pub(super) fn enter_place(&mut self, node: NodeId) -> Result<(), Error> {
        if self.current.is_none() {
            self.scratch.places.push(None);
            return Ok(());
        }
        match self.kind(node) {
            Kind::Group(value) | Kind::Attributed { value, .. } => {
                self.scratch.work.push(Work::Place(value));
            }
            Kind::Expansion {
                children,
                tail: Some(value),
            } if children.start == children.end => {
                self.scratch.work.push(Work::Place(value));
            }
            Kind::Unary {
                op: crate::syntax::ast::Unary::Dereference,
                operand: value,
            }
            | Kind::Field { value, .. } => {
                self.scratch.work.push(Work::FinishPlace(node));
                self.scratch.work.push(Work::Place(value));
            }
            Kind::Index { value, index } => {
                self.scratch.work.push(Work::FinishPlace(node));
                self.scratch.work.push(Work::Enter(index));
                self.scratch.work.push(Work::Place(value));
            }
            Kind::Path { .. } | Kind::Name(_) => {
                let Some(Target::Definition(definition)) = self.fact(node)?.reference else {
                    return Err(self.error(node, "missing place resolution"));
                };
                let place = match self.report.resolution.definitions[definition.0 as usize].kind {
                    DefKind::Local | DefKind::Parameter => self.local_place(node, definition)?,
                    DefKind::Static => Place::Static(definition),
                    _ => {
                        self.scratch.work.push(Work::TemporaryPlace(node));
                        self.enter_materialized(node);
                        return Ok(());
                    }
                };
                self.scratch.places.push(Some(place));
            }
            _ => {
                self.scratch.work.push(Work::TemporaryPlace(node));
                self.enter_materialized(node);
            }
        }
        Ok(())
    }
    fn enter_materialized(&mut self, node: NodeId) {
        let work = if self.scratch.unadjusted.last() == Some(&node) {
            Work::EnterUnadjusted(node)
        } else {
            Work::Enter(node)
        };
        self.scratch.work.push(work);
    }
    pub(super) fn project(
        &mut self,
        node: NodeId,
        ty: TypeId,
        base: Place,
        kind: ProjectionKind,
    ) -> Result<Place, Error> {
        let id = self.projection(node, ty, base, kind)?;
        self.statement(node, Action::Project(id))?;
        Ok(Place::Projection(id))
    }
    pub(super) fn projection(
        &mut self,
        node: NodeId,
        ty: TypeId,
        base: Place,
        kind: ProjectionKind,
    ) -> Result<ProjectionId, Error> {
        let id = ProjectionId(self.index(node, self.body.projections.len())?);
        self.body.projections.push(Projection { base, kind, ty });
        Ok(id)
    }
    pub(super) fn finish_place(&mut self, node: NodeId) -> Result<(), Error> {
        let base = self.pop_place(node)?;
        let place = if let Some(base) = base
            && self.current.is_some()
        {
            let base = self.automatic_dereferences(node, base)?;
            if self.fact(node)?.operation == Some(Operation::Dereference) {
                let place = self.dereference_place(node, base)?;
                self.scratch.places.push(Some(place));
                return Ok(());
            }
            let kind = match self.fact(node)?.operation {
                Some(Operation::Field { field }) => ProjectionKind::Field(field),
                Some(Operation::Index { length }) => {
                    let Kind::Index { index, .. } = self.kind(node) else {
                        return Err(self.error(node, "invalid index selection"));
                    };
                    ProjectionKind::Index {
                        index: self.value(index)?,
                        length,
                    }
                }
                _ => return Err(self.error(node, "missing selected projection")),
            };
            Some(self.project(node, self.ty(node)?, base, kind)?)
        } else {
            None
        };
        self.scratch.places.push(place);
        Ok(())
    }
    pub(super) fn read_place(&mut self, node: NodeId) -> Result<(), Error> {
        if let Some(place) = self.pop_place(node)?
            && self.current.is_some()
        {
            let ty = self.ty(node)?;
            let value = self.evaluate(node, ty, Rvalue::Read(place))?;
            self.save(node, value)?;
            if ty == TypeId::NEVER {
                self.terminate(node, super::Terminator::Unreachable);
            }
        }
        Ok(())
    }
    pub(super) fn aggregate_elements(
        &mut self,
        node: NodeId,
        elements: List,
    ) -> Result<ValueId, Error> {
        let start = self.index(node, self.body.arguments.len())?;
        let unit = self.report.package.unit(self.body.source.file);
        for element in unit.configured.ast.list(elements) {
            if self.active(*element) {
                self.body.arguments.push(self.value(*element)?);
            }
        }
        let end = self.index(node, self.body.arguments.len())?;
        self.evaluate(
            node,
            self.ty(node)?,
            Rvalue::Aggregate {
                variant: None,
                fields: ValueList { start, end },
            },
        )
    }
    pub(super) fn repeat(&mut self, node: NodeId, value: NodeId) -> Result<ValueId, Error> {
        let ty = self.ty(node)?;
        let Some(Type::Array { .. }) = self.facts.types.kind(ty) else {
            return Err(self.error(node, "missing repeat array type"));
        };
        self.evaluate(
            node,
            ty,
            Rvalue::Repeat {
                value: self.value(value)?,
            },
        )
    }
    pub(super) fn construct(
        &mut self,
        node: NodeId,
        fields: List,
        base: Option<NodeId>,
    ) -> Result<ValueId, Error> {
        let Some(Operation::Construct { variant: id }) = self.fact(node)?.operation else {
            return Err(self.error(node, "missing aggregate constructor selection"));
        };
        let variant = self.facts.members.variants[id.0 as usize];
        self.scratch.field_values.clear();
        self.scratch
            .field_values
            .resize((variant.end - variant.start) as usize, None);
        let unit = self.report.package.unit(self.body.source.file);
        for mut field in unit.configured.ast.list(fields).iter().copied() {
            if !self.active(field) {
                continue;
            }
            while let Kind::Attributed { value, .. } = self.kind(field) {
                field = value;
            }
            let Some(Operation::FieldInit { field: selected }) = self.fact(field)?.operation else {
                return Err(self.error(field, "missing aggregate field selection"));
            };
            self.scratch.field_values[(selected.0 - variant.start) as usize] =
                Some(self.value(field)?);
        }
        if base.is_some() {
            let base = self
                .pop_place(node)?
                .ok_or_else(|| self.error(node, "missing functional update place"))?;
            for index in variant.start..variant.end {
                let ordinal = (index - variant.start) as usize;
                if self.scratch.field_values[ordinal].is_some() {
                    continue;
                }
                let ty = self.facts.members.fields[index as usize].ty;
                let place = self.project(
                    node,
                    ty,
                    base,
                    ProjectionKind::Field(Selection::Nominal(FieldId(index))),
                )?;
                self.scratch.field_values[ordinal] =
                    Some(self.evaluate(node, ty, Rvalue::Read(place))?);
            }
        }
        let start = self.index(node, self.body.arguments.len())?;
        for value in &self.scratch.field_values {
            self.body
                .arguments
                .push(value.ok_or_else(|| self.error(node, "missing checked field initializer"))?);
        }
        let end = self.index(node, self.body.arguments.len())?;
        self.evaluate(
            node,
            self.ty(node)?,
            Rvalue::Aggregate {
                variant: Some(id),
                fields: ValueList { start, end },
            },
        )
    }
}
