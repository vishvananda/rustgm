//! Fresh CLI processes consume version-twelve Branch rows; malformed or older wires reject.
use super::fresh_packets::{Packet, Result, relabel, unique, with_id};
use rustgm::support::crates;
use std::os::unix::process::ExitStatusExt;
const SPEC: &str = "pa29/tests/spec/source-branches";
/// An entry Branch row: condition value 0, yes block 1, no block 2 (unique in `wait`).
const ENTRY_BRANCH: [u8; 13] = [3, 0, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0];
/// The second exit of `choose`: condition value 2, yes block 4, no block 5.
const CHOOSE_SECOND: [u8; 13] = [3, 2, 0, 0, 0, 4, 0, 0, 0, 5, 0, 0, 0];
/// Bytes from the `choose` entry Branch tag to its second Branch tag.
const CHOOSE_STRIDE: usize = 42;
/// The complete three-block column of `spin`; its final byte is the dead Unreachable tag.
const SPIN_BLOCKS: [u8; 51] = [
    3, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 3, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0,
    0, 1, 1, 0, 0, 0, 2, 0, 0, 0, 2, 0, 0, 0, 3, 0, 0, 0, 2,
];
/// Frozen positive outcomes from `pa29/tests/source-branch-outcomes.json`, with the
/// lowered two-way exits each fresh consumer must retain (client `if` included).
const POSITIVE: [(&str, &str, i32, usize); 8] = [
    ("provider", "pick-i32", 7, 1),
    ("provider", "pick-bool", 12, 2),
    ("provider", "swap-packet", 22, 1),
    ("provider", "choose-i32", 2, 2),
    ("provider", "choose-u8", 13, 2),
    ("provider", "settle-i32", 9, 1),
    ("provider", "settle-bool", 31, 2),
    ("wait", "wait-ready", 5, 1),
];
const COPY: &str = "impl Clone for bool { fn clone(&self) -> Self { *self } }\nimpl Copy for bool {}\nimpl Clone for i32 { fn clone(&self) -> Self { *self } }\nimpl Copy for i32 {}\n";
struct Branches;
impl Branches {
    fn packet(label: &str, provider: &'static str) -> Result<Packet> {
        Packet::new("body-branches", SPEC, label, provider)
    }
    fn publish(label: &str, provider: &'static str) -> Result<Packet> {
        Packet::publish(
            "body-branches",
            SPEC,
            label,
            provider,
            "module-branches-v12",
        )
    }
}
#[test]
fn source_branches_survive_wire_and_fresh_instance_consumers() -> Result {
    let packet = Branches::publish("frontend", "provider")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([12, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let descriptor = interface.source_descriptor().ok_or("source descriptor")?;
    assert_eq!(
        descriptor.profile(),
        crates::SourceProfile::Source2024ModuleBranches
    );
    for (_, name, _, branches) in POSITIVE.iter().filter(|row| row.0 == "provider") {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let lowir = std::fs::read_to_string(packet.folder.join(format!("{name}.lowir")))?;
        assert_eq!(lowir.matches("branch ").count(), *branches, "{name}");
    }
    for (name, diagnostic) in [
        ("moved", "local may be uninitialized"),
        ("nonbool", "incompatible types"),
        ("copyless", "trait obligation requires"),
    ] {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    // The first `choose` exit names the second condition, evaluated only on its
    // no path: a well-typed bool whose use the fresh consumer's own value
    // dominance proof must reject.
    let first = unique(&bytes, &CHOOSE_SECOND)?
        .checked_sub(CHOOSE_STRIDE)
        .ok_or("choose column")?;
    assert_eq!(bytes[first..first + 13], ENTRY_BRANCH);
    let early = packet.folder.join("early.rgcrate");
    std::fs::write(&early, with_id(&bytes, first + 1, 2))?;
    let choose = packet.fixture("choose-i32");
    let mut consumer = packet.consumer(&choose, "early", &early, &packet.owner());
    packet.rejected("early", &mut consumer, "does not dominate its use")?;
    let older = packet.folder.join("older.rgcrate");
    let source = packet.fixture("provider");
    packet.rejected(
        "older",
        &mut packet.export("module-nominals-v10", &source, &older),
        "require the version-twelve profile",
    )?;
    assert!(!older.exists() || std::fs::metadata(older)?.len() == 0);
    Ok(())
}
#[test]
fn malformed_older_and_foreign_branch_wires_reject_in_fresh_consumers() -> Result {
    let packet = Branches::publish("wire", "wait")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let row = unique(&bytes, &ENTRY_BRANCH)?;
    let ready = packet.fixture("wait-ready");
    let mut lost_no = bytes.clone();
    lost_no.drain(row + 9..row + 13);
    for (name, wire, diagnostic) in [
        (
            "relabelled",
            relabel(&bytes, 10),
            "unknown source original terminator",
        ),
        (
            "foreign-yes",
            with_id(&bytes, row + 5, 6),
            "successor is foreign",
        ),
        (
            "integer-condition",
            with_id(&bytes, row + 1, 1),
            "actual bool value",
        ),
        (
            "unknown-condition",
            with_id(&bytes, row + 1, 9),
            "condition is unknown",
        ),
        ("lost-no", lost_no, "truncated metadata table"),
        // Skipping the merge store is a well-formed CFG; only the fresh
        // consumer's own initialization proof can reject it.
        (
            "skipped-store",
            with_id(&bytes, row + 5, 3),
            "uninitialized or moved",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let owner = packet.owner();
        let mut consumer = packet.consumer(&ready, name, &path, &owner);
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    let mut foreign = packet.consumer(&ready, "foreign", &packet.metadata, "base@other_v12");
    packet.rejected("foreign", &mut foreign, "provider metadata is missing")?;
    let other = Branches::publish("wire", "provider")?;
    let owner = packet.owner();
    let mut swapped = packet.consumer(&ready, "swapped", &other.metadata, &owner);
    packet.rejected("swapped", &mut swapped, "provider metadata is missing")?;
    Ok(())
}
#[test]
fn version_twelve_keeps_older_bytes_and_gates_panic_rows() -> Result {
    let packet = Branches::publish("panic-gate", "spin")?;
    let current = std::fs::read(&packet.metadata)?;
    let older_path = packet.folder.join("older.rgcrate");
    let source = packet.fixture("spin");
    super::artifacts::checked(
        "older",
        &packet.folder,
        &mut packet.export("module-nominals-v10", &source, &older_path),
    )?;
    let older = std::fs::read(&older_path)?;
    assert_eq!(relabel(&current, 10), older);
    let interface = crates::decode(&older)?;
    assert_eq!(crates::encode(&interface)?, older);
    let descriptor = interface.source_descriptor().ok_or("source descriptor")?;
    assert_eq!(
        descriptor.profile(),
        crates::SourceProfile::Source2024ModuleNominals
    );
    let tag = unique(&current, &SPIN_BLOCKS)? + SPIN_BLOCKS.len() - 1;
    let mut panic = current.clone();
    panic[tag] = 4;
    assert_eq!(crates::encode(&crates::decode(&panic)?)?, panic);
    let error = crates::decode(&relabel(&panic, 10))
        .err()
        .ok_or("version ten accepted a Panic exit")?;
    assert!(
        error.contains("unknown source original terminator"),
        "{error}"
    );
    Ok(())
}
#[test]
fn branch_rows_and_fresh_consumers_grow_linearly_with_exits() -> Result {
    let packet = Branches::packet("scaling", "chain")?;
    let fixture = std::fs::read_to_string(packet.fixture("wait"))?;
    let foundation = fixture
        .split("// Authored")
        .next()
        .ok_or("original core foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::chain(false, 3i32, 4i32) }\n",
    )?;
    let mut sizes = Vec::new();
    let mut lowir = Vec::new();
    for width in [6, 24, 96] {
        let arms = vec!["if flag { value }"; width].join(" else ");
        let source = packet.folder.join(format!("chain{width}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}// Generated scaling control.\n{COPY}pub fn chain<T: Copy>(flag: bool, value: T, other: T) -> T {{ {arms} else {{ other }} }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("chain{width}.rgcrate"));
        let mut export = packet.export("module-branches-v12", &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        assert_eq!(text.matches("branch ").count(), width);
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-width rows: every exit adds the same wire bytes. LowIR names grow by
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
fn source_branch_instances_and_forged_panic_exit_execute_natively() -> Result {
    let main = Branches::publish("native", "provider")?;
    let wait = Branches::publish("native", "wait")?;
    for (provider, name, exit, _) in POSITIVE {
        let packet = if provider == "wait" { &wait } else { &main };
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(exit), "{name}");
        assert!(
            output.stdout.is_empty() && output.stderr.is_empty(),
            "{name}"
        );
    }
    // No source construct emits Panic yet. Redirect the looping edge to the dead
    // Unreachable block and relabel it Panic: the fresh consumer re-proves that
    // CFG and must reach the shared abort exit; the ready path is unchanged.
    let bytes = std::fs::read(&wait.metadata)?;
    let tags = block_tags(&bytes, unique(&bytes, &ENTRY_BRANCH)?)?;
    let (jump, dead) = (tags[2], tags[5]);
    assert_eq!(bytes[jump..jump + 5], [1, 4, 0, 0, 0]);
    assert_eq!(bytes[dead], 2);
    let mut forged = with_id(&bytes, jump + 1, 5);
    forged[dead] = 4;
    let path = wait.folder.join("forged.rgcrate");
    std::fs::write(&path, forged)?;
    for (name, label) in [
        ("wait-ready", "forged-ready"),
        ("wait-failed", "forged-failed"),
    ] {
        let mut consumer = wait.consumer(&wait.fixture(name), label, &path, &wait.owner());
        super::artifacts::checked(label, &wait.folder, &mut consumer)?;
    }
    let lowir = std::fs::read_to_string(wait.folder.join("forged-failed.lowir"))?;
    assert!(lowir.contains("call void @abort()"), "{lowir}");
    assert_eq!(wait.native("forged-ready")?.status.code(), Some(5));
    let failed = wait.native("forged-failed")?;
    assert_eq!(failed.status.signal(), Some(6), "{:?}", failed.status);
    Ok(())
}
/// Terminator tag offsets of the block column whose first row is `row`.
fn block_tags(bytes: &[u8], row: usize) -> Result<Vec<usize>> {
    let count = row.checked_sub(16).ok_or("block column")?;
    let count = u32::from_le_bytes(bytes.get(count..count + 4).ok_or("count")?.try_into()?);
    let mut tags = Vec::new();
    let mut tag = row;
    for _ in 0..count {
        tags.push(tag);
        let payload = match bytes.get(tag) {
            Some(0 | 1) => 4,
            Some(2 | 4) => 0,
            Some(3) => 12,
            _ => return Err("unknown block tag".into()),
        };
        tag += 1 + payload + 12;
    }
    Ok(tags)
}
