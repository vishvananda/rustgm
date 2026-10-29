//! Defining Conditional cleanup legality is separate from executable const permission.
mod claims;
mod preparation;
#[cfg(test)]
#[path = "../../../tests/invariants/destruct_contracts.rs"]
mod tests;
use crate::{
    body::{
        ir::{Body, BodyKey, BodyOrigin, Owner},
        regions::Contract,
    },
    resolve::model::DefId,
    sema::{function_context::FunctionContext, model::Declarations},
    types::{TypeId, parameters::environment::Environment},
};
pub(crate) use preparation::{
    begin, mark_imported_required, mark_required, prepare, validate_imported_required,
};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Definition {
    function: DefId,
    context: FunctionContext,
    source: BodyOrigin,
    signature: TypeId,
}

/// Required is not a completed negative, empty proof or executable permission.
pub(crate) enum State {
    Required(Definition),
    Prepared(Prepared),
}
pub(crate) const fn required_allocation() -> usize {
    size_of::<State>() + 2 * size_of::<usize>()
}
pub(crate) struct Prepared {
    definition: Definition,
    owned_signature: TypeId,
    const_context: bool,
    environment: Arc<Environment>,
    role: claims::Role,
    copy_role: claims::Role,
    claims: claims::Claims,
}
/// Indexed declaration data; a projected demand still needs the later proof owner.
pub(crate) enum Requirement {
    Declared,
    ProjectedDestruct,
}
impl State {
    pub(crate) fn check_environment(
        &self,
        expected: &Arc<Environment>,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        let Self::Prepared(prepared) = self else {
            return Err("const-trait defining cleanup requires completed body preparation");
        };
        if !Arc::ptr_eq(expected, &prepared.environment) {
            return Err("const-trait defining cleanup has another actual template environment");
        }
        Ok(())
    }
    pub(crate) fn retained_bytes(&self) -> usize {
        size_of::<Self>().saturating_add(match self {
            Self::Required(_) => 0,
            Self::Prepared(prepared) => prepared.claims.retained_bytes(),
        })
    }
    pub(crate) fn checked<'a>(
        &'a self,
        body: &Body,
        environment: Option<&Arc<Environment>>,
        work: &mut Work,
    ) -> Result<&'a Prepared, &'static str> {
        work.charge(1)?;
        let Self::Prepared(prepared) = self else {
            return Err("const-trait defining cleanup requires completed body preparation");
        };
        work.charge(5)?;
        if body.key() != BodyKey::Declaration(prepared.definition.function)
            || body.source != prepared.definition.source
            || body.instance.is_some()
            || body.const_context != prepared.const_context
            || body.owner.signatures()
                != Some((prepared.definition.signature, prepared.owned_signature))
            || environment.is_none_or(|e| !Arc::ptr_eq(e, &prepared.environment))
        {
            return Err("const-trait defining cleanup belongs to another body or environment");
        }
        Ok(prepared)
    }
}
impl Prepared {
    pub(crate) fn checks_cleanup(&self) -> bool {
        self.const_context
            || matches!(
                self.definition.context,
                FunctionContext::ConstTraitDefault { .. }
            )
    }
    #[cfg(test)]
    pub(crate) fn check(
        &self,
        types: &crate::types::Store,
        ty: TypeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        match self.requirement(types, ty, work)? {
            Requirement::Declared => Ok(()),
            Requirement::ProjectedDestruct => {
                Err("projected defining cleanup requires completed region formation")
            }
        }
    }
    pub(crate) fn requirement(
        &self,
        types: &crate::types::Store,
        ty: TypeId,
        work: &mut Work,
    ) -> Result<Requirement, &'static str> {
        work.charge(1)?;
        match self.claims.get(ty) {
            Some(claims::Claim::NoDrop) => return Ok(Requirement::Declared),
            Some(claims::Claim::ConditionalDestruct) => {
                self.role.require()?;
                return Ok(Requirement::Declared);
            }
            Some(claims::Claim::ProjectedDestruct) => {
                self.role.require()?;
                return Ok(Requirement::ProjectedDestruct);
            }
            Some(claims::Claim::PendingCopy) => {
                self.copy_role.require()?;
                return Err("defining cleanup Copy premise changed its readiness");
            }
            None => {}
        }
        work.charge(1)?;
        self.role.require()?;
        work.charge(1)?;
        let kind = types.kind(ty).ok_or("invalid defining cleanup subject")?;
        Err(
            if self.claims.unsupported
                || !matches!(
                    kind,
                    crate::types::Kind::Parameter(_) | crate::types::Kind::Projection(_)
                )
            {
                "quantified or composite const-trait defining Destruct obligations are not implemented"
            } else {
                "const-trait defining cleanup requires a retained conditional Destruct obligation; implied proofs are not implemented"
            },
        )
    }
}

