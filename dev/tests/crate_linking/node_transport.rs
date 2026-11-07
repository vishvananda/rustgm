//! Fresh CLI processes consume version-thirteen body IR nodes; every allocated
//! form a consumer cannot rebuild, and every older wire, rejects by name.
use super::fresh_packets::{Packet, Result, relabel, with_id};
use rustgm::support::crates;
use std::fmt::Write as _;
const SPEC: &str = "pa29/tests/spec/source-body-ir";
const BRANCHES: &str = "pa29/tests/spec/source-branches";
const GROUP: &str = "body-ir";
const PROFILE: &str = "module-bodies-v13";
const OLDER: &str = "source Binary operations require the version-thirteen body IR profile";
/// Frozen outcomes from `pa29/tests/source-body-ir-outcomes.json`, with the
/// primitive comparisons each fresh consumer lowers: one per literal pattern
/// and two per range (counts recorded after the first candidate observation).
const POSITIVE: [(&str, i32, usize); 12] = [
    ("grade-low", 3, 2),
    ("grade-edge", 41, 2),
    ("grade-high", 44, 2),
    ("band-middle", 32, 4),
    ("band-over", 36, 4),
    ("exact-hit", 21, 1),
    ("exact-bool", 23, 1),
    ("either-or", 24, 3),
    ("either-hit", 38, 3),
    ("letter-hit", 27, 2),
    ("letter-miss", 30, 2),
    ("flag-off", 14, 2),
];
/// Version-twelve clients and providers re-encoded as version-thirteen nodes.
const REENCODED: [(&str, &str, i32); 8] = [
    ("provider", "pick-i32", 7),
    ("provider", "pick-bool", 12),
    ("provider", "swap-packet", 22),
    ("provider", "choose-i32", 2),
    ("provider", "choose-u8", 13),
    ("provider", "settle-i32", 9),
    ("provider", "settle-bool", 31),
    ("wait", "wait-ready", 5),
];
fn publish(label: &str, provider: &'static str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, provider, PROFILE)
}
/// Offsets of value nodes `Binary [Value, Value, Flag, Flag]`.
fn binary_nodes(bytes: &[u8]) -> Vec<usize> {
    (0..bytes.len().saturating_sub(22))
        .filter(|&at| {
            bytes[at..at + 3] == [27, 4, 0]
                && bytes[at + 7] == 0
                && bytes[at + 12] == 8
                && bytes[at + 17] == 8
        })
        .collect()
}
/// Offset of the unique terminator node `Branch [Value, Block, Block]`.
fn branch_node(bytes: &[u8]) -> Result<usize> {
    let found: Vec<usize> = (0..bytes.len().saturating_sub(17))
        .filter(|&at| bytes[at..at + 3] == [2, 3, 0] && bytes[at + 7] == 1 && bytes[at + 12] == 1)
        .collect();
    match found.as_slice() {
        [at] => Ok(*at),
        _ => Err(format!("ambiguous Branch anchor {found:?}").into()),
    }
}
fn set(bytes: &[u8], offset: usize, value: u8) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[offset] = value;
    changed
}
#[test]
fn body_nodes_survive_wire_and_fresh_instance_consumers() -> Result {
    let packet = publish("frontend", "provider")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let descriptor = interface.source_descriptor().ok_or("source descriptor")?;
    assert_eq!(
        descriptor.profile(),
        crates::SourceProfile::Source2024ModuleBodies
    );
    for (name, _, comparisons) in POSITIVE {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let lowir = std::fs::read_to_string(packet.folder.join(format!("{name}.lowir")))?;
        assert_eq!(lowir.matches(" = cmp ").count(), comparisons, "{name}");
    }
    for (name, diagnostic) in [
        ("mismatch", "incompatible types"),
        ("copyless", "trait obligation requires"),
    ] {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    let source = packet.fixture("provider");
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(profile, &mut packet.export(profile, &source, &older), OLDER)?;
        assert!(!older.exists() || std::fs::metadata(older)?.len() == 0);
    }
    Ok(())
}
#[test]
fn version_thirteen_reencodes_branch_packets_without_changing_consumers() -> Result {
    for provider in ["provider", "wait", "spin"] {
        let twelve = Packet::publish(GROUP, BRANCHES, "twelve", provider, "module-branches-v12")?;
        let thirteen = Packet::publish(GROUP, BRANCHES, "thirteen", provider, PROFILE)?;
        let bytes = std::fs::read(&thirteen.metadata)?;
        assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
        assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
        for (_, name, _) in REENCODED.iter().filter(|row| row.0 == provider) {
            super::artifacts::checked(name, &twelve.folder, &mut twelve.client(name))?;
            super::artifacts::checked(name, &thirteen.folder, &mut thirteen.client(name))?;
            let lowir =
                |packet: &Packet| std::fs::read(packet.folder.join(format!("{name}.lowir")));
            assert_eq!(lowir(&twelve)?, lowir(&thirteen)?, "{name}");
        }
    }
    Ok(())
}
#[test]
fn unsupported_malformed_and_relabelled_nodes_reject_in_fresh_consumers() -> Result {
    let packet = publish("wire", "wire")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let [node] = <[usize; 1]>::try_from(binary_nodes(&bytes)).map_err(|n| format!("{n:?}"))?;
    let branch = branch_node(&bytes)?;
    let mut aggregate = bytes.clone();
    aggregate[node] = 3;
    let client = packet.fixture("exact-hit");
    for (name, wire, diagnostic) in [
        (
            "checked-division",
            with_id(&bytes, node + 13, 12),
            "encoded checked division or shift Rvalue::Binary is not reconstructable",
        ),
        (
            "checked-add-result",
            with_id(&bytes, node + 13, 9),
            "changed its primitive result type",
        ),
        (
            "unknown-operator",
            with_id(&bytes, node + 13, 200),
            "binary node has an unknown operator",
        ),
        (
            "unknown-form",
            with_id(&bytes, node + 18, 5),
            "binary node has an unknown operand form",
        ),
        (
            "bitwise-result",
            with_id(&bytes, node + 13, 6),
            "changed its primitive result type",
        ),
        // Amended with the enum-match checkpoint: code 14 is now a
        // reconstructable constructor test whose operands differ.
        (
            "is-variant",
            set(&bytes, node, 14),
            "body IR value node has malformed operands",
        ),
        (
            "validate",
            set(&bytes, node, 15),
            "encoded Rvalue::Validate is not reconstructable by this consumer",
        ),
        (
            "unknown-value",
            set(&bytes, node, 200),
            "unknown body IR value form",
        ),
        (
            "aggregate",
            aggregate,
            "body IR value node has malformed operands",
        ),
        (
            "static-place",
            set(&bytes, node + 2, 14),
            "encoded static places are not reconstructable by this consumer",
        ),
        (
            "unknown-kind",
            set(&bytes, node + 2, 99),
            "unknown body IR operand kind",
        ),
        (
            "arity",
            set(&bytes, node + 1, 7),
            "arity exceeds its stable bound",
        ),
        (
            "forward-operand",
            with_id(&bytes, node + 8, 1000),
            "cyclic, forward or unknown",
        ),
        (
            "false-edge",
            set(&bytes, branch, 1),
            "encoded Terminator::FalseEdge is not reconstructable by this consumer",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let mut consumer = packet.consumer(&client, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    let relabelled = packet.folder.join("relabelled.rgcrate");
    std::fs::write(&relabelled, relabel(&bytes, 12))?;
    assert!(crates::decode(&relabel(&bytes, 12)).is_err());
    let mut consumer = packet.consumer(&client, "relabelled", &relabelled, &packet.owner());
    let output = consumer.output()?;
    assert_eq!(output.status.code(), Some(1));
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut foreign = packet.consumer(&client, "foreign", &packet.metadata, "base@other_v12");
    packet.rejected("foreign", &mut foreign, "provider metadata is missing")?;
    let other = publish("wire", "provider")?;
    let mut swapped = packet.consumer(&client, "swapped", &other.metadata, &packet.owner());
    packet.rejected("swapped", &mut swapped, "provider metadata is missing")?;
    Ok(())
}
#[test]
fn binary_nodes_and_fresh_consumers_grow_linearly_with_arms() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "arms")?;
    let fixture = std::fs::read_to_string(packet.fixture("wire"))?;
    let foundation = fixture
        .split("// Authored")
        .next()
        .ok_or("original core foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::arms(200u8, 3i32, 4i32) }\n",
    )?;
    let copy = "impl Clone for u8 { fn clone(&self) -> Self { *self } }\nimpl Copy for u8 {}\nimpl Clone for i32 { fn clone(&self) -> Self { *self } }\nimpl Copy for i32 {}\n";
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let arms = (0..width).try_fold(String::new(), |mut text, arm| {
            write!(text, "{arm} => hit, ").map(|()| text)
        })?;
        let source = packet.folder.join(format!("arms{width}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}// Generated scaling control.\n{copy}pub fn arms<T: Copy>(code: u8, hit: T, miss: T) -> T {{ match code {{ {arms}_ => miss }} }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("arms{width}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        assert_eq!(binary_nodes(&std::fs::read(&metadata)?).len(), width);
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        assert_eq!(text.matches(" = cmp eq ").count(), width);
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-width nodes: every arm adds the same wire bytes. LowIR names grow by
    // digits only, so a fourfold width step stays within fivefold growth.
    let ([small, middle, large], [first, second, third]) = (
        <[u64; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(third - second <= 5 * (second - first), "{lowir:?}");
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn body_node_instances_execute_natively() -> Result {
    let packet = publish("native", "provider")?;
    let wire = publish("native", "wire")?;
    let mut runs: Vec<(&Packet, &str, i32)> = POSITIVE
        .iter()
        .map(|&(name, exit, _)| (&packet, name, exit))
        .collect();
    runs.push((&wire, "exact-hit", 21));
    let main = Packet::publish(GROUP, BRANCHES, "native-branches", "provider", PROFILE)?;
    let wait = Packet::publish(GROUP, BRANCHES, "native-branches", "wait", PROFILE)?;
    for &(provider, name, exit) in &REENCODED {
        runs.push((if provider == "wait" { &wait } else { &main }, name, exit));
    }
    for (packet, name, exit) in runs {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(exit), "{name}");
        assert!(
            output.stdout.is_empty() && output.stderr.is_empty(),
            "{name}"
        );
    }
    Ok(())
}
