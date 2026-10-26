//! Canonical alias clauses share actual declaration slots without a trait owner.
mod default_arguments;
mod dependency_cycles;
mod expansion;
mod headers;
mod mapping;
mod unused_contracts;
use super::{Checker, DefId, Error, NodeKey, regions};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefKind, Namespace},
    },
    sema::{associated::Schema, generics::Predicate},
    support::{hash::HashMap, retained::Growth},
    syntax::{ast::Kind, item::ItemKind},
    types::{GenericArgs, TypeId},
};
use std::cell::Cell;

#[derive(Clone, Copy)]
struct Owner {
    definition: DefId,
    self_definition: DefId,
    source: NodeKey,
}
#[derive(Clone, Copy)]
pub(super) struct Edge {
    source: NodeKey,
    target: DefId,
    subject: TypeId,
    original_subject: TypeId,
    arguments: GenericArgs,
    quantified: Option<u32>,
}
struct Header {
    owner: Owner,
    started: bool,
    schema: Option<Schema>,
    completed: bool,
    edges: Vec<Edge>,
}
pub(super) struct Pending {
    headers: Vec<Header>,
    index: HashMap<DefId, usize>,
    budget: Budget,
    closed: bool,
}
#[derive(Default)]
struct Budget {
    steps: Cell<usize>,
    bytes: Cell<usize>,
}
impl Budget {
    fn charge(&self, steps: usize, bytes: usize) -> Result<(), &'static str> {
        self.steps.set(self.steps.get().saturating_add(steps));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.steps.get() > 10_000_000 || self.bytes.get() > 64 * 1024 * 1024 {
            return Err("trait alias work or retention budget exceeded");
        }
        Ok(())
    }
    fn grow(&self, growth: Growth) -> Result<(), &'static str> {
        self.charge(growth.work.saturating_add(1), growth.bytes)
    }
}

