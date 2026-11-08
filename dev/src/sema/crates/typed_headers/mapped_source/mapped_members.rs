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
    /// The consumer's own variant for a validated variant declaration: its
    /// installed constructor, enum owner, ordinal and payload count agree.
    pub(crate) fn variant(
        &self,
        declaration: DeclarationRef,
        steps: &mut usize,
    ) -> Result<VariantId, &'static str> {
        super::charge(steps)?;
        let rows = self.rows();
        let Some(s::HeaderRef::Variant(index)) = rows.header(declaration)? else {
            return Err("source variant operand lacks its actual variant header");
        };
        let raw = rows.enums().variants[index as usize];
        let enumeration = rows.declarations[declaration.0 as usize]
            .definition
            .parent
            .ok_or("source variant lost its actual enum")?;
        let owner = self.declaration(enumeration, steps)?;
        let constructor = self.declaration(declaration, steps)?;
        let variant = self
            .members
            .constructor(constructor)
            .ok_or("source variant has no installed constructor")?;
        let actual = self
            .members
            .variants
            .get(variant.0 as usize)
            .ok_or("source variant member group is unknown")?;
        if actual.owner.0 != owner.0
            || actual.ordinal != raw.ordinal
            || actual.origin != Origin::External(owner)
            || actual.end - actual.start != raw.fields.end - raw.fields.start
        {
            return Err("source variant changed its installed enum, ordinal or payload");
        }
        Ok(variant)
    }
    /// A downcast payload field of a validated variant.
    fn variant_field(
        &self,
        declaration: DeclarationRef,
        ordinal: u32,
        steps: &mut usize,
    ) -> Result<Selection, &'static str> {
        let variant = self.variant(declaration, steps)?;
        let actual = self.members.variants[variant.0 as usize];
        let rows = self.rows();
        let Some(s::HeaderRef::Variant(index)) = rows.header(declaration)? else {
            return Err("source variant field header disappeared");
        };
        let field = rows.enums().variants[index as usize]
            .fields
            .slice(&rows.enums().fields)?
            .get(ordinal as usize)
            .ok_or("source variant field ordinal belongs to another variant")?;
        let id = actual
            .start
            .checked_add(ordinal)
            .filter(|id| *id < actual.end)
            .ok_or("source variant field is outside its actual canonical group")?;
        let found = self
            .members
            .fields
            .get(id as usize)
            .ok_or("source variant field disappeared from its canonical owner")?;
        let expected = match &self.mapping {
            super::selected::Mapping::Ordinary { graph, .. } => graph
                .ty(field.ty)
                .map_err(|_| "source variant field template has no current canonical mapping")?,
            super::selected::Mapping::Selected { .. } => {
                return Err("provided profiles cannot reconstruct source variant fields");
            }
        };
        if found.origin != actual.origin
            || found.ty != expected
            || found.name != Name::Index(ordinal)
            || found.access != crate::resolve::model::Access::Public
        {
            return Err("source variant field changed its canonical type, position or access");
        }
        Ok(Selection::Nominal(FieldId(id)))
    }
    pub(crate) fn nominal_field(
        &self,
        owner: DeclarationRef,
        ordinal: u32,
        steps: &mut usize,
    ) -> Result<Selection, &'static str> {
        if matches!(self.rows().header(owner)?, Some(s::HeaderRef::Variant(_))) {
            return self.variant_field(owner, ordinal, steps);
        }
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
        // Version thirteen: a tuple struct's fields are named by their
        // ordinals, which validation fixed against its constructor.
        let named = match found.name {
            Name::Named(name) => self.members.names.get(name) == field.name,
            Name::Index(index) => index == ordinal && s::positional(&field.name, ordinal as usize),
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
        if found.origin != actual.origin || found.ty != expected || !named || found.access != access
        {
            return Err(
                "source nominal field changed its current canonical type, name or access policy",
            );
        }
        Ok(Selection::Nominal(FieldId(id)))
    }
}
