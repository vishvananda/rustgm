//! Fresh CLI processes re-prove version-thirteen transparent struct headers.
//! Only the representation byte travels: every consumer re-runs the
//! representation check and the one-nontrivial-field rule over its own field
//! layouts.
use super::fresh_packets::{Packet, Result, difference, relabel};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/transparent-records";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "transparent-records";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/transparent-record-outcomes.json`.
const ACCEPTED: [&str; 7] = [
    "wrap",
    "wrap-call",
    "tagged",
    "cell",
    "cell-wrap",
    "ptr",
    "outer",
];
const NONTRIVIAL: &str = "transparent aggregate has more than one nontrivial field";
const OPPONENTS: [(&str, &str); 2] = [("mismatch", "incompatible types"), ("pair", NONTRIVIAL)];
const WIDER: &str = "source nominal representation requires its wider complete contract";
const GATE: &str = "// Generated transparent gate.\n#[repr(transparent)]\npub struct Wrap { pub v: u32 }\npub fn unwrap(w: Wrap) -> u32 { w.v }\n";
const WIRE: &str = "// Generated transparent wire control.\n#[repr(transparent)]\npub struct Wrap(pub u32);\npub fn unwrap(w: Wrap) -> u32 { w.0 }\n#[repr(C)]\npub struct Two { pub a: u8, pub b: u8 }\npub fn first_byte(t: Two) -> u8 { t.a }\npub enum Mode { A, B }\n";
const TWO: &str =
    "// Generated transparent opponent.\n#[repr(transparent)]\npub struct Two(pub u8, pub u8);\n";
