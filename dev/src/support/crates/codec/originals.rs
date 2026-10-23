//! Envelope 9 versions 0/1: explicit Rust 2024 legacy-role operation data.
mod applications;
mod bodies;
mod opcodes;
mod tables;
use super::{Interface, Reader, Writer};
use crate::support::{
    crates::{
        self, ExternalDef, ExternalKind, generics as g,
        owner_catalog::{BodyRef, DeclarationRef},
    },
    edition::Edition,
};

pub(super) fn encode(out: &mut Writer, payload: &g::Payload) -> Result<(), &'static str> {
    if payload.rows().edition != Edition::Rust2024 {
        return Err("generic row version zero requires defining edition2024");
    }
    // These descriptor IDs belong to envelope9, not to the old language header.
    let version = u8::from(payload.rows().calls.is_some());
    out.bytes(&[version, 0, 0])?; // Row version, Rust2024, legacy language-role policy.
    tables::encode(out, payload.rows())?;
    if let Some(calls) = &payload.rows().calls {
        applications::encode(out, calls)?;
    }
    out.u32(payload.rows().originals.len())?;
    for original in &payload.rows().originals {
        bodies::encode(out, original, version)?;
    }
    Ok(())
}
pub(super) fn decode(input: &mut Reader<'_>, base: &Interface) -> Result<g::Payload, &'static str> {
    let descriptor = input.bytes(3)?;
    let version = descriptor[0];
    if version > 1 || descriptor[1..] != [0, 0] {
        return Err("unsupported generic row version, defining edition or source-role policy");
    }
    let mut read = Input {
        reader: input,
        bytes: size_of::<g::Payload>(),
        base: base.base_retained_bytes(),
        version,
    };
    let mut rows = tables::decode(&mut read, base.identity())?;
    if version == 1 {
        rows.calls = Some(Box::new(applications::decode(&mut read)?));
    }
    let count = read.count::<g::Original>(crates::MAX_FUNCTIONS, 48)?;
    rows.originals = Vec::with_capacity(count);
    for _ in 0..count {
        rows.originals.push(bodies::decode(&mut read)?);
    }
    let before = read.reader.used_atoms();
    let mut work = g::Work {
        steps: before,
        ..g::Work::default()
    };
    let payload = g::Payload::with_base(base.identity(), rows, &mut work, read.base)?;
    read.reader.atoms(work.steps.saturating_sub(before))?;
    Ok(payload)
}
struct Input<'a, 'b> {
    reader: &'b mut Reader<'a>,
    bytes: usize,
    base: usize,
    version: u8,
}
impl Input<'_, '_> {
    fn reserve(&mut self, bytes: usize) -> Result<(), &'static str> {
        self.bytes = self.bytes.saturating_add(bytes);
        if self.base.saturating_add(self.bytes) > crates::MAX_BYTES {
            return Err("generic decoded table storage exceeds combined budget");
        }
        Ok(())
    }
    fn count<T>(&mut self, limit: usize, minimum: usize) -> Result<usize, &'static str> {
        let count = self.reader.count(limit, minimum)?;
        self.reader.atoms(count)?;
        self.reserve(count.saturating_mul(size_of::<T>()))?;
        Ok(count)
    }
    fn fixed<T>(&mut self, count: usize, minimum: usize) -> Result<(), &'static str> {
        if count > self.reader.remaining() / minimum {
            return Err("truncated generic metadata column");
        }
        self.reader.atoms(count)?;
        self.reserve(count.saturating_mul(size_of::<T>()))
    }
    fn id(&mut self) -> Result<u32, &'static str> {
        u32::try_from(self.reader.u32()?).map_err(|_| "generic index exceeds u32")
    }
    fn bool(&mut self) -> Result<bool, &'static str> {
        match self.reader.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err("invalid generic boolean"),
        }
    }
    fn option(&mut self) -> Result<Option<u32>, &'static str> {
        match self.reader.u8()? {
            0 => Ok(None),
            1 => self.id().map(Some),
            _ => Err("invalid generic optional index"),
        }
    }
    fn range(&mut self) -> Result<g::Range, &'static str> {
        Ok(g::Range {
            start: self.id()?,
            end: self.id()?,
        })
    }
    fn string(&mut self) -> Result<String, &'static str> {
        let value = self.reader.text(256)?;
        self.reserve(value.len())?;
        Ok(value.to_owned())
    }
    fn path(&mut self) -> Result<Vec<String>, &'static str> {
        let count = usize::from(self.reader.u16()?);
        if count == 0 || count > 64 {
            return Err("invalid generic definition path length");
        }
        self.reader.atoms(count)?;
        self.reserve(count.saturating_mul(size_of::<String>()))?;
        let mut path = Vec::with_capacity(count);
        for _ in 0..count {
            path.push(self.string()?);
        }
        Ok(path)
    }
}
fn id(out: &mut Writer, value: u32) -> Result<(), &'static str> {
    out.u32(value as usize)
}
fn option(out: &mut Writer, value: Option<u32>) -> Result<(), &'static str> {
    match value {
        None => out.u8(0),
        Some(value) => {
            out.u8(1)?;
            id(out, value)
        }
    }
}
fn range(out: &mut Writer, value: g::Range) -> Result<(), &'static str> {
    id(out, value.start)?;
    id(out, value.end)
}
