//! Named constant headers retain either checked bits or the actual associated
//! declaration's initializer ownership. Consumers still check original bodies.
use super::{ConstTerm, Declaration, DefinitionKind, HeaderIndex, HeaderRef, Profile, SourceRows};
use crate::support::crates::owner_catalog::{BodyRef, DeclarationRef};
#[path = "promoted_refs.rs"]
mod promoted;
use crate::support::crates::{
    Scalar, ScalarValue,
    generics::{Range, TypeRef, Work},
};
pub use promoted::PromotedHandler;

/// One constant's header row. Its declaration is implied: row `k` belongs to
/// the `k`-th constant declaration in declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantHeader {
    pub ty: TypeRef,
    pub body: ConstantBody,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstantBody {
    Required,
    Evaluated(ConstTerm),
    Original(BodyRef),
    /// A shared reference to a provider-owned promotion. The anchor defines
    /// the allocation; subsequent constants may refer to that same storage.
    PromotedReference {
        anchor: DeclarationRef,
        handlers: Range,
    },
}

impl ConstantBody {
    pub(crate) const fn evaluated(self) -> bool {
        matches!(self, Self::Evaluated(_) | Self::PromotedReference { .. })
    }
}

/// Each constant declaration with its row, in declaration order. Validated
/// rows pair every declaration; an unvalidated surplus is never yielded.
pub(super) fn declared<'rows>(
    declarations: &'rows [Declaration],
    constants: &'rows [ConstantHeader],
) -> impl Iterator<Item = (DeclarationRef, &'rows ConstantHeader)> {
    declarations
        .iter()
        .enumerate()
        .filter(|(_, row)| row.definition.kind == DefinitionKind::Constant)
        .zip(constants)
        .filter_map(|((index, _), header)| {
            u32::try_from(index)
                .ok()
                .map(|index| (DeclarationRef(index), header))
        })
}

/// Older profiles have no constant declaration or row; their readers reject
/// tag 13 with this same text.
pub(super) fn require_profile(
    rows: &SourceRows,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    if profile.body_nodes() {
        return Ok(());
    }
    work.charge(rows.declarations.len().saturating_add(1))?;
    if !rows.constants().is_empty()
        || rows
            .declarations
            .iter()
            .any(|row| row.definition.kind == DefinitionKind::Constant)
    {
        return Err("unknown source declaration kind");
    }
    Ok(())
}

/// Bind each constant declaration to its row in one pass. A constant
/// declaration without a row is left headerless, which the ledger rejects.
pub(super) fn headers(
    rows: &SourceRows,
    headers: &mut [Option<HeaderRef>],
    work: &mut Work,
) -> Result<(), &'static str> {
    if rows.constants().is_empty() {
        return Ok(());
    }
    work.charge(rows.declarations.len().saturating_add(1))?;
    let mut next = 0_u32;
    for (row, header) in rows.declarations.iter().zip(headers.iter_mut()) {
        if row.definition.kind != DefinitionKind::Constant {
            continue;
        }
        if header.replace(HeaderRef::Constant(next)).is_some() {
            return Err("source declaration has duplicate or conflicting typed headers");
        }
        next = next
            .checked_add(1)
            .ok_or("source constant header ordinal overflow")?;
    }
    if next as usize != rows.constants().len() {
        return Err("source constant rows lack their declaration");
    }
    Ok(())
}

