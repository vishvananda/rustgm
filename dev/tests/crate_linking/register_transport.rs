//! Version thirteen exports inline assembly in the initial assembly profile
//! as value code 0 and the original's cold assembly columns. Fresh consumers
//! rebuild each body's payload, re-prove its unsafe context and lower one
//! template per emitted instance; every form outside the profile keeps its
//! lowering text at the exporter's preflight, older profiles keep their text,
//! and forged option bytes are rejected at decode.
use super::fresh_packets::{Packet, Result, inner, spliced, unique};
use rustgm::support::crates;
const SPEC: &str = "pa41/tests/spec/asm-transport";
const GROUP: &str = "asm-transport";
const PROFILE: &str = "module-bodies-v13";
const MARKER: &str = "// Authored asm transport section.\n";
/// Frozen outcomes from `pa41/tests/asm-transport-outcomes.json`.
const ACCEPTED: [&str; 2] = ["stored", "tagged"];
const REJECTED: [(&str, &str); 2] = [
    (
        "unsafe-free",
        "call to unsafe function requires an unsafe context",
    ),
    ("mistyped", "incompatible types"),
];
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
const OLDER_TEXT: &str = "source type form requires an explicitly supported transport row";
/// rustc requires every assembly operand type to be Copy; the foundation
/// makes only raw pointers and f32 Copy.
const COPIES: &str = "impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}\nimpl Clone for u64{fn clone(&self)->Self{*self}}impl Copy for u64{}\n";
/// The B1 opponents each generated export control carries, made generic so
/// that the provider lowers nothing; `att-syntax` cannot follow the
/// foundation. Texts come from `asm-native/opponents.tsv`.
const CONTROLS: [&str; 15] = [
    "pure",
    "nomem",
    "readonly",
    "noreturn",
    "raw",
    "output",
    "inout",
    "clobber",
    "reg-integer",
    "reg-function",
    "reg-abcd",
    "reg-byte",
    "explicit",
    "xmm-scalar",
    "modifier",
];
/// B1's template headers: `nostack` with `preserves_flags`, `nostack`
/// alone, and no options, each over one operand.
const PRESERVING: &str = " intel readwrite 1 24 0 1 {\n";
const NOSTACK: &str = " intel readwrite 1 24 7 1 {\n";
const DEFAULT: &str = " intel readwrite 13 24 7 1 {\n";

fn headers(lowir: &str) -> Vec<&str> {
    lowir
        .match_indices("assembly @asm")
        .filter_map(|(start, _)| {
            let line = &lowir[start..];
            let header = &line[..=line.find('\n')?];
            Some(&header[header.find(" intel")?..])
        })
        .collect()
}
fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
#[test]
fn assembly_packets_export_and_fresh_consumers_rebuild_their_payloads() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "asm-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // Amended after the first run: the foundation's builtin `arch::asm` row
    // needs Lane B's macro envelope (tag 14, version 2) around the
    // version-thirteen core.
    assert_eq!(bytes.get(12..14), Some([14, 2].as_slice()));
    let core = inner(&bytes)?;
    assert_eq!(bytes.get(core + 13..core + 15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    let stored = std::fs::read_to_string(packet.folder.join("stored.lowir"))?;
    assert_eq!(headers(&stored), [PRESERVING, NOSTACK]);
    assert!(stored.contains("  text \"mov dword ptr [\"\n  use 0 0 0 0\n  text \"], 40\"\n"));
    // One template per emitted instance of the generic body.
    let tagged = std::fs::read_to_string(packet.folder.join("tagged.lowir"))?;
    assert_eq!(headers(&tagged), [DEFAULT, DEFAULT]);
    assert_eq!(tagged.matches("  text \"# \"\n").count(), 2);
    for (name, diagnostic) in REJECTED {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    let source = packet.fixture("provider");
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        let mut export = packet.export(profile, &source, &older);
        packet.rejected(profile, &mut export, OLDER_TEXT)?;
        assert!(unwritten(&older)?, "{profile} wrote a packet");
    }
    Ok(())
}
#[test]
fn forms_outside_the_profile_keep_their_lowering_texts_at_export() -> Result {
    let packet = Packet::new(GROUP, SPEC, "asm-controls", "provider")?;
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = provider.split(MARKER).next().ok_or("foundation")?;
    let native = packet.root.join("pa41/tests/spec/asm-native");
    let table = std::fs::read_to_string(native.join("opponents.tsv"))?;
    for label in CONTROLS {
        let path = format!("opponents/{label}.rs");
        let expected = table
            .lines()
            .find_map(|line| line.strip_prefix(&format!("{path}\t0\t")))
            .ok_or(path.clone())?;
        let opponent = std::fs::read_to_string(native.join(&path))?;
        let lines: Vec<&str> = opponent
            .lines()
            .filter(|line| *line != "use core::arch::asm;")
            .collect();
        let body = lines.join("\n");
        assert_eq!(body.matches("pub fn f(").count(), 1, "{label}");
        let control = body
            .replace("asm!(", "crate::arch::asm!(")
            .replace("pub fn f(", "pub fn f<T>(");
        let source = packet.folder.join(format!("{label}.rs"));
        std::fs::write(&source, format!("{foundation}{COPIES}{control}\n"))?;
        let metadata = packet.folder.join(format!("{label}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        packet.rejected(label, &mut export, expected)?;
        assert!(unwritten(&metadata)?, "{label} wrote a packet");
    }
    Ok(())
}
#[test]
fn forged_option_bytes_reject_at_fresh_decode() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "asm-forged", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // `store`'s instruction row after its site: templates 0..1, operands
    // 0..1, `nostack | preserves_flags` (0x28) and an explicit unsafe block.
    let row = [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 40, 0, 1];
    let options = unique(&bytes, &row)? + 16;
    for (label, value, diagnostic) in [
        (
            "nomem",
            [0x28 | 0x02, 0],
            "inline assembly nomem option is outside the initial assembly profile",
        ),
        ("undefined", [40, 1], "unknown source assembly option"),
    ] {
        let forged = spliced(&bytes, options, &value);
        let mut client = packet.forged(label, forged, "stored")?;
        packet.rejected(label, &mut client, diagnostic)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn assembly_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "asm-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
