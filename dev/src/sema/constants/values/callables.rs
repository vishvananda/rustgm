//! Reification keeps callable identity distinct from a zero-sized closure value.
use super::{Constant, Evaluator, Failure, Reader, Type, TypeId, View};
use crate::types::instance::Instance;

impl Reader<'_> {
    /// Actual guard emission retains source hook absence/completion distinctions.
    pub(crate) fn require_source_assertion(
        self,
        role: crate::support::bindings::LangItem,
    ) -> Result<(), &'static str> {
        crate::sema::compiler_items::guard_panics::require_guard(self.facts, role)
    }
    /// The completed bodyless declaration of a supported runtime intrinsic.
    /// The bring-up profile also admits unsupported declarations for trapping.
    pub(crate) fn source_bodiless_intrinsic(
        self,
        definition: crate::resolve::model::DefId,
    ) -> Result<Option<&'static str>, &'static str> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(crate::support::bindings::SourceIntrinsic::Declaration(identity)) =
            catalog.intrinsic_kind(definition)?
        else {
            return Ok(None);
        };
        let name = identity.name();
        if !crate::sema::compiler_items::strict_bodiless_intrinsic(name)
            && crate::support::bringup::selected().is_none()
        {
            return Ok(None);
        }
        let bodiless = self
            .facts
            .function_indices
            .get(&definition)
            .and_then(|&index| self.facts.functions.get(index))
            .is_some_and(|function| function.body.is_none());
        Ok(bodiless.then_some(name))
    }
    pub(crate) fn source_bit_count(
        self,
        instance: Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::bit_counts::Selection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.bit_count(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }
    pub(crate) fn source_atomic_access(
        self,
        name: &str,
        callee: TypeId,
        signature: TypeId,
    ) -> Result<Option<crate::sema::compiler_items::checked_atomic_access::Selection>, &'static str>
    {
        crate::sema::compiler_items::checked_atomic_access::select(
            self.facts, name, callee, signature,
        )
    }
    pub(crate) fn require_source_future_assertion(self) -> Result<(), &'static str> {
        crate::sema::compiler_items::guard_panics::require_future_guard(self.facts)
    }
    pub(crate) fn has_runtime_layouts(self) -> bool {
        self.facts
            .source_bindings
            .as_deref()
            .is_some_and(crate::sema::compiler_items::Catalog::has_runtime_layouts)
    }
    pub(crate) fn runtime_layout(
        self,
        instance: Instance,
    ) -> Option<crate::sema::compiler_items::RuntimeLayout> {
        self.facts
            .source_bindings
            .as_deref()?
            .runtime_layout(instance)
    }

    pub(crate) fn source_transmute(
        self,
        instance: Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::TransmuteSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.reinterpretation_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }

    pub(crate) fn source_abort(
        self,
        instance: Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::AbortSelection>, &'static str> {
        self.facts
            .source_bindings
            .as_deref()
            .map(|catalog| catalog.abort_selection(self.facts, instance, origin))
            .transpose()
            .map(Option::flatten)
    }

    pub(crate) fn source_runtime_check(
        self,
        instance: Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<
        Option<(
            crate::sema::compiler_items::RuntimeCheckSelection,
            crate::support::runtime_checks::RuntimeCheckBinding,
            crate::support::runtime_checks::RuntimeCheckConfig,
        )>,
        &'static str,
    > {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selection) = catalog.runtime_check_selection(self.facts, instance)? else {
            return Ok(None);
        };
        let (binding, config) = catalog.runtime_check_binding(origin)?;
        Ok(Some((selection, selection.binding(binding), config)))
    }
    pub(crate) fn source_select(
        self,
        instance: Instance,
    ) -> Result<Option<crate::sema::compiler_items::Select>, &'static str> {
        crate::sema::compiler_items::source_select(self.facts, instance)
    }
    pub(crate) fn source_drop_glue_definition(
        self,
    ) -> Result<Option<crate::resolve::model::DefId>, &'static str> {
        let Some(catalog) = self.facts.source_bindings.as_deref() else {
            return Ok(None);
        };
        Ok(catalog
            .validated_language(crate::support::bindings::LangItem::DropGlue)?
            .map(|declaration| declaration.definition))
    }
    pub(crate) fn source_drop_glue(
        self,
        instance: Instance,
    ) -> Result<Option<crate::sema::compiler_items::DropGlueSelection>, &'static str> {
        crate::sema::compiler_items::source_drop_glue(self.facts, instance)
    }
}

impl Evaluator<'_> {
    pub(crate) fn checked_source_const_panic(
        &self,
        instance: Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::ConstPanicSelection>, Failure> {
        Ok(crate::sema::compiler_items::source_const_panic(
            self.queries.report,
            self.facts(),
            instance,
            origin,
        )?)
    }
    pub(crate) fn checked_source_transmute(
        &self,
        instance: Instance,
        origin: crate::resolve::body_origin::BodyOrigin,
    ) -> Result<Option<crate::sema::compiler_items::TransmuteSelection>, Failure> {
        let Some(catalog) = self.facts().source_bindings.as_deref() else {
            return Ok(None);
        };
        let Some(selected) = catalog.reinterpretation_selection(self.facts(), instance, origin)?
        else {
            return Ok(None);
        };
        catalog.require_transmute_report(self.queries.report, self.facts(), &selected)?;
        Ok(Some(selected))
    }

    pub(crate) fn reify(&self, source: TypeId, value: Constant) -> Result<Constant, Failure> {
        match self.facts().types.kind(source) {
            Some(Type::FunctionItem { .. } | Type::Function { .. }) => {
                value
                    .as_function()
                    .ok_or("constant function conversion lost its selected callable")?;
                Ok(value)
            }
            Some(Type::Closure {
                environment: TypeId::UNIT,
                ..
            }) => {
                if !matches!(
                    Reader::new(self.facts()).view(value)?,
                    View::Product {
                        length: 0,
                        variant: None,
                    }
                ) {
                    return Err("constant closure conversion requires an empty environment".into());
                }
                Ok(Constant::function(Instance::closure(
                    &self.facts().types,
                    source,
                )?))
            }
            _ => Err("constant function conversion requires a noncapturing callable".into()),
        }
    }
}
