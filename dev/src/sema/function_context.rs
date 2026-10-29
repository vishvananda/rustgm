//! Defining const checking is distinct from executable function eligibility.
use super::{associated::OwnerKind, model::Declarations};
use crate::resolve::{model::DefId, unit::NodeKey};
use crate::types::parameters::{
    clauses::Clause,
    predicates::{Bound, Predicate},
};
mod abstract_uses;
pub(in crate::sema) mod assumption_environment;
pub(crate) use abstract_uses::{Call, DefiningCalls};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum FunctionContext {
    Runtime,
    ConstFunction,
    Comptime,
    ConstTraitDefault { trait_definition: DefId },
}

#[cfg(test)]
#[path = "../../tests/invariants/provided_contexts.rs"]
mod tests;
impl FunctionContext {
    pub(crate) const fn checks_const(self) -> bool {
        !matches!(self, Self::Runtime)
    }
}
impl Declarations {
    /// Only the checked source panic roles have a runtime-checked defining body.
    /// Their const declaration and every reached const-call obligation remain.
    pub(crate) fn source_function_context(
        &self,
        report: &crate::resolve::expansion::Report,
        definition: DefId,
        body: NodeKey,
    ) -> Result<FunctionContext, &'static str> {
        let context = self.function_context(definition, body)?;
        if matches!(
            context,
            FunctionContext::Comptime | FunctionContext::ConstFunction
        ) {
            match (context, self.source_comptime_contract(report, definition)) {
                (
                    FunctionContext::Comptime,
                    Ok(super::header_attributes::ContractKnowledge::KnownPresent(_)),
                )
                | (
                    FunctionContext::ConstFunction,
                    Ok(super::header_attributes::ContractKnowledge::KnownAbsent),
                ) => {}
                _ => return Err("const defining body lost its actual source declaration contract"),
            }
        }
        Ok(
            if super::compiler_items::runtime_panic_body(report, self, definition, body)? {
                FunctionContext::Runtime
            } else {
                context
            },
        )
    }

    /// Abstract/default member identity cannot become executable via Code reuse.
    pub(crate) fn check_const_member_execution(
        &self,
        definition: DefId,
    ) -> Result<(), &'static str> {
        let Some(trait_definition) = self.associated.trait_owner(definition) else {
            return Ok(());
        };
        if self.associated.constant_trait(trait_definition) != Some(true) {
            return Ok(());
        }
        let function = self
            .function_indices
            .get(&definition)
            .and_then(|&index| self.functions.get(index))
            .ok_or(
                "source const-trait member execution requires its completed actual declaration",
            )?;
        if function.definition != definition {
            return Err("source const-trait member execution changed its actual declaration");
        }
        Err(if function.body.is_some() {
            "source const-trait default execution is not implemented"
        } else {
            "source abstract const-trait member execution is not implemented"
        })
    }

    pub(in crate::sema) fn default_self_clause(
        &self,
        definition: DefId,
        body: NodeKey,
    ) -> Result<Option<Clause>, &'static str> {
        let FunctionContext::ConstTraitDefault { trait_definition } =
            self.function_context(definition, body)?
        else {
            return Ok(None);
        };
        Ok(Some(self.defining_self_clause(trait_definition)?))
    }

    /// Construct the actual const trait's defining premise, without body authority.
    pub(in crate::sema) fn defining_self_clause(
        &self,
        trait_definition: DefId,
    ) -> Result<Clause, &'static str> {
        let mut predicate = self.const_self_predicate(trait_definition)?;
        let Bound::Trait(application) = predicate.bound else {
            return Err("const trait lost its ordinary implicit Self requirement");
        };
        predicate.bound = Bound::MaybeConstTrait(application);
        Ok(predicate.into())
    }

    pub(in crate::sema) fn member_use_clause(
        &self,
        definition: DefId,
        input: super::declaration_inputs::Input,
        effect: crate::support::constness::Constness,
    ) -> Result<Clause, &'static str> {
        use crate::support::constness::Constness;
        let clause = input.value;
        if effect == Constness::Ordinary {
            return Ok(clause);
        }
        let Some(trait_definition) = self.associated.trait_owner(definition) else {
            return Ok(clause);
        };
        if self.associated.constant_trait(trait_definition) != Some(true) {
            return Ok(clause);
        }
        let owner = self
            .associated
            .owner(trait_definition)
            .and_then(|id| self.associated.get(id))
            .ok_or("const member requirement has no actual trait owner")?;
        let source = owner.source.ok_or("imported const-member defining requests require their authenticated source-profile counterpart")?;
        if input.source.source() != Some(source) {
            return Ok(clause);
        }
        let mut implicit = self.const_self_predicate(trait_definition)?;
        if clause != Clause::Predicate(implicit) {
            return Ok(clause);
        }
        let Bound::Trait(application) = implicit.bound else {
            return Err("const trait lost its ordinary implicit Self requirement");
        };
        implicit.bound = Bound::trait_application(application, effect);
        Ok(implicit.into())
    }

    fn const_self_predicate(&self, definition: DefId) -> Result<Predicate, &'static str> {
        if self.associated.constant_trait(definition) != Some(true) {
            return Err("defining Self premise has no actual const trait");
        }
        let predicate = *self
            .generics
            .trait_self_bounds
            .get(&definition)
            .ok_or("const trait requires its completed implicit Self application")?;
        let schema = self
            .associated
            .schema(definition)
            .ok_or("const trait requires its completed Self schema")?;
        let Bound::Trait(id) = predicate.bound else {
            return Err("const trait changed its canonical implicit Self effect");
        };
        if predicate.ty != schema.self_type
            || self
                .types
                .trait_application(id)
                .is_none_or(|a| a.item.0 != definition.0)
        {
            return Err("const trait changed its canonical Self application or schema");
        }
        Ok(predicate)
    }

    /// Inspect only the actual completed function and its indexed direct owner.
    /// This result supplies defining legality, never permission to execute a call.
    pub(crate) fn function_context(
        &self,
        definition: DefId,
        body: NodeKey,
    ) -> Result<FunctionContext, &'static str> {
        let function = self
            .function_indices
            .get(&definition)
            .and_then(|&index| self.functions.get(index))
            .ok_or("defining context requires a completed actual function")?;
        if function.definition != definition || function.body != Some(body) {
            return Err("defining context belongs to another function or body");
        }
        if let Some(id) = self.associated.owner(definition) {
            let owner = self
                .associated
                .get(id)
                .ok_or("defining context has an invalid associated owner")?;
            if let OwnerKind::Trait {
                definition: trait_definition,
                constant: true,
                ..
            } = owner.kind
            {
                if self.associated.owner(trait_definition) != Some(id) {
                    return Err("defining context changed its actual trait owner");
                }
                return Ok(FunctionContext::ConstTraitDefault { trait_definition });
            }
        }
        Ok(if self.is_comptime_function(definition) {
            FunctionContext::Comptime
        } else if self.is_const_function(definition) {
            FunctionContext::ConstFunction
        } else {
            FunctionContext::Runtime
        })
    }
}
