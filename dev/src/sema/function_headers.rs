//! Borrow declaration facts without inventing syntax for a source-free body.
use crate::{
    resolve::{
        body_origin::{BodyOrigin, DeclarationOrigin},
        model::DefId,
    },
    types::{TypeId, parameters::environment::Environment},
};
use std::sync::Arc;

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
    pub(crate) fn function_header(
        &self,
        definition: DefId,
    ) -> Result<Option<FunctionHeader<'_>>, &'static str> {
        let Some(&index) = self.function_indices.get(&definition) else {
            let Some(linkage) = self.crates.as_ref() else {
                return Ok(None);
            };
            linkage.require_original_store(&self.types)?;
            let header = linkage.original_header(definition)?;
            if let Some((signature, constant)) = linkage.original_callable(definition)
                && (!matches!(self.definitions.get(definition.0 as usize).copied().flatten().and_then(|ty| self.types.kind(ty)),
                    Some(crate::types::Kind::FunctionItem {item, signature: actual, ..}) if item.0 == definition.0 && actual == signature)
                    || self.is_const_function(definition) != constant)
            {
                return Err(
                    "imported function changed its canonical callable or const declaration",
                );
            }
            return Ok(header);
        };
        let function = self
            .functions
            .get(index)
            .ok_or("function header index is stale")?;
        if function.definition != definition {
            return Err("function header index belongs to another declaration");
        }
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
