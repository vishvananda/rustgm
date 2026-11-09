//! Version thirteen: an enum's ordered variants, completed discriminants and
//! positional payloads follow its ordinary nominal header. No layout, niche or
//! tag decision travels; each consumer derives them from these source facts.
use super::{Context, DeclarationRef, Declarations, DefId, Report, Stage, g, s};
use crate::{
    sema::members::Name,
    support::crates::{Scalar, ScalarValue},
    syntax::item::Shape,
    types::{ItemId, Primitive},
};
impl Stage {
    /// Enum headers carry no callable inline policy; a `must_use` enum keeps
    /// its wider contract outside this profile.
    pub(super) fn enum_contract(
        &mut self,
        report: &Report,
        facts: &Declarations,
        owner: DefId,
        declaration: DeclarationRef,
    ) -> Result<s::HeaderContract, &'static str> {
        let key = report
            .resolution
            .definitions
            .get(owner.0 as usize)
            .and_then(|definition| definition.node)
            .ok_or("source enum lost its actual declaration")?;
        if facts
            .must_use_contract(report, owner)
            .map_err(|_| "source enum must_use contract is incomplete")?
            .is_some()
        {
            return Err("source enum must_use contracts require their wider profile");
        }
        Ok(s::HeaderContract {
            declaration,
            declaration_site: self.site(declaration, key)?,
            must_use: None,
            inline: None,
            cold: false,
            caller_tracking: false,
            target_features: None,
        })
    }
    /// Record one exported variant constructor's declaration row.
    pub(super) fn publish_variant(
        &mut self,
        facts: &Declarations,
        constructor: DefId,
        row: DeclarationRef,
    ) -> Result<(), &'static str> {
        let variant = facts
            .members
            .constructor(constructor)
            .ok_or("source variant lacks its actual constructor")?;
        let index = variant.0 as usize;
        if self.variants.len() <= index {
            let growth = index.saturating_add(1).saturating_sub(self.variants.len());
            self.reserve(growth.saturating_mul(size_of::<Option<DeclarationRef>>()))?;
            self.variants.resize(index.saturating_add(1), None);
        }
        if self.variants[index].replace(row).is_some() {
            return Err("source variant has two exported constructor declarations");
        }
        Ok(())
    }
    pub(super) fn enum_header(
        &mut self,
        facts: &Declarations,
        owner: DefId,
        declaration: DeclarationRef,
        context: Context,
    ) -> Result<(), &'static str> {
        let item = ItemId(owner.0);
        let shape = facts
            .aggregates
            .get(item)
            .ok_or("source enum lacks its completed shape")?;
        let tag = shape.repr.tag.map(integer_scalar).transpose()?;
        let completed = facts.aggregates.variants(shape);
        let variants = facts.members.variants(item);
        if variants.is_empty() || completed.len() != variants.len() {
            return Err("source enum without variants is outside the initial profile");
        }
        // Version thirteen: an enum's lifetimes are its leading region slots
        // (`scoped_parameters`), named by its variant fields in its own frame.
        let start = count(self.rows.enums().variants.len())?;
        for (ordinal, (&id, completed)) in (0_u32..).zip(variants.iter().zip(completed)) {
            self.charge(1)?;
            let variant = facts.members.variants[id.0 as usize];
            let shape = match variant.shape {
                Shape::Unit => s::VariantShape::Unit,
                Shape::Tuple => s::VariantShape::Tuple,
                Shape::Record => return Err("source record variants require their wider profile"),
            };
            ScalarValue::new(tag.unwrap_or(Scalar::Isize), completed.discriminant)
                .map_err(|_| "source enum negative discriminants require their wider profile")?;
            let row = self.variant(id)?;
            let fields = facts
                .members
                .fields(id)
                .ok_or("source variant field group is unknown")?;
            let field_start = count(self.rows.enums().fields.len())?;
            for (index, field) in (0_u32..).zip(fields) {
                self.charge(1)?;
                if field.name != Name::Index(index) {
                    return Err("source variant field changed its position");
                }
                let source = field
                    .origin
                    .source()
                    .ok_or("source variant field has no original source identity")?;
                let ty = self.ty(facts, field.ty, context)?;
                let site = self.site(row, source)?;
                self.reserve(4 * size_of::<s::VariantField>())?;
                self.cold()?.enums.fields.push(s::VariantField {
                    owner: row,
                    ordinal: index,
                    ty,
                    site,
                });
            }
            let fields = g::Range {
                start: field_start,
                end: count(self.rows.enums().fields.len())?,
            };
            self.reserve(4 * size_of::<s::VariantHeader>())?;
            self.cold()?.enums.variants.push(s::VariantHeader {
                declaration: row,
                ordinal,
                shape,
                discriminant: completed.discriminant,
                fields,
            });
        }
        self.reserve(4 * size_of::<s::EnumHeader>())?;
        let end = count(self.rows.enums().variants.len())?;
        self.cold()?.enums.headers.push(s::EnumHeader {
            declaration,
            tag,
            variants: g::Range { start, end },
        });
        Ok(())
    }
}
fn count(length: usize) -> Result<u32, &'static str> {
    u32::try_from(length).map_err(|_| "source enum table overflow")
}
fn integer_scalar(tag: Primitive) -> Result<Scalar, &'static str> {
    Ok(match tag {
        Primitive::U8 => Scalar::U8,
        Primitive::U16 => Scalar::U16,
        Primitive::U32 => Scalar::U32,
        Primitive::U64 => Scalar::U64,
        Primitive::U128 => Scalar::U128,
        Primitive::Usize => Scalar::Usize,
        Primitive::I8 => Scalar::I8,
        Primitive::I16 => Scalar::I16,
        Primitive::I32 => Scalar::I32,
        Primitive::I64 => Scalar::I64,
        Primitive::I128 => Scalar::I128,
        Primitive::Isize => Scalar::Isize,
        _ => return Err("source enum representation tag is not an integer type"),
    })
}