fn source(packet: &Packet, name: &str, text: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}
fn export(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut command = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut command).map(drop)
}
/// The provider's lines before its authored transparent section.
fn shared(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let (lines, _) = fixture
        .split_once("// Authored transparent record transport controls.")
        .ok_or("transparent record foundation")?;
    Ok(lines.to_owned())
}
fn entry(packet: &Packet, name: &str, body: &str) -> Result<std::path::PathBuf> {
    source(
        packet,
        &format!("{name}.rs"),
        &format!(
            "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\n{body}\n"
        ),
    )
}
/// Whether the byte at `offset` ends a plain nominal header row: a 10-byte
/// header contract with no `must_use` or inline advisory, then the repeated
/// declaration, scope, self type, field and input ranges, declared inputs
/// and the representation byte.
fn header_row(bytes: &[u8], offset: usize) -> bool {
    (42..bytes.len()).contains(&offset)
        && bytes[offset - 42..offset - 38] == bytes[offset - 32..offset - 28]
        && bytes[offset - 34..offset - 32] == [0, 0]
}
fn with_byte(bytes: &[u8], offset: usize, value: u8) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[offset] = value;
    changed
}
#[test]
fn transparent_records_survive_wire_and_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    Ok(())
}
#[test]
fn older_profiles_fence_transparent_records_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let provider = source(&packet, "gate.rs", &format!("{nominals}{GATE}"))?;
    for profile in OLDER {
        let metadata = packet.folder.join(format!("{profile}.rgcrate"));
        let mut command = packet.export(profile, &provider, &metadata);
        packet.rejected(profile, &mut command, WIDER)?;
    }
    let metadata = packet.folder.join("gate.rgcrate");
    export(&packet, &provider, &metadata)?;
    let client = entry(
        &packet,
        "gate-client",
        "pub extern \"C\" fn entry() -> i32 { let _v = base::unwrap(base::Wrap { v: 42 }); 42 }",
    )?;
    let mut fresh = packet.consumer(&client, "gate-client", &metadata, &packet.owner());
    super::artifacts::checked("gate-client", &packet.folder, &mut fresh).map(drop)
}
/// Exports the wire control and returns its bytes with the representation
/// offsets of `Wrap`, `Two` and `Mode`. Each struct's offset is the one byte
/// at which an otherwise identical provider with another representation
/// differs; `Mode`'s header row directly follows `Two`'s.
fn located(packet: &Packet, lines: &str) -> Result<(Vec<u8>, usize, usize, usize)> {
    let wire = format!("{lines}{WIRE}");
    let provider = source(packet, "wire.rs", &wire)?;
    export(packet, &provider, &packet.metadata)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let mut offsets = Vec::with_capacity(2);
    for (label, from, to) in [
        (
            "wire-c",
            "#[repr(transparent)]\npub struct Wrap",
            "#[repr(C)]\npub struct Wrap",
        ),
        ("wire-rust", "#[repr(C)]\npub struct Two", "pub struct Two"),
    ] {
        let variant = source(packet, &format!("{label}.rs"), &wire.replacen(from, to, 1))?;
        let metadata = packet.folder.join(format!("{label}.rgcrate"));
        export(packet, &variant, &metadata)?;
        offsets.push(difference(&bytes, &std::fs::read(&metadata)?)?);
    }
    let [wrap, pair] = <[usize; 2]>::try_from(offsets.as_slice())?;
    let mode = pair + 43;
    assert_eq!(
        (bytes[wrap], bytes[pair], bytes.get(mode)),
        (2, 1, Some(&0))
    );
    assert!(
        [wrap, pair, mode]
            .iter()
            .all(|&offset| header_row(&bytes, offset))
    );
    // An enum header has an empty field range.
    assert_eq!(bytes[mode - 20..mode - 16], bytes[mode - 16..mode - 12]);
    Ok((bytes, wrap, pair, mode))
}
#[test]
fn forged_transparent_headers_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let lines = shared(&packet)?;
    let two = source(&packet, "two.rs", &format!("{lines}{TWO}"))?;
    let metadata = packet.folder.join("two.rgcrate");
    packet.rejected(
        "two",
        &mut packet.export(PROFILE, &two, &metadata),
        NONTRIVIAL,
    )?;
    let (bytes, wrap, pair, mode) = located(&packet, &lines)?;
    let user = entry(
        &packet,
        "unwrap",
        "pub extern \"C\" fn entry() -> i32 { match base::unwrap(base::Wrap(42)) { 42 => 42, _ => 7 } }",
    )?;
    let mut honest = packet.consumer(&user, "unwrap", &packet.metadata, &packet.owner());
    super::artifacts::checked("unwrap", &packet.folder, &mut honest)?;
    for (name, wire, diagnostic) in [
        (
            "representation",
            with_byte(&bytes, wrap, 3),
            "unknown source nominal representation",
        ),
        (
            "enum-transparent",
            with_byte(&bytes, mode, 2),
            "source enum nominal header carries record fields or representation",
        ),
        ("two-transparent", with_byte(&bytes, pair, 2), NONTRIVIAL),
        (
            "version-12",
            relabel(&bytes, 12),
            "unknown source declaration kind",
        ),
    ] {
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&metadata, wire)?;
        let mut consumer = packet.consumer(&user, name, &metadata, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn transparent_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "wrappers")?;
    let lines = shared(&packet)?;
    let client = entry(
        &packet,
        "client",
        "pub extern \"C\" fn entry() -> i32 { match base::w005(42).0 { 42 => 42, _ => 7 } }",
    )?;
    let mut observed = Vec::new();
    for width in [6, 24, 96] {
        let mut wrappers = String::from("// Generated scaling control.\n");
        for index in 0..width {
            writeln!(
                wrappers,
                "#[repr(transparent)]\npub struct W{index:03}(pub u8);\npub fn w{index:03}(v: u8) -> W{index:03} {{ W{index:03}(v) }}"
            )?;
        }
        let provider = source(
            &packet,
            &format!("wrappers{width}.rs"),
            &format!("{lines}{wrappers}"),
        )?;
        let metadata = packet.folder.join(format!("wrappers{width}.rgcrate"));
        export(&packet, &provider, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let lowir = std::fs::metadata(packet.folder.join(format!("{label}.lowir")))?.len();
        observed.push((std::fs::metadata(&metadata)?.len(), lowir));
    }
    // Fixed-length names: every wrapper adds the same declaration, header,
    // constructor, recipe and original bytes, and the client's LowIR names one.
    let [(small, first), (middle, _), (large, third)] =
        <[(u64, u64); 3]>::try_from(observed.as_slice())?;
    assert_eq!(
        (middle - small) * 4,
        large - middle,
        "{small} {middle} {large}"
    );
    assert!(first.abs_diff(third) <= 64, "{first} {third}");
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn transparent_record_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
