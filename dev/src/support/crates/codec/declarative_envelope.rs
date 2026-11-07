//! Exported `macro_rules!` rows wrap an unchanged, independently bounded interface.
//! Canonical nesting is native links, then macros, then the core table.
use super::{Interface, MAGIC, Reader, SCHEMA, Writer};
use crate::support::{
    crates::exported_macros::{
        Delimiter, MAX_IMAGE_BYTES, MAX_MACROS, MAX_NAME_BYTES, MAX_ROWS, Macro, Macros, Row,
        TokenClass,
    },
    edition::Edition,
};
pub(super) const TAG: u8 = 14;
const VERSION: u8 = 1;
const GROUP: u8 = 0x80;
const EDITIONS: [Edition; 4] = [
    Edition::Rust2015,
    Edition::Rust2018,
    Edition::Rust2021,
    Edition::Rust2024,
];

pub(super) fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let macros = interface.macros().ok_or("missing exported macro table")?;
    let inner = super::encode_core(interface)?;
    let mut output = Writer::default();
    output.bytes(MAGIC)?;
    output.u16(SCHEMA)?;
    output.bytes(&[0, 0, TAG, VERSION])?;
    output.u32(macros.rows().len())?;
    for row in macros.rows() {
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
    }
    output.u32(inner.len())?;
    output.bytes(&inner)?;
    Ok(output.finish())
}
pub(super) fn decode(input: &mut Reader<'_>) -> Result<Interface, &'static str> {
    if input.u8()? != VERSION {
        return Err("unknown exported macro metadata version");
    }
    // Name, edition, flags, rule count, image and row-count fields.
    let count = input.count(MAX_MACROS, 18)?;
    let mut rows = Vec::with_capacity(count);
    for _ in 0..count {
        rows.push(decode_macro(input)?);
    }
    let macros = Macros::new(rows)?;
    let length = input.u32()?;
    let inner = input.bytes(length)?;
    if !input.done() || matches!(inner.get(12), Some(&(TAG | super::link_envelope::TAG))) {
        return Err("trailing or nested exported macro metadata");
    }
    super::decode(inner)?.with_macros(macros)
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
