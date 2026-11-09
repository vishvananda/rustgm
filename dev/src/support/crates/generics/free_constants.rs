//! Version thirteen carries one header row per named constant: its type and
//! its completed value. Every consumer re-proves the value's canonical bits for
//! that type before seeding it as a completed definition; no consumer reads an
//! initializer.
use super::{ConstTerm, Declaration, DefinitionKind, HeaderIndex, HeaderRef, Profile, SourceRows};
use crate::support::crates::generics::{TypeRef, Work};
use crate::support::crates::owner_catalog::DeclarationRef;

/// One constant's header row. Its declaration is implied: row `k` belongs to
/// the `k`-th constant declaration in declaration order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConstantHeader {
    pub ty: TypeRef,
    /// The completed value, written in place with the const-term codec.
    /// `None` is a required trait const's, which no free const may be.
    pub value: Option<ConstTerm>,
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
    work: &mut Work,
) -> Result<(), &'static str> {
    if rows.constants().is_empty() {
        return Ok(());
    }
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
        free_owner(rows, row, work)?;
        let value = match header.value {
            None => return Err("source free constant requires its value"),
            Some(ConstTerm::Parameter(_)) => {
                return Err("source const value over a const parameter requires its wider profile");
            }
            Some(ConstTerm::Value(value)) => value,
        };
        if value.ty != header.ty {
            return Err("source constant value changed its declared type");
        }
        super::const_value(&rows.types, value)?;
    }
    Ok(())
}

/// A trait or impl member is an associated constant, the next profile's.
fn free_owner(rows: &SourceRows, row: &Declaration, work: &mut Work) -> Result<(), &'static str> {
    work.charge(1)?;
    let parent = row
        .definition
        .parent
        .and_then(|parent| rows.declarations.get(parent.0 as usize))
        .map(|parent| parent.definition.kind);
    match parent {
        Some(DefinitionKind::Module | DefinitionKind::Function) => Ok(()),
        Some(DefinitionKind::SelfType | DefinitionKind::Trait) => {
            Err("source associated constants require their wider profile")
        }
        _ => Err("source constant header lost its free owner"),
    }
}
