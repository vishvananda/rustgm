//! Version thirteen carries enum variant tables beside their nominal header.
//! The enum's `NominalHeader` keeps its contract, frame, Self and premises with
//! an empty field range; these columns own its variants, discriminants and
//! payload fields. Every consumer re-proves them before any storage exists.
use super::{DefinitionKind, HeaderRef, Profile, SourceRows};
use crate::support::{
    crates::{
        RecordRepr, Scalar, ScalarValue,
        generics::{Range, TypeRef, Work},
        owner_catalog::DeclarationRef,
    },
    hash::HashSet,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EnumHeader {
    pub declaration: DeclarationRef,
    /// An explicit integer `repr`; `None` is the default representation, whose
    /// discriminants are `isize` values.
    pub tag: Option<Scalar>,
    pub variants: Range,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VariantShape {
    Unit,
    Tuple,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VariantHeader {
    pub declaration: DeclarationRef,
    pub ordinal: u32,
    pub shape: VariantShape,
    /// The completed discriminant: two's-complement bits of its tag width.
    pub discriminant: u128,
    pub fields: Range,
}
/// A positional payload field. Enum fields inherit the enum's visibility.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VariantField {
    pub owner: DeclarationRef,
    pub ordinal: u32,
    pub ty: TypeRef,
    pub site: u32,
}
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnumColumns {
    pub headers: Vec<EnumHeader>,
    pub variants: Vec<VariantHeader>,
    pub fields: Vec<VariantField>,
}
impl EnumColumns {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty() && self.variants.is_empty() && self.fields.is_empty()
    }
    /// The enum header owning variant row `index`. Validated headers own
    /// contiguous ascending variant ranges, so this is one binary search.
    /// # Errors
    /// Rejects rows outside every validated variant table.
    pub fn owner(&self, index: u32) -> Result<&EnumHeader, &'static str> {
        let position = self
            .headers
            .partition_point(|header| header.variants.end <= index);
        self.headers
            .get(position)
            .filter(|header| header.variants.start <= index && index < header.variants.end)
            .ok_or("source variant has no validated enum header")
    }
}
impl EnumHeader {
    /// The discriminant type: the explicit integer tag, or `isize`.
    #[must_use]
    pub fn discriminant_type(self) -> Scalar {
        self.tag.unwrap_or(Scalar::Isize)
    }
}

/// Enum declarations, variant declarations and their columns exist only under
/// the version-thirteen body IR profile.
pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    if profile.body_nodes() {
        return Ok(());
    }
    work.charge(rows.declarations.len().saturating_add(1))?;
    if !rows.enums.is_empty()
        || rows.declarations.iter().any(|row| {
            matches!(
                row.definition.kind,
                DefinitionKind::Enum | DefinitionKind::Variant
            )
        })
    {
        return Err("source enum headers require the version-thirteen body IR profile");
    }
    Ok(())
}

