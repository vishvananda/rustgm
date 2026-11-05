//! Authored storage shape is published only after the original body has been typed.
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::model::{Declarations, Function},
    types::{Kind, TypeId, opaque::OpaqueId},
};

pub(crate) struct AuthoredBody {
    pub identity: OpaqueId,
    pub future: TypeId,
    pub signature: TypeId,
    pub output: TypeId,
}

impl Declarations {
    /// Called only after the actual `AsyncBody`'s complete region and loan proof.
    /// Closing/waking remains in the existing opaque publisher; this is no CFG permit.
    pub(crate) fn infer_checked_async_future(
        &mut self,
        owner: crate::resolve::model::DefId,
        arena: &crate::types::arena::Identity,
    ) -> Result<(), &'static str> {
        if !self.types.has_identity(arena) {
            return Err("checked async publication belongs to another actual type store");
        }
        let function = self
            .function_indices
            .get(&owner)
            .and_then(|&index| self.functions.get(index))
            .ok_or("checked async publication lost its actual source function")?;
        let Some(body) = self.authored_async_body(function)? else {
            return Err("checked async publication requires its actual authored body");
        };
        let declaration = self
            .opaques
            .declarations
            .get_mut(&body.identity)
            .ok_or("checked async publication lost its actual opaque owner")?;
        if declaration
            .inferred
            .is_some_and(|previous| previous != body.future)
            || declaration
                .hidden
                .is_some_and(|previous| previous != body.future)
            || !declaration.free_regions.is_empty()
        {
            return Err("checked async publication changed its actual future representation");
        }
        declaration.inferred = Some(body.future);
        Ok(())
    }
    /// Current checked defining publication; this is never a selected body permit.
    pub(crate) fn checked_future_definition(
        &self,
        future: TypeId,
    ) -> Result<crate::resolve::model::DefId, &'static str> {
        let Some(Kind::Future {
            identity,
            arguments,
        }) = self.types.kind(future)
        else {
            return Err("checked future application requires its genuine type identity");
        };
        let declaration = self
            .opaques
            .declarations
            .get(&identity)
            .ok_or("checked future application lost its actual opaque owner")?;
        declaration.require_current(self, identity)?;
        let function = self
            .function_indices
            .get(&declaration.owner)
            .and_then(|&index| self.functions.get(index))
            .ok_or("checked future application lost its actual source function")?;
        let actual = self
            .authored_async_body(function)?
            .ok_or("checked future application has no genuine authored template")?;
        self.types.future_application(identity, arguments)?;
        if actual.identity != identity
            || declaration.hidden != Some(actual.future)
            || declaration.inferred != Some(actual.future)
        {
            return Err("future application requires its checked defining publication");
        }
        Ok(declaration.owner)
    }
    /// Type/owner identity only; a caller still needs the actual checked template.
    pub(crate) fn authored_async_body(
        &self,
        function: &Function,
    ) -> Result<Option<AuthoredBody>, &'static str> {
        let Some(Kind::Function { result, .. }) = self.types.kind(function.signature) else {
            return Err("async body lost its actual constructor signature");
        };
        let Some(Kind::Opaque(identity, arguments)) = self.types.kind(result) else {
            return Ok(None);
        };
        let Some((application, constructor, signature, declared_output)) =
            self.types.authored_future_template(identity)?
        else {
            return Ok(None);
        };
        let declaration = self
            .opaques
            .declarations
            .get(&identity)
            .ok_or("async body lost its actual opaque declaration")?;
        declaration.require_current(self, identity)?;
        let catalog = self
            .source_bindings
            .as_deref()
            .ok_or("deferred source body lost its actual declaration catalog")?;
        let future_role = catalog
            .language(crate::support::bindings::LangItem::Future)
            .ok_or("deferred source body lost its actual Future declaration")?;
        let output_role = catalog
            .language(crate::support::bindings::LangItem::FutureOutput)
            .ok_or("deferred source body lost its actual Output declaration")?;
        super::check_future_contract(
            &self.types,
            declaration,
            declared_output,
            future_role.definition,
            output_role.definition,
        )?;
        let header = self
            .function_header(function.definition)?
            .ok_or("async body lost its actual callable header")?;
        let Some(Kind::Function { result: output, .. }) = self.types.kind(signature) else {
            return Err("async body lost its actual internal Output signature");
        };
        if application != result
            || constructor != function.signature
            || declaration.application != result
            || declaration.owner != function.definition
            || declaration.origin.source() != Some(function.node)
            || function.body.is_none()
            || self
                .opaques
                .owners
                .get(&function.definition)
                .and_then(|ids| ids.last())
                != Some(&identity)
            || !self.typed_function_bodies.contains(&function.definition)
            || header.declaration.source() != Some(function.node)
            || header.body != function.body.map(Into::into)
            || header.signature != constructor
            || self.opaques.body_results.get(&function.definition) != Some(&output)
        {
            return Err("async body changed its actual owner, ordinal, body or capture contract");
        }
        let future = self
            .types
            .find_kind(Kind::Future {
                identity,
                arguments,
            })
            .ok_or("async body lost its actual canonical future application")?;
        Ok(Some(AuthoredBody {
            identity,
            future,
            signature,
            output,
        }))
    }
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
