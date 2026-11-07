//! Fresh CLI processes re-prove version-thirteen called tuple-variant
//! constructors and name imported variants through their enum's bindings.
use super::fresh_packets::{Packet, Result, unique, with_id};
use rustgm::support::crates;
use std::fmt::Write as _;
const SPEC: &str = "pa29/tests/spec/enum-methods";
const GROUP: &str = "enum-methods";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/enum-methods-outcomes.json`.
const ACCEPTED: [&str; 5] = ["called", "mapped", "named", "guarded", "by-ref"];
const OPPONENTS: [(&str, &str); 3] = [
    ("non-exhaustive", "non-exhaustive match"),
    ("wrong-payload", "incompatible types"),
    ("wrong-arity", "call argument count mismatch"),
];
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
/// A variant row from its ordinal on: shape, sixteen discriminant bytes and
/// its payload field range.
fn variant(ordinal: u8, shape: u8, discriminant: u8, fields: (u8, u8)) -> [u8; 29] {
    let mut row = [0; 29];
    row[0] = ordinal;
    row[4] = shape;
    row[5] = discriminant;
    row[21] = fields.0;
    row[25] = fields.1;
    row
}
fn id(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("operand is outside the packet")?
            .try_into()?,
    ))
}
/// The declaration operand offsets of every one-argument function
/// application naming `target` in the current artifact.
fn applications(bytes: &[u8], target: u32) -> Vec<usize> {
    let mut pattern = [0; 8];
    pattern[4..].copy_from_slice(&target.to_le_bytes());
    bytes
        .windows(16)
        .enumerate()
        .filter(|(_, row)| {
            row[..8] == pattern
                && u32::from_le_bytes([row[12], row[13], row[14], row[15]])
                    == u32::from_le_bytes([row[8], row[9], row[10], row[11]]) + 1
        })
        .map(|(offset, _)| offset + 4)
        .collect()
}
#[test]
fn called_constructors_and_imported_variants_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    let source = packet.fixture("provider");
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &source, &older),
            "named, associated or generic alias transport requires its wider profile",
        )?;
    }
    Ok(())
}
#[test]
fn forged_constructor_applications_reject_in_fresh_consumers() -> Result {
    let packet = publish("wire")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let some = unique(&bytes, &variant(1, 1, 1, (0, 1)))? - 4;
    let two = unique(&bytes, &variant(1, 1, 1, (2, 4)))? - 4;
    // The unit `None` row immediately precedes `Some` in its enum's table.
    let none = some - 33;
    assert_eq!(id(&bytes, none + 4)?, 0, "None ordinal");
    let (some, none, two) = (id(&bytes, some)?, id(&bytes, none)?, id(&bytes, two)?);
    // `wrap` and `map` each call `Opt::Some` once.
    let calls = applications(&bytes, some);
    assert_eq!(calls.len(), 2, "{calls:?}");
    for (name, target, diagnostic) in [
        (
            "unit-constructor",
            none,
            "source unit variant has no called constructor",
        ),
        (
            "foreign-constructor",
            two,
            "source application omitted or added actual unused type formals",
        ),
    ] {
        let wire = calls.iter().fold(bytes.clone(), |wire, &offset| {
            with_id(&wire, offset, target)
        });
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let source = packet.fixture("called");
        let mut consumer = packet.consumer(&source, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}
#[test]
fn constructor_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "constructors")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("// Authored called-constructor")
        .next()
        .ok_or("constructor foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::rank(base::pick(0u8)) { 0 => 42, _ => 7 } }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let (mut variants, mut calls, mut arms) = (String::new(), String::new(), String::new());
        for index in 0..width {
            write!(variants, "V{index:03}(u8), ")?;
            write!(calls, "{index} => Wide::V{index:03}(i), ")?;
            write!(arms, "Wide::V{index:03}(x) => x, ")?;
        }
        let source = packet.folder.join(format!("constructors{width}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}// Generated scaling control.\npub enum Wide {{ {variants}}}\npub fn pick(i: u8) -> Wide {{ match i {{ {calls}_ => Wide::V000(i) }} }}\npub fn rank(w: Wide) -> u8 {{ match w {{ {arms}}} }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("constructors{width}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every variant, constructor call and arm adds the
    // same packet bytes; the consumer synthesizes each referenced constructor
    // and lowers each arm once.
    let ([small, middle, large], [first, second, third]) = (
        <[u64; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(
        (third - second) <= 5 * (second - first).max(64),
        "{lowir:?}"
    );
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn constructor_instances_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
