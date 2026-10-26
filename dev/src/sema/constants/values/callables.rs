//! Reification keeps callable identity distinct from a zero-sized closure value.
use super::{Constant, Evaluator, Failure, Reader, Type, TypeId, View};
use crate::types::instance::Instance;

impl Reader<'_> {
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
        Ok(Some((selection, binding, config)))
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
