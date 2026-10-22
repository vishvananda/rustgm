//! Projection queries retain implementation authority and their actual proof scopes.
#[cfg(test)]
mod continuation_tests;
mod cursor;
pub(in crate::sema) mod families;
mod selected;
use super::{
    model::Declarations,
    traits::{implementations::Availability, selection::Source},
};
use crate::{
    resolve::model::DefId,
    types::{
        ItemId, TypeId,
        lifetime_proof::{Obligations, ScopeId},
        parameters::environment::Environment,
        substitute::Request,
    },
};
pub(super) use cursor::{Completed, Cursor, Progress};
pub(super) use selected::Definition;
use std::sync::Arc;

pub(super) fn definition(
    facts: &Declarations,
    owner: DefId,
    item: ItemId,
) -> Result<DefId, &'static str> {
    if let Some(iteration) = facts.iteration.as_deref()
        && let Some(definition) = iteration.associated_definition(owner, item)?
    {
        return Ok(definition);
    }
    facts
        .associated
        .type_definition(owner, DefId(item.0))
        .ok_or("selected implementation is missing the associated type")
}

/// Keep private result lifetimes in the selected proof's allocation interval.
pub(super) fn implementation_result(
    facts: &mut Declarations,
    application: crate::types::projection::Application,
    source: Source,
) -> Result<TypeId, &'static str> {
    Definition::select(facts, application.item, application.arguments, source)?
        .result(facts)?
        .ok_or("missing completed associated type definition")
}

pub(super) fn normalize(
    facts: &mut Declarations,
    environment: Option<&Arc<Environment>>,
    proof: &mut Obligations,
    ambient: ScopeId,
    ty: TypeId,
    availability: Availability,
) -> Result<(TypeId, usize), &'static str> {
    normalize_reusing(facts, environment, proof, ambient, ty, availability, None)
}

pub(super) fn normalize_reusing(
    facts: &mut Declarations,
    environment: Option<&Arc<Environment>>,
    proof: &mut Obligations,
    ambient: ScopeId,
    ty: TypeId,
    availability: Availability,
    mut completed: Option<&mut Completed>,
) -> Result<(TypeId, usize), &'static str> {
    let mut session = Session::new(
        environment.cloned(),
        proof,
        ambient,
        availability,
        |_, _| Ok(None),
    )?;
    if let Some(completed) = completed.as_deref_mut() {
        session.cursor.exchange_completed(completed);
    }
    let result = session.normalize(facts, ty);
    if let Some(completed) = completed {
        session.cursor.exchange_completed(completed);
    }
    result
}

/// Completed roots share work only within one fixed environment, proof scope,
/// implementation availability and local definition provider. Evidence stays in
/// the borrowed proof for the whole session, including nested callable scopes.
pub(crate) struct Session<'proof, F> {
    cursor: Cursor,
    proof: &'proof mut Obligations,
    local: F,
}
impl<'proof, F> Session<'proof, F>
where
    F: FnMut(&mut Declarations, Request) -> Result<Option<TypeId>, &'static str>,
{
    /// The provider must prove a result for the complete application in the
    /// request's binder context; it cannot change authority between roots.
    pub(super) fn new(
        environment: Option<Arc<Environment>>,
        proof: &'proof mut Obligations,
        ambient: ScopeId,
        availability: Availability,
        local: F,
    ) -> Result<Self, &'static str> {
        Ok(Self {
            cursor: Cursor::new(environment, proof, ambient, availability)?,
            proof,
            local,
        })
    }
    pub(crate) fn normalize(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
    ) -> Result<(TypeId, usize), &'static str> {
        self.cursor.start(facts, ty)?;
        match self.cursor.advance(facts, self.proof, &mut self.local)? {
            Progress::Complete { ty, steps } => Ok((ty, steps)),
            Progress::Definition(_) => Err("missing completed associated type definition"),
            Progress::Dependency(_) => {
                Err("projection selection requires a prepared implementation header")
            }
        }
    }

    /// Retain the first demanding root's origin for newly emitted selection debts.
    /// Cached normal forms keep their original evidence and source; no prefix is copied.
    pub(crate) fn normalize_source(
        &mut self,
        facts: &mut Declarations,
        ty: TypeId,
        source: u32,
    ) -> Result<(TypeId, usize), &'static str> {
        if source == 0 {
            return self.normalize(facts, ty);
        }
        let checkpoint = self.proof.source_checkpoint();
        let result = self.normalize(facts, ty)?;
        self.proof.label_sources_since(checkpoint, source)?;
        Ok(result)
    }
}

/// Instantiated types have no symbolic caller trait permissions. Selection uses
/// validated implementations and retains all lifetime debts in the instance.
pub(crate) type Concrete<'proof> =
    Session<'proof, fn(&mut Declarations, Request) -> Result<Option<TypeId>, &'static str>>;

pub(crate) fn concrete(
    proof: &mut Obligations,
    scope: ScopeId,
) -> Result<Concrete<'_>, &'static str> {
    Session::new(None, proof, scope, Availability::Validated, |_, _| Ok(None))
}
