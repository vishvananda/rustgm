//! Variadic arguments retain their actual types after ordinary inference fallback.
use super::{Checker, Error, Kind, NodeId, Primitive, Type};

impl Checker<'_> {
    pub(super) fn variadic_readiness(
        &self,
        function: crate::sema::model::Function,
    ) -> Result<Option<super::Dependency>, Error> {
        if !matches!(self.facts.types.kind(function.signature),Some(Type::Function{convention,..}) if convention.variadic)
        {
            return Ok(None);
        }
        let definition = crate::sema::compiler_items::vararg::declaration(self.facts)
            .map_err(|message| {
                crate::sema::model::Declarations::error(self.report, function.node, message)
            })?
            .definition;
        let item = crate::types::ItemId(definition.0);
        let requirement = if self
            .facts
            .definitions
            .get(definition.0 as usize)
            .copied()
            .flatten()
            .is_none()
        {
            Some(super::Requirement::Definition(definition))
        } else if self.facts.aggregates.get(item).is_none() {
            Some(super::Requirement::Fields(item))
        } else if !self.facts.generics.nominal_contracts.contains(&definition) {
            Some(super::Requirement::Nominal(item))
        } else {
            None
        };
        Ok(requirement.map(|requirement| super::Dependency {
            requirement,
            source: function.node,
        }))
    }
    pub(super) fn variadic_parameter(
        &mut self,
        node: NodeId,
        pattern: NodeId,
        annotation: NodeId,
    ) -> Result<bool, Error> {
        if !matches!(self.kind(annotation), Kind::Variadic) {
            return Ok(false);
        }
        let region = self.body_region(node, crate::sema::model::BodyRegionOrigin::Inference)?;
        let ty = crate::sema::compiler_items::vararg::binding_type(self.facts, region)
            .map_err(|message| self.error(node, message))?;
        let term = self.known(node, ty)?;
        self.binding(pattern, term, None)?;
        let binding = self
            .facts
            .node(self.key(pattern))
            .and_then(|fact| fact.binding);
        let key = self.key(node);
        self.facts
            .node_mut(key)
            .ok_or_else(|| {
                crate::sema::model::Declarations::error(
                    self.report,
                    key,
                    "variadic parameter fact is missing",
                )
            })?
            .binding = binding;
        self.selected(node, crate::sema::model::Operation::Variadic)?;
        self.save(node, term);
        Ok(true)
    }
    pub(super) fn validate_variadic_arguments(
        &self,
        call: NodeId,
        fixed: usize,
        solved: &crate::types::infer::Solved,
    ) -> Result<(), Error> {
        let Kind::Call { arguments, .. } = self.kind(call) else {
            return Err(self.error(call, "variadic call requires ordinary call syntax"));
        };
        let unit = self.report.package.unit(self.state.file);
        for &argument in unit
            .configured
            .ast
            .list(arguments)
            .iter()
            .filter(|node| self.active(**node))
            .skip(fixed)
        {
            let term = self.term(argument)?;
            let ty = solved.types[term.0 as usize];
            let message = match self.facts.types.kind(ty) {
                Some(Type::Primitive(Primitive::F16 | Primitive::F128)) => {
                    "wide floating variadic ABI is not implemented"
                }
                Some(Type::Primitive(
                    Primitive::Bool
                    | Primitive::I8
                    | Primitive::U8
                    | Primitive::I16
                    | Primitive::U16
                    | Primitive::F32,
                )) => "variadic argument requires an explicit C promotion cast",
                Some(Type::FunctionItem { .. }) => {
                    "variadic function item requires an explicit function pointer cast"
                }
                _ => continue,
            };
            return Err(self.error(argument, message));
        }
        Ok(())
    }
}
