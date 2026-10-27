//! The SIMD FFI gate applies to direct types in actual foreign declarations.
use crate::{
    resolve::{
        bindings::FeatureName, expansion::MacroAllowanceSession, package::Error, unit::NodeKey,
    },
    sema::{declarations::Checker, model::Function},
    syntax::item::ItemKind,
    types::{Kind, TypeId, aggregate::Representation},
};
impl Checker<'_> {
    pub(in crate::sema::declarations) fn validate_foreign_simd_type(
        &self,
        function: Function,
        ty: TypeId,
        index: usize,
        allowances: &mut MacroAllowanceSession<'_>,
    ) -> Result<(), Error> {
        if !self
            .report
            .resolution
            .foreign_owners
            .contains_key(&function.definition)
        {
            return Ok(());
        }
        let Some(Kind::Adt(item, _)) = self.facts.types.kind(ty) else {
            return Ok(());
        };
        if !self
            .facts
            .aggregates
            .get(item)
            .is_some_and(|definition| definition.repr.representation == Representation::Simd)
        {
            return Ok(());
        }
        if self
            .report
            .package
            .requested_simd_feature(true)
            .map_err(|e| self.error(function.node, e))?
        {
            return Ok(());
        }
        let unit = self.report.package.unit(function.node.file);
        let node = if let Some(parameter) = unit
            .configured
            .ast
            .list(function.parameters)
            .iter()
            .copied()
            .filter(|node| unit.configured.active[node.get() as usize])
            .nth(index)
        {
            super::super::parameter_type(
                self.report,
                NodeKey {
                    node: parameter,
                    ..function.node
                },
            )?
        } else {
            let ItemKind::Function {
                result: Some(result),
                ..
            } = self.item(function.node)?.kind
            else {
                return Err(self.error(function.node, "foreign SIMD result has no source type"));
            };
            result
        };
        let span = unit.configured.ast.node(node).span;
        if !allowances.allows_at(
            function.node.file,
            span,
            &FeatureName::new("simd_ffi").map_err(|e| self.error(function.node, e))?,
        )? {
            return Err(Error::at(
                function.node.file,
                span.start,
                "foreign SIMD declaration requires simd_ffi",
            ));
        }
        Ok(())
    }
}
