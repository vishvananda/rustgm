//! Fresh CLI processes re-prove version-thirteen enum headers, constructor
//! tests, downcast fields and match exhaustiveness in transported bodies.
use super::fresh_packets::{Packet, Result, relabel, unique, with_id};
use rustgm::support::crates;
use std::fmt::Write as _;
const SPEC: &str = "pa29/tests/spec/enum-match";
const GROUP: &str = "enum-match";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/enum-match-outcomes.json`.
const ACCEPTED: [&str; 5] = ["fieldless", "tagged", "generic", "mapped", "by-ref"];
const OPPONENTS: [&str; 2] = ["wrong-enum", "wrong-payload"];
/// Body IR node 14 (`Rvalue::IsVariant`) of local 0, then its variant operand.
const TEST: [u8; 8] = [14, 2, 11, 0, 0, 0, 0, 5];
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
/// The variant operand offsets of every constructor test of local 0, in body
/// order: `pick` (A, B), `or`, `map` and `peek` (Some, None each).
fn tests(bytes: &[u8]) -> Vec<usize> {
    bytes
        .windows(TEST.len())
        .enumerate()
        .filter(|(_, window)| *window == TEST)
        .map(|(offset, _)| offset + TEST.len())
        .collect()
}
fn id(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("operand is outside the packet")?
            .try_into()?,
    ))
}
/// A variant row's shape byte followed by its sixteen discriminant bytes.
fn variant(shape: u8, discriminant: u8) -> [u8; 17] {
    let mut row = [0; 17];
    row[0] = shape;
    row[1] = discriminant;
    row
}
fn with_byte(bytes: &[u8], offset: usize, value: u8) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[offset] = value;
    changed
}
#[test]
fn enum_headers_and_matches_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    assert_eq!(tests(&bytes).len(), 8);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for name in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), "incompatible types")?;
    }
    let source = packet.fixture("provider");
    let fixture = std::fs::read_to_string(&source)?;
    let foundation = fixture
        .split("// Authored callable-role")
        .next()
        .ok_or("older-profile foundation")?;
    let sum = packet.folder.join("older.rs");
    std::fs::write(&sum, format!("{foundation}pub enum Two {{ A, B }}\n"))?;
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &sum, &older),
            "source declaration kind is outside the initial source profile",
        )?;
    }
    // Called tuple-variant constructors are the enum-methods checkpoint's;
    // provider-body match guards are the provider-guard checkpoint's.
    let called = packet.folder.join("called.rs");
    std::fs::write(
        &called,
        format!(
            "{}pub fn wrap(x: u8) -> Opt<u8> {{ Opt::Some(x) }}\n",
            std::fs::read_to_string(&source)?
        ),
    )?;
    let metadata = packet.folder.join("called.rgcrate");
    super::artifacts::checked(
        "called",
        &packet.folder,
        &mut packet.export(PROFILE, &called, &metadata),
    )?;
    let client = packet.folder.join("wrapped.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::or(base::wrap(42u8), 1u8) { 42 => 42, _ => 7 } }\n",
    )?;
    let mut consumer = packet.consumer(&client, "wrapped", &metadata, &packet.owner());
    super::artifacts::checked("wrapped", &packet.folder, &mut consumer)?;
    // Amended: this guard was fenced by name until the provider-guard
    // checkpoint; it now exports and a fresh consumer re-proves it.
    let guarded = packet.folder.join("guarded.rs");
    std::fs::write(
        &guarded,
        format!(
            "{}pub fn large(x: u8) -> bool {{ match x {{ 0..=3 => false, _ => true }} }}\npub fn guard(o: Opt<u8>) -> u8 {{ match o {{ Opt::Some(x) if large(x) => x, _ => 0 }} }}\n",
            std::fs::read_to_string(&source)?
        ),
    )?;
    let metadata = packet.folder.join("guarded.rgcrate");
    super::artifacts::checked(
        "guarded",
        &packet.folder,
        &mut packet.export(PROFILE, &guarded, &metadata),
    )?;
    let client = packet.folder.join("guard-client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::guard(base::Opt::Some(42u8)) { 42 => 42, _ => 7 } }\n",
    )?;
    let mut consumer = packet.consumer(&client, "guard-client", &metadata, &packet.owner());
    super::artifacts::checked("guard-client", &packet.folder, &mut consumer)?;
    Ok(())
}
#[test]
fn forged_enum_headers_tests_and_downcasts_reject_in_fresh_consumers() -> Result {
    let packet = publish("wire")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let operands = tests(&bytes);
    let [pick_a, pick_b, or_some, or_none, ..] = operands.as_slice() else {
        return Err("constructor tests are absent".into());
    };
    let (two_a, opt_some, opt_none) = (
        id(&bytes, *pick_a)?,
        id(&bytes, *or_some)?,
        id(&bytes, *or_none)?,
    );
    let high = unique(&bytes, &variant(0, 39))? + 1;
    let some = unique(&bytes, &variant(1, 1))?;
    for (name, client, wire, diagnostic) in [
        (
            "repeated-test",
            "fieldless",
            with_id(&bytes, *pick_b, two_a),
            "transported match is not exhaustive over its enum",
        ),
        (
            "repeated-payload-test",
            "generic",
            with_id(&bytes, *or_some, opt_none),
            "transported match is not exhaustive over its enum",
        ),
        (
            "swapped-tests",
            "generic",
            with_id(&with_id(&bytes, *or_some, opt_none), *or_none, opt_some),
            "source downcast field use is not dominated by its variant test",
        ),
        (
            "foreign-variant",
            "fieldless",
            with_id(&bytes, *pick_a, opt_none),
            "source variant belongs to another actual enum",
        ),
        (
            "duplicate-discriminant",
            "tagged",
            with_byte(&bytes, high, 3),
            "source enum has a duplicate discriminant",
        ),
        (
            "wide-discriminant",
            "tagged",
            with_byte(&bytes, high + 1, 1),
            "source enum discriminant exceeds its representation",
        ),
        (
            "data-discriminant",
            "generic",
            with_byte(&bytes, some + 1, 5),
            "source data enum discriminants require an integer representation",
        ),
        (
            "unit-payload",
            "generic",
            with_byte(&bytes, some, 0),
            "source unit variant carries payload fields",
        ),
        // Shape byte 2 is a record variant (record-variant amendment 3).
        (
            "unknown-shape",
            "generic",
            with_byte(&bytes, some, 3),
            "unknown source variant shape",
        ),
        (
            "relabelled",
            "fieldless",
            relabel(&bytes, 12),
            "unknown source declaration kind",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let source = packet.fixture(client);
        let mut consumer = packet.consumer(&source, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}
#[test]
fn enum_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "variants")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("// Authored enum-and-match")
        .next()
        .ok_or("enum foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::rank(base::lowest()) { 0 => 42, _ => 7 } }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let (mut variants, mut arms) = (String::new(), String::new());
        for index in 0..width {
            write!(variants, "V{index:03}, ")?;
            write!(arms, "Wide::V{index:03} => {index}, ")?;
        }
        let source = packet.folder.join(format!("variants{width}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}// Generated scaling control.\npub enum Wide {{ {variants}}}\npub fn lowest() -> Wide {{ Wide::V000 }}\npub fn rank(w: Wide) -> u8 {{ match w {{ {arms}}} }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("variants{width}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every variant and arm adds the same packet bytes;
    // the consumer re-proves and lowers each arm once.
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
fn enum_match_instances_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
