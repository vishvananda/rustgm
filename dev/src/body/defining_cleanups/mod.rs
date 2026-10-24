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
pub(crate) use preparation::{begin, mark_required, prepare};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Definition {
    function: DefId,
    trait_owner: DefId,
    source: BodyOrigin,
    signature: TypeId,
}

/// Required is not a completed negative, empty proof or executable permission.
pub(crate) enum State {
    Required(Definition),
    Prepared(Prepared),
}
pub(crate) struct Prepared {
    definition: Definition,
    owned_signature: TypeId,
    environment: Arc<Environment>,
    role: claims::Role,
    copy_role: claims::Role,
    claims: claims::Claims,
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
    pub(crate) fn check(
        &self,
        types: &crate::types::Store,
        ty: TypeId,
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        match self.claims.get(ty) {
            Some(claims::Claim::NoDrop) => return Ok(()),
            Some(claims::Claim::ConditionalDestruct) => {
                self.role.require()?;
                return Ok(());
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
            if self.claims.unsupported || !matches!(kind, crate::types::Kind::Parameter(_)) {
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
) -> Result<Option<Definition>, &'static str> {
    let Some(trait_definition) = facts.associated.trait_owner(definition) else {
        return Ok(None);
    };
    if facts.associated.constant_trait(trait_definition) != Some(true) {
        return Ok(None);
    }
    let source_key = source
        .source()
        .ok_or("imported const-trait defining cleanup headers are not implemented")?;
    if !matches!(facts.function_context(definition, source_key)?,
        FunctionContext::ConstTraitDefault { trait_definition: actual } if actual == trait_definition)
    {
        return Err("defining cleanup requires its actual provided member owner");
    }
    let header = facts
        .function_header(definition)?
        .ok_or("defining cleanup function header is pending")?;
    if header.body != Some(source) {
        return Err("defining cleanup changed its actual function source");
    }
    Ok(Some(Definition {
        function: definition,
        trait_owner: trait_definition,
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
    facts
        .associated
        .trait_owner(id)
        .is_some_and(|owner| facts.associated.constant_trait(owner) == Some(true))
}

fn validate_contract(definition: Definition, contract: &Contract) -> Result<(), &'static str> {
    if contract.key != BodyKey::Declaration(definition.function)
        || contract.source != definition.source
    {
        return Err("defining cleanup contract changed its actual owner or source");
    }
    Ok(())
}