/// Bind every variant declaration to its row after the nominal headers.
pub(super) fn headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    work: &mut Work,
) -> Result<(), &'static str> {
    let columns = &rows.enums;
    work.charge(
        columns
            .headers
            .len()
            .saturating_add(columns.variants.len())
            .saturating_add(columns.fields.len())
            .saturating_add(1),
    )?;
    let (mut variants, mut fields) = (0, 0);
    let mut previous = None;
    for header in &columns.headers {
        work.charge(1)?;
        if previous.is_some_and(|last: DeclarationRef| last.0 >= header.declaration.0)
            || header.variants.start != variants
            || header.variants.end < variants
        {
            return Err("source enum headers changed their declaration order or variant range");
        }
        previous = Some(header.declaration);
        nominal(rows, headers, header)?;
        if header.tag.is_some_and(|tag| !integer(tag)) {
            return Err("source enum representation tag is not an integer type");
        }
        let mut names = HashSet::default();
        let mut discriminants = HashSet::default();
        let table = header.variants.slice(&columns.variants)?;
        if table.is_empty() {
            return Err("source enum without variants is outside the initial profile");
        }
        let data = table
            .iter()
            .any(|variant| variant.fields.start != variant.fields.end);
        for (ordinal, variant) in table.iter().enumerate() {
            work.charge(1)?;
            let index = header.variants.start as usize + ordinal;
            let declaration = rows
                .declarations
                .get(variant.declaration.0 as usize)
                .ok_or("unknown source variant declaration")?;
            let cell = headers
                .get_mut(variant.declaration.0 as usize)
                .ok_or("unknown source variant header")?;
            if declaration.definition.kind != DefinitionKind::Variant
                || declaration.definition.parent != Some(header.declaration)
                || variant.ordinal as usize != ordinal
                || cell.is_some()
                || variant.fields.start != fields
                || variant.fields.end < fields
            {
                return Err("source variant changed its actual enum owner, ordinal or fields");
            }
            let name = declaration
                .name
                .as_deref()
                .ok_or("source variant lost its actual name")?;
            work.charge(name.len())?;
            if !names.insert(name) {
                return Err("source enum has a duplicate variant name");
            }
            ScalarValue::new(header.discriminant_type(), variant.discriminant)
                .map_err(|_| "source enum discriminant exceeds its representation")?;
            // Explicit discriminants on data variants require an integer tag,
            // so an untagged data enum keeps its implicit ordinal values.
            if header.tag.is_none() && data && variant.discriminant != ordinal as u128 {
                return Err("source data enum discriminants require an integer representation");
            }
            if !discriminants.insert(variant.discriminant) {
                return Err("source enum has a duplicate discriminant");
            }
            payload(rows, variant, declaration.sites, work)?;
            *cell = Some(HeaderRef::Variant(
                u32::try_from(index).map_err(|_| "source variant ordinal overflow")?,
            ));
            fields = variant.fields.end;
        }
        variants = header.variants.end;
    }
    if variants as usize != columns.variants.len() || fields as usize != columns.fields.len() {
        return Err("source enum variants or fields have incomplete owner coverage");
    }
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let named = match row.definition.kind {
            DefinitionKind::Variant => matches!(headers[index], Some(HeaderRef::Variant(_))),
            // Headers are in ascending declaration order.
            DefinitionKind::Enum => columns
                .headers
                .binary_search_by_key(&index, |header| header.declaration.0 as usize)
                .is_ok(),
            _ => true,
        };
        if !named {
            return Err("source enum or variant declaration has no complete variant table");
        }
    }
    Ok(())
}
/// The shared nominal row: an empty field range and the default record repr.
fn nominal(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    header: &EnumHeader,
) -> Result<(), &'static str> {
    let declaration = rows
        .declarations
        .get(header.declaration.0 as usize)
        .ok_or("unknown source enum declaration")?;
    let Some(Some(HeaderRef::Nominal(index))) = headers.get(header.declaration.0 as usize) else {
        return Err("source enum lacks its actual nominal header");
    };
    let row = &rows.nominals[*index as usize];
    if declaration.definition.kind != DefinitionKind::Enum
        || row.fields.start != row.fields.end
        || row.repr != RecordRepr::Rust
    {
        return Err("source enum nominal header carries record fields or representation");
    }
    Ok(())
}
fn payload(
    rows: &SourceRows,
    variant: &VariantHeader,
    sites: u32,
    work: &mut Work,
) -> Result<(), &'static str> {
    let fields = variant.fields.slice(&rows.enums.fields)?;
    if variant.shape == VariantShape::Unit && !fields.is_empty() {
        return Err("source unit variant carries payload fields");
    }
    for (ordinal, field) in fields.iter().enumerate() {
        work.charge(1)?;
        if field.owner != variant.declaration
            || field.ordinal as usize != ordinal
            || field.site >= sites
            || field.ty.0 as usize >= rows.types.len()
        {
            return Err("source variant field changed its owner, ordinal, type or site");
        }
    }
    Ok(())
}
const fn integer(tag: Scalar) -> bool {
    !matches!(
        tag,
        Scalar::Unit | Scalar::Bool | Scalar::Char | Scalar::F32 | Scalar::F64
    )
}
/// The nominal declaration kinds whose header is a `NominalHeader`.
#[must_use]
pub const fn nominal_kind(kind: DefinitionKind) -> bool {
    matches!(kind, DefinitionKind::Struct | DefinitionKind::Enum)
}