/// A free constant has a module or function owner and a closed tag-0 value of
/// exactly its declared type, canonical for that type.
pub(super) fn validate(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    if rows.constants().is_empty() {
        return Ok(());
    }
    let [min, max, _] = super::interval_roles::bound_roles(rows, profile, work)?;
    let mut promoted_end = 0;
    for (index, row) in rows.declarations.iter().enumerate() {
        if row.definition.kind != DefinitionKind::Constant {
            continue;
        }
        let declaration = DeclarationRef(
            u32::try_from(index).map_err(|_| "source constant declaration overflow")?,
        );
        let Some(HeaderRef::Constant(ordinal)) = headers.header(declaration, work)? else {
            return Err("source declaration has no complete typed header");
        };
        let header = rows
            .constants()
            .get(ordinal as usize)
            .ok_or("source constant rows lack their declaration")?;
        if min == Some(declaration) || max == Some(declaration) {
            // The role validator authenticated both as required members.
            if header.body != ConstantBody::Required {
                return Err("source required range bound cannot carry an initializer");
            }
            if headers.scopes().scope_row(declaration, work)?.is_some() {
                return Err("source required range bound cannot carry its own scope");
            }
            continue;
        }
        if let ConstantBody::PromotedReference { anchor, handlers } = header.body {
            if !profile.declaration_policies()
                || headers.scopes().scope_row(declaration, work)?.is_some()
            {
                return Err("source promoted reference requires its closed v14 declaration");
            }
            if handlers.start != promoted_end || handlers.end < handlers.start {
                return Err("source promoted handler ranges must cover the targets in order");
            }
            promoted_end = handlers.end;
            value_owner(rows, headers, row, profile, work)?;
            promoted::reference(rows, headers, declaration, *header, anchor, handlers, work)?;
            continue;
        }
        let value = match header.body {
            ConstantBody::Required | ConstantBody::Original(_)
                if profile.declaration_policies() =>
            {
                associated_owner(rows, headers, row, *header, work)?;
                if headers.scopes().parent(declaration, work)? != row.definition.parent {
                    return Err(
                        "source associated constant changed its inherited declaration scope",
                    );
                }
                continue;
            }
            ConstantBody::Required => return Err("source free constant requires its value"),
            ConstantBody::Original(_) => {
                return Err("source constant original requires the version-fourteen profile");
            }
            ConstantBody::Evaluated(ConstTerm::Parameter(_)) => {
                return Err("source const value over a const parameter requires its wider profile");
            }
            ConstantBody::Evaluated(ConstTerm::Value(value)) => value,
            ConstantBody::PromotedReference { .. } => {
                return Err("source promoted reference lost its actual allocation");
            }
        };
        if profile.declaration_policies()
            && headers.scopes().scope_row(declaration, work)?.is_some()
        {
            return Err("source evaluated constant cannot supply a declaration scope");
        }
        value_owner(rows, headers, row, profile, work)?;
        if value.ty != header.ty {
            return Err("source constant value changed its declared type");
        }
        match rows.types.get(value.ty.0 as usize) {
            Some(super::Type::F16) if profile.declaration_policies() => {
                u16::try_from(value.bits).map_err(|_| "non-canonical source f16 const value")?;
            }
            Some(super::Type::F128) if profile.declaration_policies() => {}
            Some(&super::Type::Tuple(fields)) if profile.declaration_policies() => {
                validate_inline_tuple(rows, fields, value.bits, work)?;
            }
            _ => {
                super::const_value(&rows.types, value)?;
            }
        }
    }
    if promoted_end as usize != rows.cold().promoted_handlers.len() {
        return Err("source promoted handlers have no actual allocation owner");
    }
    Ok(())
}

pub(crate) fn inline_scalar_width(ty: Scalar, total: usize) -> Result<usize, &'static str> {
    let width = ty.bit_width().div_ceil(8) as usize;
    if width == 0 || total.saturating_add(width) > size_of::<u128>() {
        return Err("source tuple constant requires nonempty inline scalar fields");
    }
    Ok(total + width)
}

fn inline_fields<'a>(
    types: &'a [super::Type],
    roots: &'a [TypeRef],
    fields: Range,
) -> Result<impl Iterator<Item = Result<Scalar, &'static str>> + 'a, &'static str> {
    let fields = fields.slice(roots)?;
    if fields.is_empty() || fields.len() > size_of::<u128>() {
        return Err("source tuple constant requires a bounded nonempty field list");
    }
    Ok(fields
        .iter()
        .map(|field| match types.get(field.0 as usize) {
            Some(&super::Type::Scalar(ty)) => Ok(ty),
            _ => Err("source tuple constant requires actual scalar field types"),
        }))
}

