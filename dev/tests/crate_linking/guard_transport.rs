//! Fresh CLI processes re-prove version-thirteen match guards in provider
//! bodies (guard views, guard starts and uses, re-selected pattern places and
//! imaginary edges) and single-variant matches, which carry no constructor test.
use super::fresh_packets::{Packet, Result, relabel, unique, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/provider-guards";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const MATCHES: &str = "pa29/tests/spec/enum-match";
const GROUP: &str = "provider-guards";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/provider-guard-outcomes.json`.
const ACCEPTED: [&str; 8] = [
    "guarded",
    "fallback",
    "retry",
    "predicate",
    "method",
    "single",
    "single-guard",
    "own-single",
];
const OPPONENTS: [(&str, &str); 3] = [
    ("moved-scrutinee", "uninitialized or moved"),
    (
        "wrong-predicate",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
    ("wrong-single", "incompatible types"),
];
const FENCED: &str = "source guard facts require a wider transport profile";
const EITHER: &str = "// Generated older-profile guard control.\npub fn either(x: bool, y: bool) -> bool { match x { true if y => true, _ => false } }\n";
const EITHER_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { if base::either(true, true) { 42 } else { 7 } }\n";
const WIRE: &str = "// Generated guard wire control.\npub fn large(x: u8) -> bool { match x { 0..=3 => false, _ => true } }\npub fn guard(o: Opt<u8>) -> u8 { match o { Opt::Some(x) if large(x) => x, _ => 0 } }\n";
/// Body IR nodes: code, arity, then typed operands (Flag 8, Local 2,
/// Block 1, Place-projection 12, Place-local 11).
const GUARD_START: [u8; 7] = [2, 1, 8, 0, 0, 0, 0];
const GUARD_USE: [u8; 7] = [3, 1, 8, 0, 0, 0, 0];
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
fn word(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("operand is outside the packet")?
            .try_into()?,
    ))
}
fn generated(packet: &Packet, name: &str, source: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, source)?;
    Ok(path)
}
fn exported(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut export = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut export).map(drop)
}
/// The unique node of `code` with arity 2 whose first operand has `kind`
/// and whose second operand has one of `second`, after `after`.
fn node(bytes: &[u8], after: usize, head: [u8; 3], second: &[u8]) -> Result<usize> {
    let found: Vec<usize> = (after..bytes.len().saturating_sub(12))
        .filter(|&offset| bytes[offset..offset + 3] == head && second.contains(&bytes[offset + 7]))
        .collect();
    match found.as_slice() {
        [offset] => Ok(*offset),
        _ => Err(format!("body IR nodes {head:?} at {found:?}").into()),
    }
}
/// One local row: scope, optional storage parent, type, kind (0 and 3 carry
/// an ordinal), optional declaration, mutability and site. Returns the kind
/// byte's offset and the row's end.
fn local_row(bytes: &[u8], mut offset: usize) -> Option<(usize, usize)> {
    let option = |offset: usize| match bytes.get(offset)? {
        0 => Some(offset + 1),
        1 => Some(offset + 5),
        _ => None,
    };
    offset = option(offset + 4)? + 4;
    let kind = offset;
    offset += match bytes.get(kind)? {
        0 | 3 => 5,
        1 | 2 => 1,
        _ => return None,
    };
    offset = option(offset)?;
    (*bytes.get(offset)? <= 1).then_some((kind, offset + 5))
}
/// The kind byte of guard view `view`: the nearest locals list before the
/// guard's start whose row `view` is a kind-3 local.
fn view_kind(bytes: &[u8], start: usize, view: u32) -> Result<usize> {
    for list in (start.saturating_sub(4096)..start).rev() {
        let Ok(count) = word(bytes, list) else {
            continue;
        };
        if !(1..=64).contains(&count) || count <= view {
            continue;
        }
        let mut offset = list + 4;
        let mut kinds = Vec::new();
        for _ in 0..count {
            let Some((kind, end)) = local_row(bytes, offset) else {
                break;
            };
            kinds.push(kind);
            offset = end;
        }
        if kinds.len() == count as usize && bytes[kinds[view as usize]] == 3 {
            return Ok(kinds[view as usize]);
        }
    }
    Err("guard view local row is absent".into())
}
#[test]
fn provider_guards_survive_wire_and_fresh_consumers() -> Result {
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
    for profile in OLDER {
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
fn provider_guards_keep_older_gates_and_amend_the_enum_match_fence() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let either = generated(&packet, "either.rs", &format!("{nominals}{EITHER}"))?;
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &either, &older),
            FENCED,
        )?;
    }
    let metadata = packet.folder.join("either.rgcrate");
    exported(&packet, &either, &metadata)?;
    let client = generated(&packet, "either-client.rs", EITHER_CLIENT)?;
    let mut consumer = packet.consumer(&client, "either-client", &metadata, &packet.owner());
    super::artifacts::checked("either-client", &packet.folder, &mut consumer)?;
    // Amended: the enum-match `guarded` control was fenced by name.
    let matches = std::fs::read_to_string(packet.root.join(format!("{MATCHES}/provider.rs")))?;
    let guarded = generated(
        &packet,
        "guarded.rs",
        &format!(
            "{matches}pub fn large(x: u8) -> bool {{ match x {{ 0..=3 => false, _ => true }} }}\npub fn guard(o: Opt<u8>) -> u8 {{ match o {{ Opt::Some(x) if large(x) => x, _ => 0 }} }}\n"
        ),
    )?;
    let metadata = packet.folder.join("guarded.rgcrate");
    exported(&packet, &guarded, &metadata)?;
    let client = generated(
        &packet,
        "guarded-client.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::guard(base::Opt::Some(42u8)) { 42 => 42, _ => 7 } }\n",
    )?;
    let mut consumer = packet.consumer(&client, "guarded-client", &metadata, &packet.owner());
    super::artifacts::checked("guarded-client", &packet.folder, &mut consumer)?;
    Ok(())
}
/// Same-length or minimal wire forgeries of the generated `guard` packet,
/// with their consumer diagnostics, and the sound imaginary-edge swap.
type Forgery = (&'static str, Vec<u8>, &'static str);
fn forgeries(bytes: &[u8]) -> Result<(Vec<Forgery>, [Vec<u8>; 2])> {
    let start = unique(bytes, &GUARD_START)?;
    let used = unique(bytes, &GUARD_USE)?;
    // BindView [Local(view), Place(projection)] follows the guard's start.
    let bind = node(bytes, start, [4, 2, 2], &[11, 12])?;
    let view = word(bytes, bind + 3)?;
    let kind = view_kind(bytes, start, view)?;
    // FalseEdge [Block(real), Block(imaginary)] enters the guarded decision;
    // its real successor ends in the variant test whose `yes` it can skip to.
    let edge = node(bytes, start, [1, 2, 1], &[1])?;
    let real = word(bytes, edge + 3)?;
    let imaginary = word(bytes, edge + 8)?;
    // Branch [Value(test), Block(yes), Block(no)] whose failure is the
    // imaginary successor: the guarded decision's constructor test.
    let tests: Vec<usize> = (edge..bytes.len().saturating_sub(17))
        .filter(|&offset| {
            bytes[offset..offset + 3] == [2, 3, 0]
                && bytes[offset + 7] == 1
                && bytes[offset + 12] == 1
                && word(bytes, offset + 13).ok() == Some(imaginary)
        })
        .collect();
    let [test] = tests.as_slice() else {
        return Err(format!("guarded constructor tests {tests:?}").into());
    };
    let past = word(bytes, test + 8)?;
    let mut viewless = bytes[..kind].to_vec();
    viewless.push(1);
    viewless.extend_from_slice(&bytes[kind + 5..]);
    let swapped = with_id(&with_id(bytes, edge + 3, imaginary), edge + 8, real);
    // The one guard row (tested range, view range) after the calls.
    let rows: Vec<usize> = (used..bytes.len().saturating_sub(20))
        .filter(|&offset| {
            word(bytes, offset).ok() == Some(1)
                && word(bytes, offset + 12).ok() == Some(view)
                && word(bytes, offset + 16).ok() == Some(view + 1)
        })
        .collect();
    let [row] = rows.as_slice() else {
        return Err(format!("guard rows {rows:?}").into());
    };
    let untested = with_id(bytes, row + 8, word(bytes, row + 4)?);
    Ok((
        vec![
            (
                "view-binding",
                viewless,
                "source guard view binds an ordinary local",
            ),
            (
                "start-ordinal",
                with_id(bytes, start + 3, 1),
                "source guard ordinal is outside its guard table",
            ),
            (
                "use-ordinal",
                with_id(bytes, used + 3, 1),
                "source guard ordinal is outside its guard table",
            ),
            (
                "bind-parameter",
                with_id(bytes, bind + 3, 0),
                "source guard view binds an ordinary local",
            ),
            (
                "skip-test",
                with_id(bytes, edge + 3, past),
                "source downcast field use is not dominated by its variant test",
            ),
            (
                "version-12",
                relabel(bytes, 12),
                "unknown source declaration kind",
            ),
        ],
        [swapped, untested],
    ))
}
#[test]
fn forged_guard_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("// Authored provider-guard transport controls.")
        .next()
        .ok_or("guard foundation")?;
    let source = generated(&packet, "wire.rs", &format!("{foundation}{WIRE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let client = packet.fixture("fallback");
    let mut honest = packet.consumer(&client, "honest", &packet.metadata, &packet.owner());
    super::artifacts::checked("honest", &packet.folder, &mut honest)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let (forged, sound) = forgeries(&bytes)?;
    for (name, wire, diagnostic) in forged {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let mut consumer = packet.consumer(&client, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // Sound relabels. The swap makes the guarded arm unreachable, so it never
    // downcasts. Without tested places the guard keeps its views' deep
    // protection, and every downcast is still re-proved by its own test.
    for (name, wire) in ["swapped", "untested"].into_iter().zip(sound) {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let mut consumer = packet.consumer(&client, name, &path, &packet.owner());
        super::artifacts::checked(name, &packet.folder, &mut consumer)?;
    }
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn guarded_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "guards")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("// Authored provider-guard transport controls.")
        .next()
        .ok_or("guard foundation")?;
    let client = generated(
        &packet,
        "client.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::g005(base::Opt::Some(42u8)) { 42 => 42, _ => 7 } }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut guards = String::new();
        for index in 0..width {
            writeln!(
                guards,
                "pub fn g{index:03}(o: Opt<u8>) -> u8 {{ match o {{ Opt::Some(x) if large(x) => x, _ => {index} }} }}"
            )?;
        }
        let source = generated(
            &packet,
            &format!("guards{width}.rs"),
            &format!(
                "{foundation}// Generated scaling control.\npub fn large(x: u8) -> bool {{ match x {{ 0..=3 => false, _ => true }} }}\n{guards}"
            ),
        )?;
        let metadata = packet.folder.join(format!("guards{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names and 16-byte scalars: every guarded function adds the
    // same declaration, header, body and guard-column bytes.
    let ([small, middle, large], [first, _, third]) = (
        <[u64; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(first.abs_diff(third) <= 64, "{lowir:?}");
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn provider_guards_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let either = generated(&packet, "either.rs", &format!("{nominals}{EITHER}"))?;
    let metadata = packet.folder.join("either.rgcrate");
    exported(&packet, &either, &metadata)?;
    let client = generated(&packet, "either-client.rs", EITHER_CLIENT)?;
    let mut consumer = packet.consumer(&client, "either-client", &metadata, &packet.owner());
    super::artifacts::checked("either-client", &packet.folder, &mut consumer)?;
    let output = packet.native("either-client")?;
    assert_eq!(output.status.code(), Some(42), "either-client");
    Ok(())
}
