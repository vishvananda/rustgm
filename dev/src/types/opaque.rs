//! Published opaque identities never expose their defining body's representation.
pub(super) mod bounds;
use super::{GenericArgs, Kind, Properties, Store, TypeId, copy};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OpaqueId(pub u32);

/// A defining-body proof keeps each permitted name in its own capture context.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegionChoice {
    pub captures: GenericArgs,
    pub region: super::RegionId,
    pub choice: super::RegionId,
}

/// Bounds available to every caller, independent of the eventual hidden type.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Declaration {
    pub trait_result: bool,
    pub sized: bool,
    pub copies: copy::Properties,
}

impl Store {
    /// Physical storage is separate from canonical type equality and public bounds.
    /// Only the defining-context preparation publishes these fully substituted heads.
    /// # Errors
    /// Rejects an opaque application whose representation has not been prepared.
    pub fn storage_head(&self, ty: TypeId) -> Result<TypeId, &'static str> {
        if !matches!(self.kind(ty), Some(Kind::Opaque(..))) {
            return Ok(ty);
        }
        self.storage_heads
            .as_ref()
            .and_then(|heads| heads.get(&ty))
            .copied()
            .ok_or("opaque storage requires a checked defining type")
    }

    pub(crate) fn publish_storage_head(
        &mut self,
        ty: TypeId,
        head: TypeId,
    ) -> Result<(), &'static str> {
        if !matches!(self.kind(ty), Some(Kind::Opaque(..)))
            || matches!(self.kind(head), None | Some(Kind::Opaque(..)))
        {
            return Err("invalid opaque storage head");
        }
        let heads = self.storage_heads.get_or_insert_with(Default::default);
        if heads.get(&ty).is_some_and(|previous| *previous != head) {
            return Err("opaque storage representation changed");
        }
        heads.insert(ty, head);
        Ok(())
    }

    /// Reserve an identity before any application or hidden type is published.
    /// # Errors
    /// Rejects inconsistent builtin promises and exhausted compact identities.
    pub fn declare_opaque(&mut self, declaration: Declaration) -> Result<OpaqueId, &'static str> {
        if declaration.copies.copy && !declaration.copies.clone_
            || declaration.copies.clone_ && !declaration.sized
        {
            return Err("inconsistent opaque builtin promises");
        }
        let declarations = self.opaques.get_or_insert_with(Default::default);
        let id = OpaqueId(u32::try_from(declarations.len()).map_err(|_| "too many opaque types")?);
        declarations.push(declaration);
        Ok(id)
    }

    #[must_use]
    pub fn opaque_declaration(&self, id: OpaqueId) -> Option<Declaration> {
        self.opaques.as_ref()?.get(id.0 as usize).copied()
    }

    pub(super) fn opaque_properties(
        &self,
        id: OpaqueId,
        arguments: GenericArgs,
    ) -> Result<Properties, &'static str> {
        let declaration = self
            .opaque_declaration(id)
            .ok_or("invalid opaque identity")?;
        let mut properties = Properties::default();
        self.argument_properties(arguments, &mut properties)?;
        properties.set(super::Property::Sized, declaration.sized);
        properties.set(super::Property::Opaques, true);
        if declaration.trait_result {
            properties.set(super::Property::Projections, true);
        }
        Ok(properties)
    }
}
