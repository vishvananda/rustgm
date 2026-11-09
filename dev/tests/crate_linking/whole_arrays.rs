//! Fresh CLI processes consume whole array values exported under version
//! thirteen as type recipe tag 14, `14 | element | term`, with K's const term
//! in place. Each consumer proves the element sized from its own facts and
//! folds a value length to its array kind and a slot length to its const
//! parameter, so values, generic instances, fields, const-generic records
//! and the imported `[T; N]` impl headers all use them. List and repeat
//! rvalues, element patterns, unsizing and indexing keep their fences by
//! name, older profiles keep the type-form fence and forged lengths reject.
use super::fresh_packets::{Packet, Result, relabel};
use rustgm::support::crates;
use std::fmt::Write as _;
const SPEC: &str = "pa29/tests/spec/array-types";
const GROUP: &str = "array-types";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/array-type-outcomes.json`.
const ACCEPTED: [&str; 5] = ["values", "generic", "fields", "wide", "copies"];
const INCOMPATIBLE: &str = "incompatible types";
const REJECTED: [(&str, &str); 4] = [
    ("wrong-length", INCOMPATIBLE),
    ("wrong-element", INCOMPATIBLE),
    ("moved", "local may be uninitialized or moved at this read"),
    ("wide-length", INCOMPATIBLE),
];
const TYPE_FORM: &str = "source type form requires an explicitly supported transport row";
const RVALUE: &str = "source rvalue requires a complete wider transport row";
/// Each generated control is the provider's first 147 lines, the
/// lifetime-nominals foundation, then these lines, as the oracle wrote them.
const CONTROLS: [(&str, &str, &str); 5] = [
    ("list", "pub fn list(x: u8) -> [u8; 2] { [x, x] }\n", RVALUE),
    ("repeat", "pub fn zeros() -> [u8; 4] { [0; 4] }\n", RVALUE),
    (
        "element",
        "pub fn head(a: [u8; 4]) -> u8 { let [x, _, _, _] = a; x }\n",
        "source projection transport requires its wider explicit contract",
    ),
    (
        "unsize",
        "pub fn view(a: &[u8; 4]) -> &[u8] { a }\n",
        "reference unsizing requires its actual source CoerceUnsized declaration",
    ),
    // Observed on T's first development build, before any other T run.
    (
        "index",
        "#[cfg(rustc_oracle)]\n#[lang = \"panic_bounds_check\"]\n#[track_caller]\nfn panic_bounds_check(index: usize, len: usize) -> ! { abort() }\npub fn at(a: [u8; 4], i: usize) -> u8 { a[i] }\n",
        "source binary operator requires its actual language declaration",
    ),
];
type Forgery = fn(&mut [u8], usize);
/// Each forged `[u8; 4]` row and the name a fresh consumer rejects it by.
const FORGED: [(&str, Forgery, &str); 3] = [
    (
        "term-tag-2",
        |bytes, at| bytes[at + 5] = 2,
        "unknown source const term",
    ),
    (
        "bits-above-usize-width",
        |bytes, at| bytes[at + 18] = 1,
        "non-canonical source const value",
    ),
    (
        "term-type-is-the-element-row",
        |bytes, at| bytes.copy_within(at + 1..at + 5, at + 6),
        "source array length term is not usize",
    ),
];
fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
/// The provider's first 147 lines, which every generated control keeps.
fn foundation(packet: &Packet) -> Result<String> {
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let lines: Vec<&str> = provider.lines().take(147).collect();
    Ok(format!("{}\n", lines.join("\n")))
}
/// The offset of the one `14 | element | 0 | ty | bits` row whose length
/// is 4, which only `[u8; 4]` has.
fn anchor(bytes: &[u8]) -> Result<usize> {
    let mut four = [0_u8; 16];
    four[0] = 4;
    let found: Vec<usize> = bytes
        .windows(26)
        .enumerate()
        .filter(|(_, row)| row[0] == 14 && row[5] == 0 && row[10..] == four)
        .map(|(offset, _)| offset)
        .collect();
    let [offset] = *found.as_slice() else {
        return Err(format!("array anchors {found:?}").into());
    };
    Ok(offset)
}
/// Writes `source` under the packet's folder and exports it under `profile`.
fn generated(
    packet: &Packet,
    name: &str,
    profile: &str,
    source: &str,
) -> Result<std::process::Command> {
    let path = packet.folder.join(format!("{name}.rs"));
    std::fs::write(&path, source)?;
    let metadata = packet.folder.join(format!("{name}.rgcrate"));
    Ok(packet.export(profile, &path, &metadata))
}
#[test]
fn clients_move_whole_arrays_and_element_access_keeps_its_fences() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "arrays-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in REJECTED {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    let foundation = foundation(&packet)?;
    for (name, lines, diagnostic) in CONTROLS {
        let mut export = generated(&packet, name, PROFILE, &format!("{foundation}{lines}"))?;
        packet.rejected(name, &mut export, diagnostic)?;
        assert!(
            unwritten(&packet.folder.join(format!("{name}.rgcrate")))?,
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn forged_array_lengths_reject_by_name_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "arrays-forged", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let at = anchor(&bytes)?;
    for (name, forgery, diagnostic) in FORGED {
        let mut forged = bytes.clone();
        forgery(&mut forged, at);
        assert_ne!(forged, bytes, "{name}");
        let mut client = packet.forged(name, forged, "values")?;
        packet.rejected(name, &mut client, diagnostic)?;
    }
    // Version twelve's reader stops at the declaration tag that precedes
    // every type row.
    let mut client = packet.forged("relabelled-v12", relabel(&bytes, 12), "values")?;
    packet.rejected(
        "relabelled-v12",
        &mut client,
        "unknown source declaration kind",
    )?;
    Ok(())
}
#[test]
fn older_profiles_keep_the_array_type_form_fence() -> Result {
    let packet = Packet::new(GROUP, SPEC, "arrays-older", "provider")?;
    // The foundation's aliases stop both older profiles first.
    for profile in OLDER {
        let metadata = packet.folder.join(format!("{profile}.rgcrate"));
        let mut export = packet.export(profile, &packet.fixture("provider"), &metadata);
        packet.rejected(
            profile,
            &mut export,
            "named, associated or generic alias transport requires its wider profile",
        )?;
        assert!(unwritten(&metadata)?, "{profile} wrote a packet");
    }
    let nominals = std::fs::read_to_string(
        packet
            .root
            .join("pa29/tests/spec/source-nominals/provider.rs"),
    )?;
    let gate = format!("{nominals}pub fn keep(a: [u8; 4]) -> [u8; 4] {{ a }}\n");
    for profile in OLDER {
        let name = format!("gate-{profile}");
        let mut export = generated(&packet, &name, profile, &gate)?;
        packet.rejected(&name, &mut export, TYPE_FORM)?;
    }
    let mut export = generated(&packet, "gate-v13", PROFILE, &gate)?;
    super::artifacts::checked("gate-v13", &packet.folder, &mut export)?;
    Ok(())
}
#[test]
fn array_signatures_cost_the_same_bytes_each_and_unnamed_ones_cost_clients_nothing() -> Result {
    let packet = Packet::new(GROUP, SPEC, "arrays-scaling", "provider")?;
    let foundation = foundation(&packet)?;
    let client = packet.folder.join("unnamed.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { 42 }\n",
    )?;
    let mut sizes = Vec::new();
    let mut emitted = Vec::new();
    for count in [6_usize, 24, 96] {
        let mut source = foundation.clone();
        for i in 1..=count {
            writeln!(
                source,
                "pub fn a{i:03}(a: [u8; {i:03}]) -> [u8; {i:03}] {{ a }}"
            )?;
        }
        let name = format!("scaled-{count}");
        let mut export = generated(&packet, &name, PROFILE, &source)?;
        super::artifacts::checked(&name, &packet.folder, &mut export)?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        sizes.push(std::fs::metadata(&metadata)?.len());
        let label = format!("unnamed-{count}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        emitted.push(std::fs::read(packet.folder.join(format!("{label}.lowir")))?);
    }
    let [small, middle, large] = *sizes.as_slice() else {
        return Err("three scaled packets".into());
    };
    let each = (middle - small) / 18;
    assert_eq!(middle - small, 18 * each, "{sizes:?}");
    assert_eq!(large - middle, 72 * each, "{sizes:?}");
    assert!(emitted.windows(2).all(|pair| pair[0] == pair[1]));
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn array_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "arrays-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
