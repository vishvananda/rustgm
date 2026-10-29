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
const MODULE_CONTRACT_VERSION: u16 = 2;
const AVAILABILITY_VERSION: u16 = 3;
const FEATURE_PREDICATE_VERSION: u16 = 4;
const DEFINING_RUST2024: u8 = 0;

pub(super) fn encode(interface: &Interface) -> Result<Vec<u8>, &'static str> {
    let payload = interface
        .generic_payload()
        .ok_or("source envelope lacks its complete payload")?;
    let descriptor = interface
        .source_descriptor()
        .ok_or("source envelope lacks its actual descriptor")?;
    let tag = match descriptor.profile() {
        Profile::Source2024Functions
        | Profile::Source2024FunctionContracts
        | Profile::Source2024ModuleContracts
        | Profile::Source2024ModuleAvailability
        | Profile::Source2024ModuleFeaturePredicates => TAG,
        Profile::Source2024Provided | Profile::Source2024ProvidedContracts => PROVIDED_TAG,
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
    out.u16(match descriptor.profile() {
        Profile::Source2024ModuleFeaturePredicates => FEATURE_PREDICATE_VERSION,
        Profile::Source2024ModuleAvailability => AVAILABILITY_VERSION,
        Profile::Source2024ModuleContracts => MODULE_CONTRACT_VERSION,
        Profile::Source2024FunctionContracts | Profile::Source2024ProvidedContracts => {
            CONTRACT_VERSION
        }
        _ => ROW_VERSION,
    })?;
    out.u8(DEFINING_RUST2024)?;
    out.identity(interface.identity())?;
    out.u32(interface.dependencies().len())?;
    for dependency in interface.dependencies() {
        out.identity(dependency)?;
    }
    if descriptor.profile() == Profile::Source2024ProvidedContracts {
        ordinary_columns::encode_provided_contracts(&mut out, descriptor.rows())?;
    } else if tag == PROVIDED_TAG {
        ordinary_columns::encode_provided(&mut out, descriptor.rows())?;
    } else if descriptor.profile().availability() {
        ordinary_columns::encode_module_availability(&mut out, descriptor.rows())?;
    } else if descriptor.profile() == Profile::Source2024ModuleContracts {
        ordinary_columns::encode_module_contracts(&mut out, descriptor.rows())?;
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
    let version = defining_version(input, provided)?;
    let identity = input.identity()?;
    let dependencies = decode_dependencies(input)?;
    let interface = Interface::new(identity, dependencies, Vec::new())?;
    let base = interface.base_retained_bytes();
    if base > MAX_BYTES {
        return Err("source envelope base exceeds retained storage budget");
    }
    let (rows, attributes, modules, contracts, availability) = if !provided
        && matches!(version, AVAILABILITY_VERSION | FEATURE_PREDICATE_VERSION)
    {
        let packet = if version == FEATURE_PREDICATE_VERSION {
            ordinary_columns::decode_module_feature_predicates(input, base)?
        } else {
            ordinary_columns::decode_module_availability(input, base)?
        };
        (
            packet.rows,
            Some(packet.attributes),
            Some(packet.modules),
            None,
            Some(packet.availability),
        )
    } else if provided && version == CONTRACT_VERSION {
        let (rows, attributes, contracts) =
            ordinary_columns::decode_provided_contracts(input, base)?;
        (rows, Some(attributes), None, Some(contracts), None)
    } else if provided {
        (
            ordinary_columns::decode_provided(input, base)?,
            None,
            None,
            None,
            None,
        )
    } else if version == MODULE_CONTRACT_VERSION {
        let (rows, attributes, modules) = ordinary_columns::decode_module_contracts(input, base)?;
        (rows, Some(attributes), Some(modules), None, None)
    } else if version == CONTRACT_VERSION {
        let (rows, attributes) = ordinary_columns::decode_contracts(input, base)?;
        (rows, Some(attributes), None, None, None)
    } else {
        (
            ordinary_columns::decode(input, base)?,
            None,
            None,
            None,
            None,
        )
    };
    if !input.done() {
        return Err("trailing source metadata bytes");
    }
    let mut work = Work {
        steps: input.used_atoms(),
        ..Work::default()
    };
    let payload = if let Some(availability) = availability {
        let publish = if version == FEATURE_PREDICATE_VERSION {
            Payload::from_source_module_feature_predicates
        } else {
            Payload::from_source_module_availability
        };
        publish(
            interface.identity(),
            rows,
            attributes.ok_or("availability advisory columns missing")?,
            modules.ok_or("availability namespace columns missing")?,
            availability,
            base,
            &mut work,
        )?
    } else if let Some(contracts) = contracts {
        Payload::from_source_provided_contracts(
            interface.identity(),
            rows,
            attributes.ok_or("provided advisory completion is missing")?,
            contracts,
            base,
            &mut work,
        )?
    } else if provided {
        Payload::from_source_provided(interface.identity(), rows, base, &mut work)?
    } else if let Some(modules) = modules {
        Payload::from_source_module_contracts(
            interface.identity(),
            rows,
            attributes.ok_or("module advisory columns missing")?,
            modules,
            base,
            &mut work,
        )?
    } else if let Some(attributes) = attributes {
        Payload::from_source_contracts(interface.identity(), rows, attributes, base, &mut work)?
    } else {
        Payload::from_source(interface.identity(), rows, base, &mut work)?
    };
    interface.with_generic_payload(payload)
}

fn defining_version(input: &mut Reader<'_>, provided: bool) -> Result<u16, &'static str> {
    let version = input.u16()?;
    if (provided && !matches!(version, ROW_VERSION | CONTRACT_VERSION)
        || !provided
            && !matches!(
                version,
                ROW_VERSION
                    | CONTRACT_VERSION
                    | MODULE_CONTRACT_VERSION
                    | AVAILABILITY_VERSION
                    | FEATURE_PREDICATE_VERSION
            ))
        || input.u8()? != DEFINING_RUST2024
    {
        return Err("unsupported source row version or defining edition");
    }
    Ok(version)
}

#[cfg(test)]
#[path = "../../../../tests/invariants/packet_consumers.rs"]
mod tests;

#[cfg(test)]
#[path = "../../../../tests/invariants/domain_artifacts.rs"]
mod module_tests;
