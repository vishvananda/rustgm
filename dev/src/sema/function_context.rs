//! Defining const checking is distinct from executable function eligibility.
use super::{associated::OwnerKind, model::Declarations};
use crate::resolve::{body_origin::ProofSite, model::DefId, unit::NodeKey};
use crate::support::bindings::LangItem;
use crate::types::{
    TypeId,
    parameters::{
        clauses::Clause,
        predicates::{Bound, Predicate},
    },
    traits::ApplicationId,
};
mod abstract_uses;
pub(in crate::sema) mod assumption_environment;
mod entry_contracts;
pub(crate) use abstract_uses::{Call, DefiningCalls};
pub(crate) use entry_contracts::{DefaultExecution, DefaultMaterialization};

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
    /// A source member inherits the well-formed inputs of its completed impl.
    pub(crate) fn source_impl_inputs(
        &self,
        member: DefId,
    ) -> Option<(DefId, TypeId, Option<ApplicationId>)> {
        let owner = self.associated.get(self.associated.owner(member)?)?;
        let OwnerKind::Impl { trait_type, .. } = owner.kind else {
            return None;
        };
        let definition = owner.self_definition;
        if trait_type.is_none() {
            let schema = self.associated.schema(member)?;
            return Some((definition, schema.self_type, None));
        }
        let registry = self.generics.implementations.as_deref()?;
        if !registry.is_validated(definition) {
            return None;
        }
        let predicate = registry.header(definition)?.predicate;
        let (application, _) = predicate.bound.trait_parts()?;
        Some((definition, predicate.ty, Some(application)))
    }

    /// Authenticated source policy can defer defining const checks.
    /// The const declaration and every reached executable obligation remain.
    pub(crate) fn source_function_context(
        &self,
        report: &crate::resolve::expansion::Report,
        definition: DefId,
        body: NodeKey,
    ) -> Result<FunctionContext, &'static str> {
        let context = self.function_context(definition, body)?;
        if self.associated.trait_owner(definition).is_some() {
            match self.source_non_const_trait_method_contract(report, definition) {
                Ok(
                    super::header_attributes::ContractKnowledge::KnownPresent(_)
                    | super::header_attributes::ContractKnowledge::KnownAbsent,
                ) => {}
                _ => {
                    return Err("trait defining body lost its completed source constness contract");
                }
            }
        }
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
            if super::compiler_items::runtime_panic_body(report, self, definition, body)?
                || (matches!(context, FunctionContext::ConstFunction)
                    && self.unchecked_const_body(report, definition, body)?)
            {
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
        if self.nonconst_trait_member(definition)? {
            return Err("non-const trait methods cannot be called in const contexts");
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

    pub(crate) fn imported_default_premise(
        &self,
        definition: DefId,
    ) -> Result<Option<(DefId, Clause)>, &'static str> {
        self.imported_conditional_member_trait(definition)?
            .map(|owner| Ok((owner, self.defining_self_clause(owner)?)))
            .transpose()
    }

    /// Construct the actual const trait's defining premise, without body authority.
    pub(crate) fn defining_self_clause(
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
        if self.nonconst_trait_member(definition)? {
            return Err("non-const trait methods cannot be called in const contexts");
        }
        let owner = self
            .associated
            .owner(trait_definition)
            .and_then(|id| self.associated.get(id))
            .ok_or("const member requirement has no actual trait owner")?;
        let site = if let Some(source) = owner.source {
            source.into()
        } else {
            self.imported_member_self_site(definition, trait_definition)?
        };
        if input.source != site {
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

    pub(in crate::sema) fn imported_const_operator_role(
        &self,
        definition: DefId,
    ) -> Result<Option<super::compiler_items::Role>, &'static str> {
        use crate::support::crates::generics::source::Profile;
        let Some(catalog) = self.source_bindings.as_deref() else {
            return Ok(None);
        };
        for kind in Profile::CONST_BINARY_ROLES
            .into_iter()
            .chain(Profile::CONST_DEREF_ROLES)
            .chain(LangItem::COMPARISON_TRAITS)
        {
            if let Some(role) = catalog.role(kind)?
                && role.definition == definition
                && role.origin.source().is_none()
            {
                return Ok(Some(role));
            }
        }
        Ok(None)
    }

    /// Imported conditional methods come from witnessed language roles.
    pub(crate) fn imported_conditional_member_trait(
        &self,
        definition: DefId,
    ) -> Result<Option<DefId>, &'static str> {
        let Some(owner) = self.associated.trait_owner(definition) else {
            return Ok(None);
        };
        let Some(header) = self
            .declaration_header(definition)?
            .filter(|header| header.declaration.source().is_none())
        else {
            return Ok(None);
        };
        if header.body.is_none() && self.imported_const_operator_role(owner)?.is_some() {
            return Ok(Some(owner));
        }
        let Some(catalog) = self.source_bindings.as_deref() else {
            return Ok(None);
        };
        if header.body.is_none()
            && self.associated.constant_trait(owner) == Some(true)
            && catalog
                .matching_role(
                    definition,
                    LangItem::REQUIRED_FOR_METHODS
                        .into_iter()
                        .chain([LangItem::CloneFn])
                        .map(|role| ((), role)),
                )?
                .is_some()
        {
            return Ok(Some(owner));
        }
        for kind in [LangItem::Drop, LangItem::Clone, LangItem::Iterator]
            .into_iter()
            .chain(LangItem::COMPARISON_TRAITS)
        {
            if let Some(role) = catalog.role(kind)?
                && role.definition == owner
                && role.origin.source().is_none()
                && (!matches!(kind, LangItem::Clone | LangItem::Iterator) || header.body.is_some())
                && self.associated.constant_trait(owner) == Some(true)
                && (kind != LangItem::Iterator
                    || catalog.imported_member_named(
                        header.declaration,
                        &LangItem::PROVIDED_ITERATOR_METHODS,
                    )?)
            {
                return Ok(Some(owner));
            }
        }
        Ok(None)
    }

    /// The loaded Clone role seals the provided default before any body demand.
    pub(crate) fn imported_clone_default(&self, definition: DefId) -> Result<bool, &'static str> {
        self.imported_default_for(definition, LangItem::Clone)
    }

    pub(crate) fn imported_const_default(&self, definition: DefId) -> Result<bool, &'static str> {
        Ok(self.imported_clone_default(definition)?
            || self.imported_default_for(definition, LangItem::Iterator)?)
    }

    fn imported_default_for(
        &self,
        definition: DefId,
        kind: LangItem,
    ) -> Result<bool, &'static str> {
        let Some(catalog) = self.source_bindings.as_deref() else {
            return Ok(false);
        };
        let Some(role) = catalog.role(kind)? else {
            return Ok(false);
        };
        if role.origin.source().is_some()
            || self.associated.trait_owner(definition) != Some(role.definition)
            || self.associated.constant_trait(role.definition) != Some(true)
        {
            return Ok(false);
        }
        let Some(header) = self
            .declaration_header(definition)?
            .filter(|header| header.declaration.source().is_none() && header.body.is_some())
        else {
            return Ok(false);
        };
        if kind == LangItem::Iterator
            && !catalog
                .imported_member_named(header.declaration, &LangItem::PROVIDED_ITERATOR_METHODS)?
        {
            return Ok(false);
        }
        Ok(catalog.validated_role(kind)? == Some(role))
    }

    fn imported_member_self_site(
        &self,
        definition: DefId,
        trait_definition: DefId,
    ) -> Result<ProofSite, &'static str> {
        if self.imported_conditional_member_trait(definition)? != Some(trait_definition) {
            return Err(
                "imported const-member defining requests require their authenticated source-profile counterpart",
            );
        }
        let catalog = self
            .crates
            .as_deref()
            .and_then(super::crates::Linkage::origin_catalog)
            .ok_or("imported const member lost its actual origin catalog")?;
        let trait_header = self
            .generics
            .traits
            .as_deref()
            .and_then(|traits| traits.header(crate::types::ItemId(trait_definition.0)))
            .ok_or("imported const member lost its actual trait header")?;
        // Import preparation authenticates the implicit Self clause at the
        // first position after the canonical prefix of these same rows.
        let (inputs, canonical) = trait_header
            .inputs
            .imported_parts(catalog, trait_definition)?;
        let input = inputs
            .get(canonical.len())
            .ok_or("imported const member lost its actual implicit Self input")?;
        Ok(input.source)
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
                return Ok(if self.nonconst_trait_member(definition)? {
                    FunctionContext::Runtime
                } else {
                    FunctionContext::ConstTraitDefault { trait_definition }
                });
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
