//! Standalone source tables never nest inside an earlier scalar/generic envelope.
use super::{
    Interface, MAGIC, PROFILE, Reader, SCHEMA, Writer, decode_dependencies, ordinary_columns,
};
use crate::support::{
    crates::{
        MAX_BYTES,
        generics::{Payload, Work, source::Profile},
    },
    edition::Edition,
};
pub(super) const TAG: u8 = 10;
const ROW_VERSION: u16 = 0;
const DEFINING_RUST2024: u8 = 0;

pub(super) fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let payload = interface
        .generic_payload()
        .ok_or("source envelope lacks its complete payload")?;
    let descriptor = interface
        .source_descriptor()
        .ok_or("source envelope lacks its actual descriptor")?;
    if descriptor.profile() != Profile::Source2024Functions
        || descriptor.edition() != Edition::Rust2024
        || descriptor.identity() != interface.identity()
    {
        return Err("source envelope changed its actual defining profile/identity");
    }
    let mut out = Writer::source(
        interface
            .base_retained_bytes()
            .saturating_add(payload.retained_bytes()),
    );
    out.bytes(MAGIC)?;
    out.u16(SCHEMA)?;
    out.bytes(&PROFILE[..2])?;
    out.u8(TAG)?;
    out.u16(ROW_VERSION)?;
    out.u8(DEFINING_RUST2024)?;
    out.identity(interface.identity())?;
    out.u32(interface.dependencies().len())?;
    for dependency in interface.dependencies() {
        out.identity(dependency)?;
    }
    ordinary_columns::encode(&mut out, descriptor.rows())?;
    Ok(out.finish())
}
pub(super) fn decode(input: &mut Reader<'_>) -> Result<Interface, &'static str> {
    if input.u16()? != ROW_VERSION || input.u8()? != DEFINING_RUST2024 {
        return Err("unsupported source row version or defining edition");
    }
    let identity = input.identity()?;
    let dependencies = decode_dependencies(input)?;
    let interface = Interface::new(identity, dependencies, Vec::new())?;
    let base = interface.base_retained_bytes();
    if base > MAX_BYTES {
        return Err("source envelope base exceeds retained storage budget");
    }
    let rows = ordinary_columns::decode(input, base)?;
    if !input.done() {
        return Err("trailing source metadata bytes");
    }
    let mut work = Work {
        steps: input.used_atoms(),
        ..Work::default()
    };
    let payload = Payload::from_source(interface.identity(), rows, base, &mut work)?;
    interface.with_generic_payload(payload)
}

#[cfg(test)]
#[path = "../../../../tests/invariants/packet_consumers.rs"]
mod tests;
