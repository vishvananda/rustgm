//! Keep owned value scopes while the builder still has the selected syntax scope map.
use super::{Builder, Error, NodeId, ValueId};
use crate::body::{
    drops::values::Release,
    ir::{Action, BlockId, Body, Temporary, Terminator},
    scopes::{Id, Lifetime},
    storage::Location,
};

pub(super) fn finish(
    report: &crate::resolve::expansion::Report,
    facts: &crate::sema::model::Declarations,
    program: &mut crate::body::ir::Program,
    query: Option<&mut crate::types::destruction::NeedsDrop>,
) -> Result<(), Error> {
    program.drops =
        crate::body::drops::collect(facts, program, query).map_err(|(source, message)| {
            report.diagnostic(Error::at(
                source.file,
                report
                    .package
                    .unit(source.file)
                    .configured
                    .ast
                    .node(source.node)
                    .span
                    .start,
                message,
            ))
        })?;
    Ok(())
}

impl Builder<'_> {
    pub(super) fn create_owned(&mut self, node: NodeId, value: ValueId) -> Result<(), Error> {
        let Some(query) = self.drop_query.as_deref_mut() else {
            return Ok(());
        };
        let ty = self.body.values[value.0 as usize].ty;
        if !query
            .demand(&self.facts.types, &self.facts.aggregates, ty)
            .map_err(|message| self.error(node, message))?
        {
            return Ok(());
        }
        let block = self
            .current
            .ok_or_else(|| self.error(node, "owned value has no block"))?;
        let temporary = Temporary {
            value,
            source: node,
            lifetime: self.owned_lifetime(node)?,
            block,
            before: self.index(node, self.body.statements.len())?,
            exits_before: self.index(node, self.scope_exits.len())?,
        };
        self.body
            .owned
            .get_or_insert_with(Box::default)
            .insert(temporary)
            .map_err(|message| self.error(node, message))
    }
    fn owned_lifetime(&self, node: NodeId) -> Result<Lifetime, Error> {
        self.scope_plan
            .node(node)
            .map(|info| info.temporary)
            .or_else(|| {
                self.facts
                    .node(super::NodeKey {
                        node,
                        ..self.body.source
                    })
                    .and_then(|facts| facts.binding)
                    .and_then(|binding| self.scope_plan.binding(binding))
                    .map(Lifetime::Scope)
            })
            .ok_or_else(|| self.error(node, "missing owned value scope"))
    }
    pub(super) fn forward_owned(&mut self, node: NodeId, value: ValueId) {
        if let Some(owned) = &mut self.body.owned
            && let Some(scope) = self.scope_plan.node(node)
        {
            owned.forward(value, scope.temporary);
        }
    }
    pub(super) fn release_return(&mut self, node: NodeId, value: ValueId) -> Result<(), Error> {
        if self
            .body
            .owned
            .as_ref()
            .and_then(|o| o.creation(value))
            .is_none()
        {
            return Ok(());
        }
        let location = Location::At {
            block: self
                .current
                .ok_or_else(|| self.error(node, "return has no block"))?,
            before: self.index(node, self.body.statements.len())?,
        };
        let exits_before = self.index(node, self.scope_exits.len())?;
        if let Some(owned) = &mut self.body.owned {
            owned.returned.push(Release {
                value,
                location,
                exits_before,
            });
        }
        Ok(())
    }
}

pub(super) fn constructor(
    facts: &crate::sema::model::Declarations,
    body: &mut Body,
    query: &mut crate::types::destruction::NeedsDrop,
) -> Result<(), &'static str> {
    for (index, statement) in body.statements.iter().enumerate() {
        let Action::Evaluate(value) = statement.action else {
            continue;
        };
        if query.demand(
            &facts.types,
            &facts.aggregates,
            body.values[value.0 as usize].ty,
        )? {
            body.owned
                .get_or_insert_with(Box::default)
                .insert(Temporary {
                    value,
                    source: statement.source,
                    lifetime: Lifetime::Scope(Id(0)),
                    block: BlockId(0),
                    before: u32::try_from(index + 1)
                        .map_err(|_| "too many constructor statements")?,
                    exits_before: 0,
                })?;
        }
    }
    let Terminator::Return(value) = body.blocks[0].terminator else {
        return Err("missing constructor return");
    };
    if let Some(owned) = &mut body.owned
        && owned.creation(value).is_some()
    {
        let location = body
            .storage
            .as_ref()
            .and_then(|s| s.exits.first())
            .ok_or("missing constructor cleanup")?
            .location;
        owned.returned.push(Release {
            value,
            location,
            exits_before: 0,
        });
    }
    Ok(())
}
