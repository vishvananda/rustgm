//! A borrowed current Members view authenticates source-free nominal field identities.
use super::{Original, s};
use crate::{
    sema::members::{FieldId, Name, Origin, Selection, VariantId},
    support::crates::{generics::TypeRef, owner_catalog::DeclarationRef},
    types::Kind,
};
impl Original<'_> {
    pub(crate) fn nominal_variant(
        &self,
        root: TypeRef,
        steps: &mut usize,
    ) -> Result<VariantId, &'static str> {
        super::charge(steps)?;
        if !self.rows().descriptor().profile().nominals() {
            return Err("source nominal reconstruction requires its explicit version-ten profile");
        }
        let s::Type::Nominal { declaration, .. } = *self
            .rows()
            .types
            .get(root.0 as usize)
            .ok_or("source nominal value type is unknown")?
        else {
            return Err("source nominal value omitted its actual nominal type recipe");
        };
        if declaration.artifact.0 != 0 {
            return Err("source nominal value belongs to a foreign actual provider");
        }
        let owner = self.declaration(declaration.declaration, steps)?;
        let store = match &self.mapping {
            super::selected::Mapping::Ordinary { store, .. } => *store,
            super::selected::Mapping::Selected { .. } => {
                return Err("provided profiles do not own source nominal storage");
            }
        };
        if !matches!(store.kind(self.ty(root,steps)?),Some(Kind::Adt(item,_)) if item.0 == owner.0)
        {
            return Err("source nominal value changed its canonical declaration identity");
        }
        self.nominal_owner(declaration.declaration, steps)
    }
    fn nominal_owner(
        &self,
        declaration: DeclarationRef,
        steps: &mut usize,
    ) -> Result<VariantId, &'static str> {
        super::charge(steps)?;
        let owner = self.declaration(declaration, steps)?;
        let rows = self.rows();
        let Some(s::HeaderRef::Nominal(index)) = rows.header(declaration)? else {
            return Err("source nominal field lacks its actual owner header");
        };
        let raw = &rows.nominals[index as usize];
        let [variant] = self.members.variants(crate::types::ItemId(owner.0)) else {
            return Err("source nominal has no complete actual member group");
        };
        let actual = self
            .members
            .variants
            .get(variant.0 as usize)
            .ok_or("source nominal member group is unknown")?;
        if actual.owner.0 != owner.0
            || actual.ordinal != 0
            || actual.origin != Origin::External(owner)
            || actual.end - actual.start != raw.fields.end - raw.fields.start
        {
            return Err("source nominal changed its complete actual member owner or field count");
        }
        Ok(*variant)
    }
    pub(crate) fn nominal_field(
        &self,
        owner: DeclarationRef,
        ordinal: u32,
        steps: &mut usize,
    ) -> Result<Selection, &'static str> {
        let variant = self.nominal_owner(owner, steps)?;
        let actual = self
            .members
            .variants
            .get(variant.0 as usize)
            .ok_or("source nominal member group disappeared")?;
        let rows = self.rows();
        let Some(s::HeaderRef::Nominal(index)) = rows.header(owner)? else {
            return Err("source nominal field header disappeared");
        };
        let field = rows.nominals[index as usize]
            .fields
            .slice(&rows.fields)?
            .get(ordinal as usize)
            .ok_or("source nominal field ordinal belongs to another owner")?;
        let id = actual
            .start
            .checked_add(ordinal)
            .filter(|id| *id < actual.end)
            .ok_or("source nominal field is outside its actual canonical group")?;
        let found = self
            .members
            .fields
            .get(id as usize)
            .ok_or("source nominal field disappeared from its actual canonical owner")?;
        let Name::Named(name) = found.name else {
            return Err("source nominal field changed its actual named shape");
        };
        let expected = match &self.mapping {
            super::selected::Mapping::Ordinary { graph, .. } => graph
                .ty(field.ty)
                .map_err(|_| "source nominal field template has no current canonical mapping")?,
            super::selected::Mapping::Selected { .. } => {
                return Err("provided profiles cannot reconstruct ordinary source nominal fields");
            }
        };
        let access = if field.visibility == s::ModuleVisibility::Public {
            crate::resolve::model::Access::Public
        } else {
            crate::resolve::model::Access::ExternalPrivate
        };
        if found.origin != actual.origin
            || found.ty != expected
            || self.members.names.get(name) != field.name
            || found.access != access
        {
            return Err(
                "source nominal field changed its current canonical type, name or access policy",
            );
        }
        Ok(Selection::Nominal(FieldId(id)))
    }
}
