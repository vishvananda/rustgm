//! Authored storage shape is published only after the original body has been typed.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::{Declarations, Function},
    types::Kind,
};

impl Declarations {
    pub(crate) fn prepare_authored_future_shape(
        &mut self,
        report: &Report,
        function: &Function,
    ) -> Result<(), Error> {
        let Some(body_output) = self.async_body_output(report, function)? else {
            return Ok(());
        };
        if function.body.is_none() {
            return Ok(());
        }
        let error = |message| report.body_error(function.node, message);
        if self
            .source_bindings
            .as_deref()
            .and_then(|catalog| {
                catalog.language(crate::support::bindings::LangItem::AsyncDropInPlace)
            })
            .is_some_and(|declaration| declaration.definition == function.definition)
        {
            // Compiler-operation recipes never acquire authored execution/storage authority.
            return Ok(());
        }
        if !self.typed_function_bodies.contains(&function.definition) {
            return Err(error(
                "future shape requires its actual completed body typing",
            ));
        }
        let Some(Kind::Function { result, .. }) = self.types.kind(function.signature) else {
            return Err(error("future shape lost its actual public signature"));
        };
        let Some(Kind::Opaque(identity, arguments)) = self.types.kind(result) else {
            return Err(error("future shape lost its actual opaque application"));
        };
        let declaration = self
            .opaques
            .declarations
            .get(&identity)
            .ok_or_else(|| error("future shape lost its actual defining owner"))?;
        declaration.require_current(self, identity).map_err(error)?;
        if declaration.owner != function.definition
            || declaration.application != result
            || declaration.origin.source() != Some(function.node)
        {
            return Err(error("future shape changed its actual source ownership"));
        }
        let [
            _,
            crate::types::parameters::clauses::Clause::Predicate(equality),
        ] = declaration.bounds.as_slice()
        else {
            return Err(error(
                "future shape lost its actual declared Output equality",
            ));
        };
        let crate::types::parameters::predicates::Bound::ProjectionEqual(output) = equality.bound
        else {
            return Err(error("future shape lost its actual declared Output result"));
        };
        let (future, member) =
            super::super::compiler_items::futures::signature_roles(report, self)?;
        super::check_future_contract(&self.types, declaration, output, future, member)
            .map_err(error)?;
        self.types
            .declare_future_shape(identity, result, function.signature, output, body_output)
            .map_err(error)?;
        self.types
            .intern(Kind::Future {
                identity,
                arguments,
            })
            .map_err(error)?;
        Ok(())
    }
}
