//! Original initializer operands retain only their own rigid declaration parameters.
use super::{Body, Contract, Declarations};
use crate::support::hash::HashSet;
use crate::{
    body::ir::{BodyKey, Owner},
    sema::{model::Initializer, values::Instance},
    types::{
        RegionId,
        arguments::{Argument, ConstKind},
        bound_uses,
        parameters::{ParameterId, uses::Uses},
    },
};

pub(super) struct Parameters {
    allowed: HashSet<ParameterId>,
    owned: HashSet<RegionId>,
    uses: Uses,
    regions: bound_uses::Uses,
    universals: u32,
}

pub(crate) fn initializer(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    initializer: &Initializer,
) -> Result<(), String> {
    let definition = initializer.definition;
    if body.instance.is_some()
        || body.key() != BodyKey::Declaration(definition)
        || body.key() != contract.key
        || body.source != initializer.semantic_source()
        || body.source != contract.source
        || body.result != initializer.ty
        || !matches!(body.owner, Owner::Initializer { definition: owner, .. } if owner == definition)
        || !initializer.type_checked()
        || !facts
            .initializer_index(definition)
            .and_then(|index| facts.initializers.get(index))
            .is_some_and(|actual| {
                actual.semantic_source() == body.source && actual.ty == body.result
            })
    {
        return Err("original initializer dispatch belongs to another declaration or body".into());
    }
    resolve_parameters(facts, body, contract, definition)
}

pub(crate) fn anonymous(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
) -> Result<(), String> {
    let inline = facts
        .inline_constants
        .as_deref()
        .and_then(|registry| registry.get(body.source))
        .ok_or("original anonymous dispatch has no typed body")?;
    let definition = inline
        .declaration
        .ok_or("original anonymous dispatch has no declaration")?;
    if body.instance.is_some()
        || !matches!(body.owner, Owner::AnonymousConstant)
        || body.key() != BodyKey::Anonymous(inline.source)
        || body.key() != contract.key
        || body.source != contract.source
        || inline.result != Some(body.result)
        || inline.universals != contract.universals
        || facts.constant_body_type(body.source) != Some(body.result)
    {
        return Err("original anonymous dispatch belongs to another declaration or body".into());
    }
    resolve_parameters(facts, body, contract, definition)
}

fn resolve_parameters(
    facts: &mut Declarations,
    body: &mut Body,
    contract: &mut Contract,
    definition: crate::resolve::model::DefId,
) -> Result<(), String> {
    let mut visits = 0;
    let dependent = body.values.iter().any(|value| {
        visits += 1;
        matches!(value.kind, crate::body::ir::Rvalue::AssociatedConstant(_))
    });
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(visits);
    if !dependent {
        return super::resolve(facts, body, contract);
    }
    let mut allowed = HashSet::default();
    for parameter in facts.generics.parameters_in_scope(definition) {
        facts.body_work.canonical_steps += 1;
        if matches!(parameter.argument, Argument::Lifetime(_)) {
            continue;
        }
        if allowed.len() >= 1_000_000 {
            return Err("original initializer parameter budget exceeded".into());
        }
        allowed.insert(crate::types::parameters::argument_id(
            &facts.types,
            parameter.argument,
        )?);
    }
    if allowed.is_empty() {
        return super::resolve(facts, body, contract);
    }
    let owned = owned_regions(facts, contract)?;
    let parameters = Some(Parameters {
        allowed,
        owned,
        uses: Uses::default(),
        regions: bound_uses::Uses::including_free(),
        universals: contract.universals,
    });
    super::resolve_with_original(facts, body, contract, parameters)
}