pub(crate) struct Work {
    pub(crate) steps: usize,
    limit: usize,
}
impl Work {
    pub(crate) const fn new(limit: usize) -> Self {
        Self { steps: 0, limit }
    }
    pub(crate) fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.steps = self.steps.saturating_add(count);
        if self.steps > self.limit {
            return Err("defining cleanup work budget exceeded");
        }
        Ok(())
    }
}

pub(crate) fn definition(
    facts: &Declarations,
    definition: DefId,
    source: BodyOrigin,
    steps: &mut usize,
) -> Result<Option<Definition>, &'static str> {
    let default = facts
        .associated
        .trait_owner(definition)
        .is_some_and(|owner| facts.associated.constant_trait(owner) == Some(true));
    if !default && !generic_const_function(facts, definition) {
        return Ok(None);
    }
    let header = facts
        .function_header(definition)?
        .ok_or("defining cleanup function header is pending")?;
    if header.body.is_none() {
        return Ok(None);
    }
    let context = if let Some(source_key) = source.source() {
        facts.function_context(definition, source_key)?
    } else {
        let view = crate::sema::crates::source_original(facts, definition, steps)?
            .ok_or("imported defining cleanup requires its actual complete source original")?;
        if view.source()? != source || !view.const_context() || default {
            return Err("imported defining cleanup changed its free const original context");
        }
        FunctionContext::ConstFunction
    };
    if !context.checks_const() {
        return Err("defining cleanup requires its actual provided member owner");
    }
    if header.body != Some(source) {
        return Err("defining cleanup changed its actual function source");
    }
    Ok(Some(Definition {
        function: definition,
        context,
        source,
        signature: header.signature,
    }))
}

pub(crate) fn needs_preparation(facts: &Declarations, body: &Body) -> bool {
    if body.instance.is_some() {
        return false;
    }
    let Owner::Function { definition: id, .. } = body.owner else {
        return false;
    };
    generic_const_function(facts, id)
        || facts
            .associated
            .trait_owner(id)
            .is_some_and(|owner| facts.associated.constant_trait(owner) == Some(true))
}

/// Nongeneric const bodies have no parameter cleanup promises to retain.
/// Const trait defaults keep their separate actual Self owner above.
fn generic_const_function(facts: &Declarations, id: DefId) -> bool {
    facts.source_bindings.is_some()
        && facts.is_const_function(id)
        && facts
            .lifetimes
            .callables
            .registration(crate::types::ItemId(id.0))
            .map_or_else(
                || {
                    facts.generics.parameters_in_scope(id).any(|parameter| {
                        !matches!(
                            parameter.argument,
                            crate::types::arguments::Argument::Lifetime(_)
                        )
                    })
                },
                |(_, formals)| !formals.is_empty(),
            )
}

fn validate_contract(definition: Definition, contract: &Contract) -> Result<(), &'static str> {
    if contract.key != BodyKey::Declaration(definition.function)
        || contract.source != definition.source
    {
        return Err("defining cleanup contract changed its actual owner or source");
    }
    Ok(())
}
