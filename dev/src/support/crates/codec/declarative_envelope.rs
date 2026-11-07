//! Exported macro rows wrap an unchanged, independently bounded interface.
//! Canonical nesting is native links, then macros, then the core table.
//!
//! Version one holds only declarative root rows. Version two prefixes every
//! row with a kind byte and its module placement, and adds builtin rows,
//! re-export rows and typed internal-unstable feature lists. A table that needs
//! none of them keeps its exact version-one bytes, and a version-two table must
//! need one of them.
//!
//! Version three keeps the version-two layout and appends to each definition
//! row its `#[unstable]` feature (a tag byte, then the feature when the tag is
//! 1), and to each builtin row, before that, its allowance list. Only a table
//! with such a feature or a builtin allowance list writes version three.
use super::{Interface, MAGIC, Reader, SCHEMA, Writer};
use crate::support::{
    crates::exported_macros::{
        BuiltinMacro, Delimiter, MAX_FEATURES, MAX_IDENTITY_BYTES, MAX_IMAGE_BYTES, MAX_MACROS,
        MAX_NAME_BYTES, MAX_ROWS, Macro, MacroKind, Macros, ReexportMacro, Row, Syntax, TokenClass,
    },
    edition::Edition,
};
pub(super) const TAG: u8 = 14;
const VERSION_ONE: u8 = 1;
const VERSION_TWO: u8 = 2;
const VERSION_THREE: u8 = 3;
const GROUP: u8 = 0x80;
const DECLARATIVE: u8 = 0;
const BUILTIN: u8 = 1;
const REEXPORT: u8 = 2;
const EDITIONS: [Edition; 4] = [
    Edition::Rust2015,
    Edition::Rust2018,
    Edition::Rust2021,
    Edition::Rust2024,
];

/// One row of the merged, canonically ordered version-two table.
enum Entry<'a> {
    Declarative(&'a Macro),
    Builtin(&'a BuiltinMacro),
    Reexport(&'a ReexportMacro),
}
impl Entry<'_> {
    fn key(&self) -> (&[String], &str) {
        match self {
            Self::Declarative(row) => (&[], row.name()),
            Self::Builtin(row) => (row.placement(), row.name()),
            Self::Reexport(row) => (row.placement(), row.name()),
        }
    }
}

/// Every row of the table, ordered by its unique (placement, name) key.
fn merged(macros: &Macros) -> Vec<Entry<'_>> {
    let mut entries: Vec<_> = macros
        .rows()
        .iter()
        .map(Entry::Declarative)
        .chain(macros.builtins().iter().map(Entry::Builtin))
        .chain(macros.reexports().iter().map(Entry::Reexport))
        .collect();
    entries.sort_unstable_by(|left, right| left.key().cmp(&right.key()));
    entries
}

