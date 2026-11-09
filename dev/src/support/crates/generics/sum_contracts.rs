//! Version thirteen carries enum variant tables beside their nominal header.
//! The enum's `NominalHeader` keeps its contract, frame, Self and premises with
//! an empty field range; these columns own its variants, discriminants and
//! payload fields. Every consumer re-proves them before any storage exists.
#[cfg(test)]
#[path = "../../../../tests/invariants/braced_variant_rows.rs"]
mod record_tests;
use super::{
    Declaration, DefinitionKind, HeaderRef, NominalField, NominalHeader, Profile, SourceRows,
};
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
    /// Version thirteen: a record variant has no constructor. Its one
    /// declaration row names its type-namespace definition, and its fields
    /// carry their names.
    Record,
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
/// A payload field, positional unless its variant is a record variant. Enum
/// fields inherit the enum's visibility.
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
    /// Empty unless some variant is a record variant; then one name per
    /// field row, a record variant's field names and an empty name for every
    /// positional field. The wire carries only the record variants' names.
    pub names: Vec<String>,
}
impl EnumColumns {
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.headers.is_empty()
            && self.variants.is_empty()
            && self.fields.is_empty()
            && self.names.is_empty()
    }
    /// The field names of variant row `variant`: its own names if it is a
    /// record variant, otherwise `None`.
    /// # Errors
    /// Rejects a record variant whose fields have no names.
    pub fn record_names(&self, variant: &VariantHeader) -> Result<Option<&[String]>, &'static str> {
        if variant.shape != VariantShape::Record {
            return Ok(None);
        }
        variant
            .fields
            .slice(&self.names)
            .map(Some)
            .map_err(|_| "source record variant names have incomplete field coverage")
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
/// Version thirteen: a called tuple-variant or tuple-struct constructor. Its
/// function application names the variant declaration; its type formals are
/// its enum's or struct's and its signature is `fn(payload..) -> Self`, read
/// from these rows only.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Constructor {
    /// The enum's variant row, or `None` for a tuple struct, whose payload is
    /// its struct's own ordinal-named fields.
    pub variant: Option<VariantHeader>,
    /// The enum's or struct's nominal header row.
    pub nominal: u32,
}
/// A called constructor's positional payload rows.
#[derive(Clone, Copy)]
pub enum ConstructorPayload<'rows> {
    Variant(&'rows [VariantField]),
    Struct(&'rows [NominalField]),
}
impl ConstructorPayload<'_> {
    #[must_use]
    pub const fn len(self) -> usize {
        match self {
            Self::Variant(fields) => fields.len(),
            Self::Struct(fields) => fields.len(),
        }
    }
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }
    /// The payload field at `index`, in its owner's own frame.
    #[must_use]
    pub fn ty(self, index: usize) -> Option<TypeRef> {
        match self {
            Self::Variant(fields) => fields.get(index).map(|field| field.ty),
            Self::Struct(fields) => fields.get(index).map(|field| field.ty),
        }
    }
}
impl Constructor {
    /// The constructor a call target names, or `None` for any non-variant
    /// target. A unit variant or unit struct is a value, never a called
    /// constructor.
    /// # Errors
    /// Rejects unit constructors, unknown rows and a variant without its owner.
    pub fn of(
        (enums, declarations): (&EnumColumns, &[Declaration]),
        declaration: DeclarationRef,
        mut header: impl FnMut(DeclarationRef) -> Result<Option<HeaderRef>, &'static str>,
    ) -> Result<Option<Self>, &'static str> {
        // Packets without variant declarations pay nothing here.
        if declarations
            .get(declaration.0 as usize)
            .is_none_or(|row| row.definition.kind != DefinitionKind::Variant)
        {
            return Ok(None);
        }
        let index = match header(declaration)? {
            Some(HeaderRef::Variant(index)) => index,
            Some(HeaderRef::Constructor(nominal)) => {
                return Ok(Some(Self {
                    variant: None,
                    nominal,
                }));
            }
            _ => return Ok(None),
        };
        let variant = *enums
            .variants
            .get(index as usize)
            .ok_or("source constructor variant row is unknown")?;
        match variant.shape {
            VariantShape::Tuple => {}
            VariantShape::Unit => return Err("source unit variant has no called constructor"),
            VariantShape::Record => {
                return Err("source record variant has no called constructor");
            }
        }
        let owner = declarations
            .get(declaration.0 as usize)
            .and_then(|row| row.definition.parent)
            .ok_or("source constructor lost its actual enum")?;
        let Some(HeaderRef::Nominal(nominal)) = header(owner)? else {
            return Err("source constructor enum has no nominal header");
        };
        Ok(Some(Self {
            variant: Some(variant),
            nominal,
        }))
    }
    /// The constructor's payload: its variant's fields, or a tuple struct's
    /// own fields. A unit struct is a value, never a called constructor.
    /// # Errors
    /// Rejects foreign ranges and a called unit struct.
    pub fn payload<'rows>(
        &self,
        (enums, nominals, fields): (&'rows EnumColumns, &[NominalHeader], &'rows [NominalField]),
    ) -> Result<ConstructorPayload<'rows>, &'static str> {
        if let Some(variant) = self.variant {
            return Ok(ConstructorPayload::Variant(
                variant.fields.slice(&enums.fields)?,
            ));
        }
        let owner = nominals
            .get(self.nominal as usize)
            .ok_or("source struct constructor has no nominal header")?;
        let fields = owner.fields.slice(fields)?;
        if fields.is_empty() {
            return Err("source unit struct has no called constructor");
        }
        Ok(ConstructorPayload::Struct(fields))
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
    if !rows.enums().is_empty()
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
    let columns = names_coverage(rows.enums())?;
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
    let mut constructed = Vec::new();
    for (index, row) in rows.declarations.iter().enumerate() {
        work.charge(1)?;
        let named = match row.definition.kind {
            DefinitionKind::Variant if headers[index].is_none() => {
                let constructor = struct_constructor(rows, headers, row, &mut constructed, work)?;
                headers[index] = Some(constructor);
                true
            }
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
    positional_structs(rows, &constructed, work)
}
/// Version thirteen: a variant declaration parented by a struct is that tuple
/// or unit struct's constructor. Nothing about it travels but the row: its
/// shape and payload are its struct's own fields, named by their ordinals, and
/// its name is its struct's.
fn struct_constructor(
    rows: &SourceRows,
    headers: &[Option<HeaderRef>],
    row: &Declaration,
    constructed: &mut Vec<bool>,
    work: &mut Work,
) -> Result<HeaderRef, &'static str> {
    let parent = row
        .definition
        .parent
        .ok_or("source variant lost its actual parent")?;
    let owner = rows
        .declarations
        .get(parent.0 as usize)
        .ok_or("unknown source variant parent")?;
    if owner.definition.kind != DefinitionKind::Struct {
        return Err("source enum or variant declaration has no complete variant table");
    }
    let Some(Some(HeaderRef::Nominal(nominal))) = headers.get(parent.0 as usize).copied() else {
        return Err("source struct constructor lacks its struct's nominal header");
    };
    let fields = rows.nominals[nominal as usize].fields.slice(&rows.fields)?;
    work.charge(fields.len().saturating_add(1))?;
    if !fields
        .iter()
        .enumerate()
        .all(|(ordinal, field)| positional(&field.name, ordinal))
    {
        return Err("source record struct has no constructor");
    }
    if constructed.is_empty() {
        work.charge(rows.nominals.len())?;
        constructed.resize(rows.nominals.len(), false);
    }
    if std::mem::replace(&mut constructed[nominal as usize], true) {
        return Err("source struct has a second constructor");
    }
    if row.name.is_none() || row.name != owner.name {
        return Err("source struct constructor changed its struct's name");
    }
    Ok(HeaderRef::Constructor(nominal))
}
/// Only a constructed struct names its fields by their ordinals, and it
/// names every field that way.
fn positional_structs(
    rows: &SourceRows,
    constructed: &[bool],
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(rows.nominals.len().saturating_add(rows.fields.len()))?;
    for (index, header) in rows.nominals.iter().enumerate() {
        if constructed.get(index).copied().unwrap_or(false) {
            continue;
        }
        if header
            .fields
            .slice(&rows.fields)?
            .iter()
            .enumerate()
            .any(|(ordinal, field)| positional(&field.name, ordinal))
        {
            return Err("source tuple struct fields require its constructor");
        }
    }
    Ok(())
}
/// The canonical decimal spelling of a field's ordinal, which no record field
/// name can take.
#[must_use]
pub fn positional(name: &str, ordinal: usize) -> bool {
    let mut digits = [0_u8; 20];
    let mut cursor = digits.len();
    let mut value = ordinal;
    loop {
        cursor -= 1;
        digits[cursor] = b'0' + u8::try_from(value % 10).unwrap_or(0);
        value /= 10;
        if value == 0 {
            break;
        }
    }
    name.as_bytes() == &digits[cursor..]
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
    let fields = variant.fields.slice(&rows.enums().fields)?;
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
    field_names(&rows.enums().names, variant, work)
}
/// Names travel exactly when some variant is a record variant, and then for
/// every field row.
fn names_coverage(columns: &EnumColumns) -> Result<&EnumColumns, &'static str> {
    let records = columns
        .variants
        .iter()
        .any(|variant| variant.shape == VariantShape::Record);
    let expected = if records { columns.fields.len() } else { 0 };
    if columns.names.len() != expected {
        return Err("source record variant names have incomplete field coverage");
    }
    Ok(columns)
}
/// A record variant's fields are named as record fields are, once each; a
/// positional field carries no name.
fn field_names(
    names: &[String],
    variant: &VariantHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    if names.is_empty() {
        return Ok(());
    }
    let names = variant.fields.slice(names)?;
    if variant.shape != VariantShape::Record {
        return if names.iter().all(String::is_empty) {
            Ok(())
        } else {
            Err("source positional variant field carries a name")
        };
    }
    let mut seen = HashSet::default();
    for name in names {
        work.charge(name.len().saturating_add(1))?;
        crate::support::crates::identity::validate_component(name)?;
        if !seen.insert(name.as_str()) {
            return Err("source record variant has a duplicate field name");
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
