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
pub(super) const PROVIDED_TAG: u8 = 11;
const ROW_VERSION: u16 = 0;
const CONTRACT_VERSION: u16 = 1;
const DEFINING_RUST2024: u8 = 0;

pub(super) fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let payload = interface
        .generic_payload()
        .ok_or("source envelope lacks its complete payload")?;
    let descriptor = interface
        .source_descriptor()
        .ok_or("source envelope lacks its actual descriptor")?;
    let tag = match descriptor.profile() {
        Profile::Source2024Functions | Profile::Source2024FunctionContracts => TAG,
        Profile::Source2024Provided => PROVIDED_TAG,
        Profile::Source2024 => return Err("unpublished source facade has no wire profile"),
    };
    if descriptor.edition() != Edition::Rust2024 || descriptor.identity() != interface.identity() {
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
    out.u8(tag)?;
    out.u16(
        if descriptor.profile() == Profile::Source2024FunctionContracts {
            CONTRACT_VERSION
        } else {
            ROW_VERSION
        },
    )?;
    out.u8(DEFINING_RUST2024)?;
    out.identity(interface.identity())?;
    out.u32(interface.dependencies().len())?;
    for dependency in interface.dependencies() {
        out.identity(dependency)?;
    }
    if tag == PROVIDED_TAG {
        ordinary_columns::encode_provided(&mut out, descriptor.rows())?;
    } else if descriptor.profile() == Profile::Source2024FunctionContracts {
        ordinary_columns::encode_contracts(&mut out, descriptor.rows())?;
    } else {
        ordinary_columns::encode(&mut out, descriptor.rows())?;
    }
    Ok(out.finish())
}
pub(super) fn decode(input: &mut Reader<'_>) -> Result<Interface, &'static str> {
    decode_profile(input, false)
}
pub(super) fn decode_provided(input: &mut Reader<'_>) -> Result<Interface, &'static str> {
    decode_profile(input, true)
}
fn decode_profile(input: &mut Reader<'_>, provided: bool) -> Result<Interface, &'static str> {
    let version = input.u16()?;
    if (version != ROW_VERSION && (provided || version != CONTRACT_VERSION))
        || input.u8()? != DEFINING_RUST2024
    {
        return Err("unsupported source row version or defining edition");
    }
    let identity = input.identity()?;
    let dependencies = decode_dependencies(input)?;
    let interface = Interface::new(identity, dependencies, Vec::new())?;
    let base = interface.base_retained_bytes();
    if base > MAX_BYTES {
        return Err("source envelope base exceeds retained storage budget");
    }
    let (rows, attributes) = if provided {
        (ordinary_columns::decode_provided(input, base)?, None)
    } else if version == CONTRACT_VERSION {
        let (rows, attributes) = ordinary_columns::decode_contracts(input, base)?;
        (rows, Some(attributes))
    } else {
        (ordinary_columns::decode(input, base)?, None)
    };
    if !input.done() {
        return Err("trailing source metadata bytes");
    }
    let mut work = Work {
        steps: input.used_atoms(),
        ..Work::default()
    };
    let payload = if provided {
        Payload::from_source_provided(interface.identity(), rows, base, &mut work)?
    } else if let Some(attributes) = attributes {
        Payload::from_source_contracts(interface.identity(), rows, attributes, base, &mut work)?
    } else {
        Payload::from_source(interface.identity(), rows, base, &mut work)?
    };
    interface.with_generic_payload(payload)
}

#[cfg(test)]
#[path = "../../../../tests/invariants/packet_consumers.rs"]
mod tests;
