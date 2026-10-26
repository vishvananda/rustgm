//! Borrow declaration facts without inventing syntax for a source-free body.
use crate::{
    resolve::{
        body_origin::{BodyOrigin, DeclarationOrigin},
        model::DefId,
    },
    types::{TypeId, parameters::environment::Environment},
};
use std::sync::Arc;

/// Declaration identity and authenticated body presence carry no body environment.
/// Complete assumptions and the lifetime scheme remain in their owning tables.
pub(crate) struct DeclarationHeader {
    pub declaration: DeclarationOrigin,
    pub body: Option<BodyOrigin>,
    pub signature: TypeId,
    pub universals: u32,
}

/// The declaration signature is distinct from its relocated public callable.
/// A missing body is explicit; it does not classify a foreign declaration.
pub(crate) struct FunctionHeader<'a> {
    pub declaration: DeclarationOrigin,
    pub body: Option<BodyOrigin>,
    pub signature: TypeId,
    pub universals: u32,
    pub environment: Option<&'a Arc<Environment>>,
}
impl super::model::Declarations {
    #[cfg(test)]
    pub(crate) const fn metadata_model_layouts() -> (usize, usize, usize, usize) {
        (
            size_of::<super::associated::Owner>(),
            size_of::<super::associated::OwnerKind>(),
            size_of::<super::traits::Header>(),
            size_of::<super::traits::implementations::Implementation>(),
        )
    }
    pub(crate) fn declaration_header(
        &self,
        definition: DefId,
    ) -> Result<Option<DeclarationHeader>, &'static str> {
        let Some(function) = self.source_header(definition)? else {
            let Some(linkage) = self.crates.as_ref() else {
                return Ok(None);
            };
            self.require_imported_callable(linkage, definition)?;
            return linkage.original_declaration_header(definition);
        };
        Ok(Some(DeclarationHeader {
            declaration: function.node.into(),
            body: function.body.map(BodyOrigin::Source),
            signature: function.signature,
            universals: self
                .lifetimes
                .functions
                .get(&definition)
                .map_or(0, |s| s.total),
        }))
    }
    fn source_header(
        &self,
        definition: DefId,
    ) -> Result<Option<&super::model::Function>, &'static str> {
        let Some(&index) = self.function_indices.get(&definition) else {
            return Ok(None);
        };
        let function = self
            .functions
            .get(index)
            .ok_or("function header index is stale")?;
        if function.definition != definition {
            return Err("function header index belongs to another declaration");
        }
        Ok(Some(function))
    }
    fn require_imported_callable(
        &self,
        linkage: &super::crates::Linkage,
        definition: DefId,
    ) -> Result<(), &'static str> {
        linkage.require_original_store(&self.types)?;
        self.require_imported_callable_after_store(linkage, definition)
    }
    fn require_imported_callable_after_store(
        &self,
        linkage: &super::crates::Linkage,
        definition: DefId,
    ) -> Result<(), &'static str> {
        if let Some((signature, constant)) = linkage.original_callable(definition)
            && (!matches!(self.definitions.get(definition.0 as usize).copied().flatten().and_then(|ty| self.types.kind(ty)),
                Some(crate::types::Kind::FunctionItem {item, signature: actual, ..}) if item.0 == definition.0 && actual == signature)
                || self.is_const_function(definition) != constant)
        {
            return Err("imported function changed its canonical callable or const declaration");
        }
        Ok(())
    }
    pub(crate) fn function_header(
        &self,
        definition: DefId,
    ) -> Result<Option<FunctionHeader<'_>>, &'static str> {
        let Some(function) = self.source_header(definition)? else {
            let Some(linkage) = self.crates.as_ref() else {
                return Ok(None);
            };
            linkage.require_original_store(&self.types)?;
            let header = linkage.original_header(definition)?;
            self.require_imported_callable_after_store(linkage, definition)?;
            return Ok(header);
        };
        Ok(Some(FunctionHeader {
            declaration: function.node.into(),
            body: function.body.map(BodyOrigin::Source),
            signature: function.signature,
            universals: self
                .lifetimes
                .functions
                .get(&definition)
                .map_or(0, |s| s.total),
            environment: self.generics.body_environment(definition),
        }))
    }
}

#[cfg(test)]
#[path = "../../tests/invariants/function_headers.rs"]
mod tests;