pub(crate) fn inline_tuple_width(
    types: &[super::Type],
    roots: &[TypeRef],
    fields: Range,
) -> Result<usize, &'static str> {
    inline_fields(types, roots, fields)?.try_fold(0, |width, ty| inline_scalar_width(ty?, width))
}

fn validate_inline_tuple(
    rows: &SourceRows,
    fields: Range,
    bits: u128,
    work: &mut Work,
) -> Result<(), &'static str> {
    let bytes = bits.to_le_bytes();
    let mut data = bytes.as_slice();
    let mut width = 0;
    for ty in inline_fields(&rows.types, &rows.roots, fields)? {
        work.charge(1)?;
        let ty = ty?;
        let end = inline_scalar_width(ty, width)?;
        let bits = super::read_static_bits(&mut data, end - width)?;
        ScalarValue::new(ty, bits)?;
        width = end;
    }
    if data.iter().any(|&byte| byte != 0) {
        return Err("source tuple constant has nonzero unused payload bytes");
    }
    Ok(())
}

fn associated_owner(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    row: &Declaration,
    header: ConstantHeader,
    work: &mut Work,
) -> Result<(), &'static str> {
    let parent = row
        .definition
        .parent
        .ok_or("associated constant has no owner")?;
    match headers.header(parent, work)? {
        Some(HeaderRef::Trait(_)) => Ok(()),
        Some(HeaderRef::Implementation(index))
            if matches!(header.body, ConstantBody::Original(_)) =>
        {
            let implementation = &rows.implementations[index as usize];
            if implementation.trait_application.is_some()
                || valid_inherent_original(rows, headers, parent, implementation, header, work)?
            {
                Ok(())
            } else {
                Err("source associated constant changed its actual implementation owner or result")
            }
        }
        Some(HeaderRef::Function(_)) | None if header.body == ConstantBody::Required => {
            Err("source free constant requires its value")
        }
        _ => Err("source associated constant changed its actual trait or implementation owner"),
    }
}

fn valid_inherent_original(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    parent: DeclarationRef,
    implementation: &super::ImplHeader,
    header: ConstantHeader,
    work: &mut Work,
) -> Result<bool, &'static str> {
    if !implementation.inherent_constant_owner(&rows.types) {
        return Ok(false);
    }
    if headers.scopes().complete_slots(parent, work)? == 0 {
        return Ok(implementation.closed_inherent_constant_owner(&rows.types)
            && matches!(
                rows.types.get(header.ty.0 as usize),
                Some(super::Type::Nominal { .. })
            ));
    }
    for input in implementation.inputs.slice(&rows.clauses)? {
        work.charge(1)?;
        if input.binder.is_some()
            || matches!(input.requirement, super::Requirement::UnstableFeature(_))
        {
            return Ok(false);
        }
    }
    Ok(true)
}

/// An evaluated member belongs to its own closed inherent implementation.
fn value_owner(
    rows: &SourceRows,
    headers: &HeaderIndex<'_>,
    row: &Declaration,
    profile: Profile,
    work: &mut Work,
) -> Result<(), &'static str> {
    work.charge(1)?;
    let owner = row
        .definition
        .parent
        .ok_or("source constant header lost its free owner")?;
    match rows
        .declarations
        .get(owner.0 as usize)
        .map(|parent| parent.definition.kind)
    {
        Some(DefinitionKind::Module | DefinitionKind::Function) => return Ok(()),
        Some(DefinitionKind::SelfType) if profile.declaration_policies() => {
            if let Some(HeaderRef::Implementation(index)) = headers.header(owner, work)?
                && rows
                    .implementations
                    .get(index as usize)
                    .is_some_and(|implementation| {
                        implementation.closed_inherent_constant_owner(&rows.types)
                    })
                && headers.scopes().complete_slots(owner, work)? == 0
            {
                return Ok(());
            }
        }
        Some(DefinitionKind::Trait | DefinitionKind::SelfType) => {}
        _ => return Err("source constant header lost its free owner"),
    }
    Err("source associated constants require their wider profile")
}