pub(super) fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let macros = interface.macros().ok_or("missing exported macro table")?;
    let inner = super::encode_core(interface)?;
    let mut output = Writer::default();
    output.bytes(MAGIC)?;
    output.u16(SCHEMA)?;
    let version = macros.version();
    if version > VERSION_ONE {
        let three = version == VERSION_THREE;
        output.bytes(&[0, 0, TAG, version])?;
        let entries = merged(macros);
        output.u32(entries.len())?;
        for entry in entries {
            match entry {
                Entry::Declarative(row) => {
                    output.u8(DECLARATIVE)?;
                    output.path(&[])?;
                    encode_macro(&mut output, row)?;
                    names(&mut output, row.internal_unstable())?;
                    if three {
                        unstable(&mut output, row.unstable())?;
                    }
                }
                Entry::Builtin(row) => {
                    encode_builtin(&mut output, row)?;
                    if three {
                        names(&mut output, row.internal_unstable())?;
                        unstable(&mut output, row.unstable())?;
                    }
                }
                Entry::Reexport(row) => {
                    output.u8(REEXPORT)?;
                    output.path(row.placement())?;
                    output.string(row.name())?;
                    output.u32(row.owner() as usize)?;
                    output.path(row.target_placement())?;
                    output.string(row.target_name())?;
                }
            }
        }
    } else {
        output.bytes(&[0, 0, TAG, VERSION_ONE])?;
        output.u32(macros.rows().len())?;
        for row in macros.rows() {
            encode_macro(&mut output, row)?;
        }
    }
    output.u32(inner.len())?;
    output.bytes(&inner)?;
    Ok(output.finish())
}
fn encode_macro(output: &mut Writer, row: &Macro) -> Result<(), &'static str> {
    output.string(row.name())?;
    output.u8(row.edition() as u8)?;
    output.u8(u8::from(row.local_inner_macros()))?;
    output.u32(row.rules() as usize)?;
    output.string(row.image())?;
    output.u32(row.rows().len())?;
    for tree in row.rows() {
        let (code, value) = match *tree {
            Row::Token { class, length } => (class as u8, length),
            Row::Group { delimiter, rows } => (GROUP | delimiter as u8, rows),
        };
        output.u8(code)?;
        output.u32(value as usize)?;
    }
    Ok(())
}
fn encode_builtin(output: &mut Writer, row: &BuiltinMacro) -> Result<(), &'static str> {
    output.u8(BUILTIN)?;
    output.path(row.placement())?;
    output.string(row.name())?;
    output.u8(row.syntax() as u8)?;
    output.string(row.identity())?;
    output.u8(row.kind() as u8)?;
    names(output, row.helpers())
}
fn unstable(output: &mut Writer, feature: Option<&str>) -> Result<(), &'static str> {
    match feature {
        None => output.u8(0),
        Some(feature) => {
            output.u8(1)?;
            output.string(feature)
        }
    }
}
fn names(output: &mut Writer, names: &[String]) -> Result<(), &'static str> {
    output.u32(names.len())?;
    for name in names {
        output.string(name)?;
    }
    Ok(())
}
pub(super) fn decode(input: &mut Reader<'_>) -> Result<Interface, &'static str> {
    let macros = match input.u8()? {
        VERSION_ONE => {
            // Name, edition, flags, rule count, image and row-count fields.
            let count = input.count(MAX_MACROS, 18)?;
            let mut rows = Vec::with_capacity(count);
            for _ in 0..count {
                rows.push(decode_macro(input)?);
            }
            Macros::new(rows)?
        }
        version @ (VERSION_TWO | VERSION_THREE) => decode_placed(input, version)?,
        _ => return Err("unknown exported macro metadata version"),
    };
    let length = input.u32()?;
    let inner = input.bytes(length)?;
    if !input.done() || matches!(inner.get(12), Some(&(TAG | super::link_envelope::TAG))) {
        return Err("trailing or nested exported macro metadata");
    }
    super::decode(inner)?.with_macros(macros)
}
fn decode_placed(input: &mut Reader<'_>, version: u8) -> Result<Macros, &'static str> {
    let three = version == VERSION_THREE;
    // Kind, placement count and name length.
    let count = input.count(MAX_MACROS, 7)?;
    let (mut rows, mut builtins, mut reexports) = (Vec::new(), Vec::new(), Vec::new());
    let mut previous: Option<(Vec<String>, String)> = None;
    for _ in 0..count {
        let kind = input.u8()?;
        let placement = input.module_path()?;
        let key = match kind {
            DECLARATIVE => {
                if !placement.is_empty() {
                    return Err("declarative exported macros belong to the crate root");
                }
                let mut row = decode_macro(input)?;
                row = row.with_internal_unstable(decode_names(input)?)?;
                if three {
                    row = row.with_unstable(decode_unstable(input)?)?;
                }
                let name = row.name().to_owned();
                rows.push(row);
                (placement, name)
            }
            BUILTIN => {
                let mut row = decode_builtin(input, placement)?;
                if three {
                    row = row
                        .with_internal_unstable(decode_names(input)?)?
                        .with_unstable(decode_unstable(input)?)?;
                }
                let key = (row.placement().to_vec(), row.name().to_owned());
                builtins.push(row);
                key
            }
            REEXPORT => {
                let name = input.string(MAX_NAME_BYTES)?;
                let owner =
                    u32::try_from(input.u32()?).map_err(|_| "invalid exported macro owner")?;
                let target_placement = input.module_path()?;
                let target_name = input.string(MAX_NAME_BYTES)?;
                let row =
                    ReexportMacro::new(placement, name, owner, target_placement, target_name)?;
                let key = (row.placement().to_vec(), row.name().to_owned());
                reexports.push(row);
                key
            }
            _ => return Err("unknown exported macro row kind"),
        };
        if previous.as_ref().is_some_and(|previous| *previous >= key) {
            return Err("exported macros must be unique and sorted by placement and name");
        }
        previous = Some(key);
    }
    let macros = Macros::with_reexports(rows, builtins, reexports)?;
    if macros.version() != version {
        return Err(if three {
            "version-three exported macro table has only version-two rows"
        } else {
            "version-two exported macro table has only version-one rows"
        });
    }
    Ok(macros)
}
fn decode_unstable(input: &mut Reader<'_>) -> Result<Option<String>, &'static str> {
    match input.u8()? {
        0 => Ok(None),
        1 => input.string(MAX_NAME_BYTES).map(Some),
        _ => Err("unknown exported macro stability"),
    }
}
fn decode_builtin(
    input: &mut Reader<'_>,
    placement: Vec<String>,
) -> Result<BuiltinMacro, &'static str> {
    let name = input.string(MAX_NAME_BYTES)?;
    let syntax = *Syntax::ALL
        .get(usize::from(input.u8()?))
        .ok_or("unknown exported builtin macro syntax")?;
    let identity = input.string(MAX_IDENTITY_BYTES)?;
    let kind = *MacroKind::ALL
        .get(usize::from(input.u8()?))
        .ok_or("unknown exported builtin macro kind")?;
    let helpers = decode_names(input)?;
    BuiltinMacro::new(placement, name, syntax, identity, kind, helpers)
}
fn decode_names(input: &mut Reader<'_>) -> Result<Vec<String>, &'static str> {
    let count = input.count(MAX_FEATURES, 4)?;
    let mut names = Vec::with_capacity(count);
    for _ in 0..count {
        names.push(input.string(MAX_NAME_BYTES)?);
    }
    Ok(names)
}
fn decode_macro(input: &mut Reader<'_>) -> Result<Macro, &'static str> {
    let name = input.string(MAX_NAME_BYTES)?;
    let edition = *EDITIONS
        .get(usize::from(input.u8()?))
        .ok_or("unknown exported macro edition")?;
    let local_inner_macros = match input.u8()? {
        0 => false,
        1 => true,
        _ => return Err("unknown exported macro flags"),
    };
    let rules = u32::try_from(input.u32()?).map_err(|_| "invalid exported macro rule count")?;
    let length = input.u32()?;
    if length > MAX_IMAGE_BYTES {
        return Err("exported macro body exceeds the token metadata budget");
    }
    let image = std::str::from_utf8(input.bytes(length)?)
        .map_err(|_| "exported macro image is not UTF-8")?
        .to_owned();
    let count = input.count(MAX_ROWS, 5)?;
    let mut trees = Vec::with_capacity(count);
    for _ in 0..count {
        let code = input.u8()?;
        let value = u32::try_from(input.u32()?).map_err(|_| "invalid exported macro row")?;
        trees.push(if code & GROUP == 0 {
            Row::Token {
                class: *TokenClass::ALL
                    .get(usize::from(code))
                    .ok_or("unknown exported macro token class")?,
                length: value,
            }
        } else {
            Row::Group {
                delimiter: *Delimiter::ALL
                    .get(usize::from(code & !GROUP))
                    .ok_or("unknown exported macro delimiter")?,
                rows: value,
            }
        });
    }
    Macro::new(name, edition, local_inner_macros, rules, image, trees)
}