impl Parameters {
    pub(super) fn deferred(
        &mut self,
        facts: &mut Declarations,
        instance: Instance,
    ) -> Result<bool, String> {
        if !Self::parameterized(facts, instance)? {
            return Ok(false);
        }
        let before = self.uses.steps.saturating_add(self.regions.steps);
        let mut direct = 0;
        let result = (|| {
            for argument in facts
                .types
                .arguments(instance.arguments)
                .ok_or("invalid original initializer operand arguments")?
            {
                direct += 1;
                if direct > 1_000_000 {
                    return Err("original initializer operand budget exceeded");
                }
                match argument {
                    Argument::Lifetime(region) => self.region(region)?,
                    Argument::Type(ty) => {
                        self.uses.collect(&facts.types, ty)?;
                        self.regions.collect(&facts.types, ty)?;
                    }
                    Argument::Const(id) => {
                        if let ConstKind::Parameter(parameter) = facts
                            .types
                            .const_kind(id)
                            .ok_or("invalid original initializer operand constant")?
                        {
                            self.uses.constants.insert(parameter);
                        }
                    }
                }
            }
            Ok(true)
        })();
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(
            self.uses
                .steps
                .saturating_add(self.regions.steps)
                .saturating_sub(before)
                .saturating_add(direct),
        );
        result.map_err(String::from)
    }

    pub(super) fn validate(&self, facts: &mut Declarations) -> Result<(), String> {
        let mut work = 0_usize;
        let result = (|| {
            let mut visit = || {
                work += 1;
                if work > 1_000_000 {
                    return Err("original initializer validation work budget exceeded");
                }
                Ok(())
            };
            for parameter in self.uses.parameters.iter().chain(&self.uses.constants) {
                visit()?;
                if !self.allowed.contains(parameter) {
                    return Err("original initializer operand has a foreign parameter");
                }
            }
            for region in self.regions.free_regions() {
                visit()?;
                if self.region(region).is_err() {
                    return Err("original initializer operand has a foreign lifetime");
                }
            }
            for &index in &self.regions.indices {
                visit()?;
                if u32::from(index) >= self.universals {
                    return Err("original initializer operand has a foreign lifetime");
                }
            }
            Ok(())
        })();
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
        result.map_err(String::from)
    }

    fn parameterized(facts: &mut Declarations, instance: Instance) -> Result<bool, &'static str> {
        let mut dependent = false;
        for argument in facts
            .types
            .arguments(instance.arguments)
            .ok_or("invalid original initializer operand arguments")?
        {
            facts.body_work.canonical_steps += 1;
            dependent |= match argument {
                Argument::Type(ty) => facts
                    .types
                    .contains_parameters(ty)
                    .ok_or("invalid original initializer operand type")?,
                Argument::Const(id) => matches!(
                    facts
                        .types
                        .const_kind(id)
                        .ok_or("invalid original initializer operand constant")?,
                    ConstKind::Parameter(_)
                ),
                Argument::Lifetime(_) => false,
            };
        }
        Ok(dependent)
    }

    fn region(&self, region: RegionId) -> Result<(), &'static str> {
        if region == RegionId::STATIC || self.owned.contains(&region) {
            return Ok(());
        }
        if region
            .bound_parts()
            .is_some_and(|(depth, slot)| depth == 0 && u32::from(slot) < self.universals)
        {
            Ok(())
        } else {
            Err("original initializer operand has a foreign lifetime")
        }
    }
}

fn owned_regions(
    facts: &mut Declarations,
    contract: &Contract,
) -> Result<HashSet<RegionId>, &'static str> {
    let mut owned = HashSet::default();
    let mut work = 0_usize;
    let mut add = |id: RegionId| -> Result<(), &'static str> {
        work = work.saturating_add(1);
        if work > 1_000_000 {
            return Err("original initializer identity work budget exceeded");
        }
        if id == RegionId::STATIC
            || id.bound_parts().is_some()
            || !facts
                .body_regions
                .get(contract.source, id)
                .is_some_and(|record| record.body == contract.source)
        {
            return Err("original initializer identity belongs to another body");
        }
        if crate::support::retained::new_map::<RegionId, ()>(owned.len() + 1) > 32 * 1024 * 1024 {
            return Err("original initializer identity storage budget exceeded");
        }
        owned.insert(id);
        Ok(())
    };
    let result = (|| {
        for obligation in &contract.obligations {
            if let crate::body::regions::Kind::Variable { id, .. } = obligation.kind {
                add(id)?;
            }
        }
        for proof in &contract.proofs {
            for range in proof.proof.owned_regions() {
                for id in range {
                    add(RegionId(id))?;
                }
            }
        }
        Ok(())
    })();
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work);
    result?;
    Ok(owned)
}

#[cfg(test)]
#[path = "../../../tests/invariants/rigid_operands.rs"]
mod tests;
