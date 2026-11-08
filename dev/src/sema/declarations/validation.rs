//! Validate declaration uses under their own immutable assumptions.
use super::{Checker, Error, Type};

impl Checker<'_> {
    pub(super) fn validate_owner_bounds(&mut self) -> Result<(), Error> {
        for index in 0..self.facts.associated.owners().len() {
            let Some(owner) = self
                .facts
                .associated
                .source_at(index)
                .map_err(Error::unlocated)?
            else {
                continue;
            };
            if self
                .facts
                .associated
                .schema(owner.self_definition)
                .is_none()
            {
                continue;
            }
            let definition = self
                .facts
                .associated
                .trait_owner(owner.self_definition)
                .unwrap_or(owner.self_definition);
            if !self.facts.generics.predicates.contains_key(&definition) {
                continue;
            }
            let environment = self.facts.generics.environments.get(&definition).cloned();
            let mut proof = self.declaration_proof(definition, environment);
            self.validate_bounds(definition, &mut proof)?;
            self.retain_trait_proof(definition, owner.source, &mut proof);
        }
        self.validate_alias_declarations()
    }
    pub(super) fn validate_signatures(&mut self) -> Result<(), Error> {
        let mut allowances = self.report.macro_allowances();
        let mut unsized_parameters = crate::sema::unsized_parameters::Draft::default();
        for index in 0..self.facts.functions.len() {
            let function = self.facts.functions[index];
            self.validate_trait_method(&function)?;
            crate::sema::target_features::validate_method(&self.facts, function.definition)?;
            self.validate_method_receiver(&function)?;
            let environment = self
                .facts
                .generics
                .environments
                .get(&function.definition)
                .cloned();
            let mut proof = self.declaration_proof(function.definition, environment);
            self.validate_bounds(function.definition, &mut proof)?;
            let Some(Type::Function {
                parameters, result, ..
            }) = self.facts.types.kind(function.signature)
            else {
                return Err(self.error(function.node, "invalid function signature"));
            };
            let result = self
                .facts
                .async_body_output(self.report, &function)?
                .unwrap_or(result);
            let count = self
                .facts
                .types
                .list(parameters)
                .ok_or_else(|| self.error(function.node, "invalid parameters"))?
                .len();
            for index in 0..=count {
                let ty = if index == count {
                    result
                } else {
                    self.facts
                        .types
                        .list(parameters)
                        .ok_or_else(|| self.error(function.node, "invalid parameters"))?[index]
                };
                self.validate_foreign_simd_type(function, ty, index, &mut allowances)?;
                let shape = proof
                    .check(&mut self.facts, ty)
                    .map_err(|e| self.error(function.node, e))?;
                if !shape.well_formed {
                    return Err(self.error(
                        function.node,
                        "function signature contains an ill-formed type",
                    ));
                }
                if !(shape.sized
                    || (index < count && unsized_parameters.enabled(self.report)?)
                    || (function.body.is_none()
                        && self
                            .facts
                            .associated
                            .trait_owner(function.definition)
                            .is_some()))
                {
                    return Err(self.error(
                        function.node,
                        "function parameters and result must be sized",
                    ));
                }
            }
            self.retain_trait_proof(function.definition, function.node, &mut proof);
        }
        for index in 0..self.facts.initializers.len() {
            let initializer = self.facts.initializers[index];
            let environment = self
                .facts
                .generics
                .environments
                .get(&initializer.definition)
                .cloned();
            let mut proof = self.declaration_proof(initializer.definition, environment);
            let shape = proof
                .check(&mut self.facts, initializer.ty)
                .map_err(|e| self.error(initializer.node, e))?;
            if !shape.well_formed || !shape.sized {
                return Err(self.error(
                    initializer.node,
                    "constant and static types must be well-formed and sized",
                ));
            }
            if matches!(
                initializer.kind,
                crate::sema::model::InitializerKind::Static { mutable: false }
            ) {
                self.require_shared_static_sync(&initializer, &mut proof)?;
            }
            self.retain_trait_proof(initializer.definition, initializer.node, &mut proof);
        }
        self.validate_constant_declarations()?;
        Ok(())
    }
    /// rustc's well-formedness check for a shared static: the `sync` lang
    /// item must exist and the static's type must implement it. Foreign
    /// statics have no initializer, and a `static mut` needs neither.
    fn require_shared_static_sync(
        &mut self,
        initializer: &crate::sema::model::Initializer,
        proof: &mut crate::sema::nominals::Proof,
    ) -> Result<(), Error> {
        use crate::types::{
            GenericArgs, ItemId,
            auto_traits::AutoTrait,
            parameters::predicates::{Bound, Predicate},
            traits::Application,
        };
        let node = initializer.node;
        let bound = match self.facts.source_bindings.as_deref() {
            // The built-in library always provides its own structural Sync.
            None => Bound::Auto(AutoTrait::Sync),
            Some(catalog) => {
                let declaration =
                    match catalog.auto_marker(crate::support::bindings::LangItem::Sync, true) {
                        Ok(Some(declaration)) => declaration,
                        Ok(None) => return Err(self.error(node, "requires `sync` lang_item")),
                        Err(message) => return Err(self.error(node, message)),
                    };
                let application = self
                    .facts
                    .types
                    .intern_trait_application(Application {
                        item: ItemId(declaration.definition.0),
                        arguments: GenericArgs::EMPTY,
                    })
                    .map_err(|message| self.error(node, message))?;
                Bound::Trait(application)
            }
        };
        let predicate = Predicate {
            ty: initializer.ty,
            bound,
        };
        if !proof
            .prove(&mut self.facts, predicate)
            .map_err(|message| self.error(node, message))?
        {
            return Err(self.error(node, "shared static type must implement Sync"));
        }
        Ok(())
    }
}
