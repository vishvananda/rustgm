//! Const parameter identity precedes the mandatory completed source marker proof.
#[cfg(test)]
#[path = "../../../../tests/invariants/unstable_bound_parameters.rs"]
mod feature_bound_tests;
use super::{Argument, Checker, ConstKind, DefId, Error, NodeKey, Type, TypeId};
use crate::{
    support::bindings::LangItem,
    types::{
        Primitive,
        parameters::predicates::{Bound, Predicate},
    },
};

impl Checker<'_> {
    pub(super) fn require_const_parameter_identity(
        &self,
        key: NodeKey,
        ty: TypeId,
    ) -> Result<(), Error> {
        if matches!(self.facts.types.kind(ty), Some(Type::Primitive(p))
            if p.integer().is_some() || matches!(p, Primitive::Bool | Primitive::Char))
        {
            return Ok(());
        }
        if !self.facts.types.supports_const_parameter_type(ty) {
            return Err(self.error(key, "invalid const parameter type"));
        }
        if !self
            .report
            .package
            .requested_aggregate_const_parameters()
            .map_err(|e| self.error(key, e))?
        {
            return Err(self.error(
                key,
                "aggregate const parameter types require min_adt_const_params or adt_const_params",
            ));
        }
        if self.facts.source_bindings.is_none() {
            return Err(self.error(
                key,
                "aggregate const parameters require actual source marker declarations",
            ));
        }
        if self.facts.types.contains_parameters(ty) != Some(false)
            || self.facts.types.contains_projections(ty) != Some(false)
            || self.facts.types.has_free_regions(ty, 0) != Some(false)
        {
            return Err(self.error(
                key,
                "dependent const parameter types require their separate language profile",
            ));
        }
        Ok(())
    }

    pub(in crate::sema::declarations) fn validate_const_parameter_types(
        &mut self,
    ) -> Result<(), Error> {
        if !self
            .report
            .package
            .requested_aggregate_const_parameters()
            .map_err(Error::unlocated)?
        {
            return Ok(());
        }
        crate::sema::header_attributes::reserve_source_contracts(
            &mut self.facts,
            self.report.resolution.definitions.len(),
            0,
        )?;
        for index in 0..self.report.resolution.definitions.len() {
            let owner = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::unlocated("const owner identity overflow"))?,
            );
            let count = self
                .facts
                .generics
                .parameters
                .get(&owner)
                .map_or(0, Vec::len);
            for slot in 0..count {
                let parameter = self.facts.generics.parameters[&owner][slot];
                let Argument::Const(id) = parameter.argument else {
                    continue;
                };
                let Some(ConstKind::Parameter(_)) = self.facts.types.const_kind(id) else {
                    return Err(Error::unlocated(
                        "const parameter declaration lost its rigid identity",
                    ));
                };
                let key = self.report.resolution.definitions[parameter.binding.0 as usize]
                    .node
                    .ok_or_else(|| {
                        Error::unlocated("const parameter lacks its actual local source")
                    })?;
                let ty = self
                    .facts
                    .types
                    .const_type(id)
                    .ok_or_else(|| self.error(key, "const parameter lost its actual type"))?;
                // Each environment proof retains its own lifetime witnesses, so
                // the next parameter's proof is numbered after the retained one.
                let environment = self.facts.generics.environments.get(&owner).cloned();
                let mut proof = self.declaration_proof(owner, environment);
                self.prove_const_parameter_type(key, ty, &mut proof)?;
                self.retain_trait_proof(owner, key, &mut proof);
            }
        }
        Ok(())
    }
    fn prove_const_parameter_type(
        &mut self,
        key: NodeKey,
        ty: TypeId,
        proof: &mut crate::sema::nominals::Proof,
    ) -> Result<(), Error> {
        let declaration = self
            .facts
            .source_bindings
            .as_deref()
            .ok_or_else(|| {
                self.error(
                    key,
                    "const parameter proof requires its actual source catalog",
                )
            })?
            .validated_language(LangItem::ConstParamTy)
            .map_err(|e| self.error(key, e))?
            .ok_or_else(|| self.error(key, "const parameter type requires actual ConstParamTy"))?;
        crate::sema::header_attributes::reserve_source_contracts(&mut self.facts, 8, 0)?;
        let application = crate::types::traits::Application {
            item: crate::types::ItemId(declaration.definition.0),
            arguments: crate::types::GenericArgs::EMPTY,
        };
        let application = if let Some(id) = self.facts.types.find_trait_application(application) {
            id
        } else {
            let growth = self.facts.types.trait_application_growth();
            crate::sema::header_attributes::reserve_source_contracts(
                &mut self.facts,
                growth.work,
                growth.bytes,
            )?;
            self.facts
                .types
                .intern_trait_application(application)
                .map_err(|e| self.error(key, e))?
        };
        // rustc's `check_param_wf` proves `Ty: ConstParamTy_` under the
        // owner's param-env, so the owner's `#[unstable_feature_bound]` proves
        // an impl's feature condition. The type itself is already closed.
        if !proof
            .prove(
                &mut self.facts,
                Predicate {
                    ty,
                    bound: Bound::Trait(application),
                },
            )
            .map_err(|e| self.error(key, e))?
        {
            return Err(self.error(
                key,
                "const parameter type must implement actual ConstParamTy",
            ));
        }
        Ok(())
    }
}