pub(super) fn collect(report: &Report) -> Result<Option<Box<Pending>>, Error> {
    let mut pending: Option<Box<Pending>> = None;
    for (index, definition) in report.resolution.definitions.iter().enumerate() {
        if definition.kind != DefKind::SelfType {
            continue;
        }
        let Some(source) = definition.node else {
            continue;
        };
        let ast = &report.package.unit(source.file).configured.ast;
        let Kind::Item(item) = ast.node(source.node).kind else {
            continue;
        };
        if !matches!(ast.items[item as usize].kind, ItemKind::TraitAlias { .. }) {
            continue;
        }
        let error = |message| crate::sema::model::Declarations::error(report, source, message);
        let module = report.resolution.scopes[definition.scope.0 as usize].module;
        let actual = *report
            .resolution
            .item_definitions
            .get(&(module, source.node, Namespace::Type))
            .ok_or_else(|| error("trait alias has no actual declaration identity"))?;
        if report.resolution.definitions[actual.0 as usize].kind != DefKind::TraitAlias {
            return Err(error(
                "trait alias Self belongs to a different declaration kind",
            ));
        }
        let owner = Owner {
            definition: actual,
            self_definition: DefId(
                u32::try_from(index).map_err(|_| error("too many alias definitions"))?,
            ),
            source,
        };
        if pending.is_none() {
            pending = Some(Pending::new().map_err(error)?);
        }
        let table = pending
            .as_mut()
            .ok_or_else(|| error("missing allocated alias table"))?;
        table.insert(owner).map_err(error)?;
    }
    Ok(pending)
}
impl Pending {
    fn new() -> Result<Box<Self>, &'static str> {
        let budget = Budget::default();
        budget.charge(1, size_of::<Self>())?;
        Ok(Box::new(Self {
            headers: Vec::new(),
            index: HashMap::default(),
            budget,
            closed: false,
        }))
    }
    fn insert(&mut self, owner: Owner) -> Result<(), &'static str> {
        self.budget.grow(Growth::vector::<Header>(
            self.headers.len(),
            self.headers.capacity(),
            1,
        ))?;
        self.budget.grow(Growth::map::<DefId, usize>(
            self.index.len(),
            self.index.capacity(),
            1,
        ))?;
        if self
            .index
            .insert(owner.definition, self.headers.len())
            .is_some()
        {
            return Err("trait alias has multiple lexical Self declarations");
        }
        self.headers.push(Header {
            owner,
            started: false,
            schema: None,
            completed: false,
            edges: Vec::new(),
        });
        Ok(())
    }
    fn header(&self, definition: DefId) -> Result<&Header, &'static str> {
        self.budget.charge(1, 0)?;
        self.index
            .get(&definition)
            .and_then(|index| self.headers.get(*index))
            .ok_or("trait alias has no local canonical owner")
    }
}
impl Checker<'_> {
    pub(super) fn alias_schema(&self, owner: DefId) -> Option<&Schema> {
        let pending = self.trait_aliases.as_ref()?;
        let index = *pending.index.get(&owner)?;
        pending.headers[index].schema.as_ref()
    }

    pub(super) fn charge_alias_growth(&self, growth: Growth) -> Result<(), Error> {
        self.trait_aliases
            .as_ref()
            .ok_or_else(|| Error::unlocated("missing alias lookup owner"))?
            .budget
            .grow(growth)
            .map_err(Error::unlocated)
    }

    pub(super) fn alias_self_schema(&self, id: DefId) -> Result<Option<&Schema>, Error> {
        let Some(pending) = &self.trait_aliases else {
            return Ok(None);
        };
        pending.budget.charge(4, 0).map_err(Error::unlocated)?;
        let definition = &self.report.resolution.definitions[id.0 as usize];
        if definition.kind != DefKind::SelfType {
            return Ok(None);
        }
        let Some(source) = definition.node else {
            return Ok(None);
        };
        let module = self.report.resolution.scopes[definition.scope.0 as usize].module;
        let Some(owner) =
            self.report
                .resolution
                .item_definitions
                .get(&(module, source.node, Namespace::Type))
        else {
            return Ok(None);
        };
        let Some(&index) = pending.index.get(owner) else {
            return Ok(None);
        };
        let header = &pending.headers[index];
        if header.owner.self_definition != id {
            return Err(self.error(source, "alias lexical Self has a different canonical owner"));
        }
        let schema = header
            .schema
            .as_ref()
            .ok_or_else(|| self.error(source, "alias Self schema is not prepared"))?;
        if self.facts.definitions[id.0 as usize] != Some(schema.self_type) {
            return Err(self.error(source, "alias lexical Self is not complete"));
        }
        Ok(Some(schema))
    }
    pub(super) fn push_alias_edge(&self, edges: &mut Vec<Edge>, edge: Edge) -> Result<(), Error> {
        let pending = self
            .trait_aliases
            .as_ref()
            .ok_or_else(|| self.error(edge.source, "missing canonical alias owner"))?;
        pending
            .budget
            .grow(Growth::vector::<Edge>(edges.len(), edges.capacity(), 1))
            .map_err(|e| self.error(edge.source, e))?;
        edges.push(edge);
        Ok(())
    }
    pub(super) fn alias_self(&self, definition: DefId) -> Option<DefId> {
        let pending = self.trait_aliases.as_ref()?;
        let index = pending.index.get(&definition)?;
        Some(pending.headers[*index].owner.self_definition)
    }
    pub(super) fn alias_scope_count(
        &self,
        scope: Option<super::generics::bounds::lexical::Scope>,
    ) -> Option<u32> {
        scope.map(|scope| scope.count(&self.regions))
    }
    pub(super) fn retain_alias_edges(
        &mut self,
        owner: DefId,
        edges: Vec<Edge>,
    ) -> Result<(), Error> {
        if edges.is_empty() {
            return Ok(());
        }
        let pending = self
            .trait_aliases
            .as_mut()
            .ok_or_else(|| Error::unlocated("missing alias declaration owner"))?;
        let index = *pending
            .index
            .get(&owner)
            .ok_or_else(|| Error::unlocated("alias edges require an alias owner"))?;
        pending.headers[index].edges = edges;
        Ok(())
    }
    pub(super) fn canonical_alias_edge(
        &mut self,
        path: NodeKey,
        target: DefId,
        subject: TypeId,
        original_subject: TypeId,
        context: regions::Context,
        quantified: Option<u32>,
    ) -> Result<Edge, Error> {
        let arguments = self.application_arguments(path, target, context)?;
        Ok(Edge {
            source: path,
            target,
            subject,
            original_subject,
            arguments,
            quantified,
        })
    }
    pub(super) fn reject_alias_execution(&self) -> Result<(), Error> {
        if let Some(pending) = &self.trait_aliases {
            let source = pending
                .headers
                .first()
                .ok_or_else(|| Error::unlocated("empty allocated alias inventory"))?
                .owner
                .source;
            return Err(self.error(
                source,
                "trait alias contract expansion is not implemented for all consumers and export",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/transparent_contracts.rs"]
mod tests;
