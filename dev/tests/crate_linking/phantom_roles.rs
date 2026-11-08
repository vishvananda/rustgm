//! Fresh CLI processes re-prove version-thirteen `PhantomData` roles. Only the
//! spelled role row travels: every consumer re-proves the named unit struct
//! and derives its formal's variance, and rejects any other imported formal
//! that comes out bivariant.
use super::fresh_packets::{Packet, Result, difference, relabel, unique};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/phantom-roles";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "phantom-roles";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/phantom-role-outcomes.json`.
const ACCEPTED: [&str; 6] = [
    "value",
    "marker",
    "marked",
    "marked-call",
    "shrink",
    "local",
];
const LIFETIME: &str = "required lifetime does not outlive its target";
const OPPONENTS: [(&str, &str); 3] = [
    ("grow", LIFETIME),
    ("marked-grow", LIFETIME),
    ("mismatch", "incompatible types"),
];
/// Older profiles reject every unit struct, `PhantomData` included, by its
/// constructor declaration before any role is exported.
const UNIT: &str = "source declaration kind is outside the initial source profile";
const INTEGRITY: &str = "imported source nominal has a bivariant formal";
const GATE: &str = "// Generated PhantomData gate.\n#[lang = \"phantom_data\"]\npub struct PhantomData<T: PointeeSized>;\n";
const WIRE: &str = "// Generated PhantomData wire control.\n#[lang = \"phantom_data\"]\npub struct PhantomData<T: PointeeSized>;\npub struct Held<T> { pub held_byte: u8, pub held_mark: PhantomData<T> }\npub struct Unit;\n#[repr(C)]\npub struct Two { pub a: u8, pub b: u8 }\npub enum Mode { A, B }\n";
/// The spelled role row: tag 255, the spelling's length and its bytes.
const ROLE: &[u8] = b"\xff\x0c\x00\x00\x00phantom_data";
/// A spelled row's tag, length, spelling and declaration address.
const ROLE_ROW: usize = ROLE.len() + 8;
/// A plain nominal header row, ending in its representation byte.
const HEADER_ROW: usize = 43;
fn source(packet: &Packet, name: &str, text: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}
fn export(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut command = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut command).map(drop)
}
/// The provider's lines before its authored `PhantomData` section.
fn shared(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let (lines, _) = fixture
        .split_once("// Authored PhantomData role transport controls.")
        .ok_or("PhantomData role foundation")?;
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
/// The declaration that a plain nominal header row, ending at `offset`, names
/// in its contract and repeats in its header.
fn declaration(bytes: &[u8], offset: usize) -> Result<[u8; 4]> {
    let contract = bytes
        .get(offset + 1 - HEADER_ROW..offset + 5 - HEADER_ROW)
        .ok_or("header row")?;
    let header = bytes.get(offset - 32..offset - 28).ok_or("header row")?;
    if contract != header {
        return Err("not a plain nominal header row".into());
    }
    Ok(<[u8; 4]>::try_from(contract)?)
}
fn spliced(bytes: &[u8], offset: usize, value: &[u8]) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[offset..offset + value.len()].copy_from_slice(value);
    changed
}
/// The offsets of a role column's count and of each row, if `count` is one
/// at `offset`. Compact rows are a tag and an address; spelled rows add the
/// spelling.
fn role_rows(bytes: &[u8], offset: usize) -> Option<Vec<usize>> {
    let count = u32::from_le_bytes(bytes.get(offset..offset + 4)?.try_into().ok()?);
    if !(1..=64).contains(&count) {
        return None;
    }
    let mut rows = Vec::new();
    let mut cursor = offset + 4;
    for _ in 0..count {
        rows.push(cursor);
        cursor += match *bytes.get(cursor)? {
            0..=13 => 9,
            255 => {
                let length =
                    u32::from_le_bytes(bytes.get(cursor + 1..cursor + 5)?.try_into().ok()?);
                if !(1..=64).contains(&length) {
                    return None;
                }
                13 + length as usize
            }
            _ => return None,
        };
    }
    (cursor <= bytes.len()).then_some(rows)
}
/// The role column containing `role`: its count's offset and its row
/// offsets. Scanning upward, the first count that parses through the row
/// starts the whole column; later ones start inside it.
fn role_column(bytes: &[u8], role: usize) -> Result<(usize, Vec<usize>)> {
    (role.saturating_sub(4096)..role)
        .find_map(|offset| {
            role_rows(bytes, offset)
                .filter(|rows| rows.contains(&role))
                .map(|rows| (offset, rows))
        })
        .ok_or_else(|| "role column".into())
}
/// The honest packet with the `PhantomData` role row removed, the role column's
/// count reduced and the policy's role range, the first `[0, 0, count]`
/// triple of provider, start and end, shortened to match.
fn without_role(bytes: &[u8], role: usize) -> Result<Vec<u8>> {
    let (count, rows) = role_column(bytes, role)?;
    let total = u32::try_from(rows.len())?;
    let policy = [0, 0, total]
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect::<Vec<u8>>();
    let range = bytes
        .windows(policy.len())
        .position(|window| window == policy)
        .filter(|&offset| offset < count)
        .ok_or("policy role range")?;
    let shorter = (total - 1).to_le_bytes();
    let mut changed = spliced(&spliced(bytes, count, &shorter), range + 8, &shorter);
    changed.drain(role..role + ROLE_ROW);
    Ok(changed)
}
#[test]
fn phantom_roles_survive_wire_and_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    unique(&bytes, ROLE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    Ok(())
}
#[test]
fn older_profiles_fence_phantom_roles_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let provider = source(&packet, "gate.rs", &format!("{nominals}{GATE}"))?;
    for profile in OLDER {
        let metadata = packet.folder.join(format!("{profile}.rgcrate"));
        let mut command = packet.export(profile, &provider, &metadata);
        packet.rejected(profile, &mut command, UNIT)?;
    }
    let metadata = packet.folder.join("gate.rgcrate");
    export(&packet, &provider, &metadata)?;
    let client = entry(
        &packet,
        "gate-client",
        "pub extern \"C\" fn entry() -> i32 { let _m: base::PhantomData<u8> = base::PhantomData; 42 }",
    )?;
    let mut fresh = packet.consumer(&client, "gate-client", &metadata, &packet.owner());
    super::artifacts::checked("gate-client", &packet.folder, &mut fresh).map(drop)
}
/// A found false rejection: with no lifetime-generic nominal imported, a fresh
/// consumer prepared no body storage, so a unit value of a type-generic one
/// could not be lowered. The `PhantomData` gate client is the other witness.
#[test]
fn imported_generic_unit_values_lower_without_lifetime_nominals() -> Result {
    let packet = &Packet::new(GROUP, SPEC, "units", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let provider = source(
        packet,
        "opt.rs",
        &format!(
            "{nominals}// Generated generic unit-variant control.\npub enum Opt<T> {{ None, Some(T) }}\n"
        ),
    )?;
    let metadata = packet.folder.join("opt.rgcrate");
    export(packet, &provider, &metadata)?;
    let client = entry(
        packet,
        "opt-client",
        "pub extern \"C\" fn entry() -> i32 { let _o: base::Opt<u8> = base::Opt::None; 42 }",
    )?;
    let mut fresh = packet.consumer(&client, "opt-client", &metadata, &packet.owner());
    super::artifacts::checked("opt-client", &packet.folder, &mut fresh).map(drop)
}
/// The wire control's bytes, its role row and the representation offsets of
/// its nominal headers, which follow one another in declaration order. `Two`'s
/// is the one byte at which a provider without its `repr(C)` differs.
struct Located {
    bytes: Vec<u8>,
    role: usize,
    unit: [u8; 4],
    two: [u8; 4],
    mode: [u8; 4],
}
fn located(packet: &Packet, lines: &str) -> Result<Located> {
    let wire = format!("{lines}{WIRE}");
    let provider = source(packet, "wire.rs", &wire)?;
    export(packet, &provider, &packet.metadata)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let variant = source(
        packet,
        "wire-rust.rs",
        &wire.replacen("#[repr(C)]\npub struct Two", "pub struct Two", 1),
    )?;
    let metadata = packet.folder.join("wire-rust.rgcrate");
    export(packet, &variant, &metadata)?;
    let two = difference(&bytes, &std::fs::read(&metadata)?)?;
    let [phantom, held, unit, pair, mode] =
        [4, 3, 2, 1, 0].map(|rows| two + HEADER_ROW - rows * HEADER_ROW);
    assert_eq!(
        [phantom, held, unit, pair, mode].map(|offset| bytes[offset]),
        [0, 0, 0, 1, 0]
    );
    let role = unique(&bytes, ROLE)?;
    let address = bytes
        .get(role + ROLE.len()..role + ROLE_ROW)
        .ok_or("role address")?;
    assert_eq!(address[..4], [0; 4]);
    assert_eq!(address[4..], declaration(&bytes, phantom)?);
    declaration(&bytes, held)?;
    Ok(Located {
        unit: declaration(&bytes, unit)?,
        two: declaration(&bytes, pair)?,
        mode: declaration(&bytes, mode)?,
        bytes,
        role,
    })
}
#[test]
fn forged_phantom_roles_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let lines = shared(&packet)?;
    let wire = located(&packet, &lines)?;
    let bytes = &wire.bytes;
    let user = entry(
        &packet,
        "value",
        "pub extern \"C\" fn entry() -> i32 { let _m: base::PhantomData<u8> = base::PhantomData; 42 }",
    )?;
    let mut honest = packet.consumer(&user, "value", &packet.metadata, &packet.owner());
    super::artifacts::checked("value", &packet.folder, &mut honest)?;
    let target = wire.role + ROLE.len() + 4;
    let spelling = wire.role + 5;
    let byte = unique(bytes, b"\x09\x00\x00\x00held_byte")?;
    let mark = unique(bytes, b"\x09\x00\x00\x00held_mark")?;
    let byte_type = bytes.get(byte + 13..byte + 17).ok_or("held_byte type")?;
    for (name, forged, diagnostic) in [
        (
            "unit",
            spliced(bytes, target, &wire.unit),
            "source PhantomData requires one actual type formal",
        ),
        (
            "two",
            spliced(bytes, target, &wire.two),
            "imported PhantomData requires its module-owned unit structure",
        ),
        (
            "mode",
            spliced(bytes, target, &wire.mode),
            "imported declaration changed its exact identity or kind",
        ),
        (
            "maybe-uninit",
            spliced(bytes, spelling, b"maybe_uninit"),
            "ordinary source roles cannot claim a nominal declaration",
        ),
        (
            "unknown",
            spliced(bytes, spelling, b"phantom_dat_"),
            "unknown source language role spelling",
        ),
        ("deleted", without_role(bytes, wire.role)?, INTEGRITY),
        ("unused", spliced(bytes, mark + 13, byte_type), INTEGRITY),
        (
            "version-12",
            relabel(bytes, 12),
            "unknown source declaration kind",
        ),
    ] {
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&metadata, forged)?;
        let mut consumer = packet.consumer(&user, name, &metadata, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn phantom_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "records")?;
    let lines = shared(&packet)?;
    let client = entry(
        &packet,
        "client",
        "pub extern \"C\" fn entry() -> i32 { match base::m005::<u8>(42).v { 42 => 42, _ => 7 } }",
    )?;
    let mut observed = Vec::new();
    for width in [6, 24, 96] {
        let mut records = String::from(
            "// Generated scaling control.\n#[lang = \"phantom_data\"]\npub struct PhantomData<T: PointeeSized>;\n",
        );
        for index in 0..width {
            writeln!(
                records,
                "pub struct M{index:03}<T> {{ pub v: u8, pub m: PhantomData<T> }}\npub fn m{index:03}<T>(v: u8) -> M{index:03}<T> {{ M{index:03} {{ v, m: PhantomData }} }}"
            )?;
        }
        let provider = source(
            &packet,
            &format!("records{width}.rs"),
            &format!("{lines}{records}"),
        )?;
        let metadata = packet.folder.join(format!("records{width}.rgcrate"));
        export(&packet, &provider, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let lowir = std::fs::metadata(packet.folder.join(format!("{label}.lowir")))?.len();
        observed.push((std::fs::metadata(&metadata)?.len(), lowir));
    }
    // Fixed-length names: every record adds the same declaration, header,
    // fields, constructor and original bytes, and the client names one.
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
fn phantom_role_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
