//! Fresh CLI processes re-prove version-thirteen writes and mutable reborrows
//! through a unique reference field of an immutable owning binding, as rustc
//! admits them; older profiles keep main's gate by name.
use super::fresh_packets::{Packet, Result, relabel, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/unique-writes";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "unique-writes";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/unique-write-outcomes.json`.
const ACCEPTED: [&str; 6] = ["bump", "after", "lend", "set", "put", "deep"];
const OPPONENTS: [(&str, &str); 2] = [
    ("shared-path", "cannot assign through a shared reference"),
    ("field-write", "cannot assign through an immutable local"),
];
const OWN: &str = "// Generated unique-write gate.\npub struct Own { pub r: &'static mut u8 }\npub fn poke(o: Own) { *o.r = 42; }\n";
const WIRE: &str = "// Generated unique-write wire control.\npub struct Count<'a> { pub z: u8, pub n: &'a mut u8 }\npub fn poke(c: Count<'_>) { *c.n = 42; }\n";
const POKE: &str =
    "let mut n = 7u8; base::poke(base::Count { z: 0, n: &mut n }); match n { 42 => 42, _ => 7 }";
fn generated(packet: &Packet, name: &str, source: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, source)?;
    Ok(path)
}
fn exported(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut export = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut export).map(drop)
}
fn foundation(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    Ok(fixture
        .split("// Authored unique-path write controls.")
        .next()
        .ok_or("unique-write foundation")?
        .to_owned())
}
fn client(packet: &Packet, name: &str, body: &str) -> Result<std::path::PathBuf> {
    generated(
        packet,
        &format!("{name}.rs"),
        &format!(
            "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {{ {body} }}\n"
        ),
    )
}
#[test]
fn unique_path_writes_survive_wire_and_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
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
fn older_profiles_keep_the_immutable_write_gate_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let source = generated(&packet, "own.rs", &format!("{nominals}{OWN}"))?;
    for profile in OLDER {
        let older = packet.folder.join(format!("own-{profile}.rgcrate"));
        packet.rejected(
            &format!("own-{profile}"),
            &mut packet.export(profile, &source, &older),
            "source operation omitted a required nontrivial formation input",
        )?;
    }
    exported(&packet, &source, &packet.folder.join("own.rgcrate"))
}
fn word(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
fn unique_offset(found: &[usize], what: &str) -> Result<usize> {
    match found {
        [offset] => Ok(*offset),
        _ => Err(format!("{what} {found:?}").into()),
    }
}
/// `poke`'s two body-IR projection rows: the field `c.n` (a local place
/// base, field node 3 with declaration and ordinal-1 operands) and its mutable
/// dereference (projection-0 place base, dereference node 1 with region and
/// mutability operands). Returns the offset of the dereference's mutability.
fn poke_dereference(bytes: &[u8]) -> Result<usize> {
    let found: Vec<usize> = (0..bytes.len())
        .filter(|&offset| {
            let dereference = offset + 21;
            bytes.get(offset) == Some(&11)
                && bytes.get(offset + 9..offset + 12) == Some([3, 2, 5].as_slice())
                && bytes.get(offset + 16) == Some(&8)
                && word(bytes, offset + 17) == Some(1)
                && bytes.get(dereference) == Some(&12)
                && word(bytes, dereference + 1) == Some(0)
                && bytes.get(dereference + 9..dereference + 12) == Some([1, 2, 7].as_slice())
                && bytes.get(dereference + 16) == Some(&8)
                && word(bytes, dereference + 17) == Some(1)
        })
        .collect();
    unique_offset(&found, "unique-write dereference rows").map(|offset| offset + 38)
}
/// `poke`'s Store statement node (11, two operands) into the projection-1
/// place `*c.n`. Returns the offset of its place index.
fn poke_store(bytes: &[u8]) -> Result<usize> {
    let found: Vec<usize> = (0..bytes.len())
        .filter(|&offset| {
            bytes.get(offset..offset + 3) == Some([11, 2, 12].as_slice())
                && word(bytes, offset + 3) == Some(1)
                && bytes.get(offset + 7) == Some(&0)
        })
        .collect();
    unique_offset(&found, "unique-write Store statements").map(|offset| offset + 3)
}
#[test]
fn forged_unique_write_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let foundation = foundation(&packet)?;
    let source = generated(&packet, "wire.rs", &format!("{foundation}{WIRE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let user = client(&packet, "poke", POKE)?;
    let mut honest = packet.consumer(&user, "poke", &packet.metadata, &packet.owner());
    super::artifacts::checked("poke", &packet.folder, &mut honest)?;
    let bytes = std::fs::read(&packet.metadata)?;
    for (name, wire, diagnostic) in [
        (
            "owning-field",
            with_id(&bytes, poke_store(&bytes)?, 0),
            "source Store writes through an immutable dereference",
        ),
        (
            "shared-dereference",
            with_id(&bytes, poke_dereference(&bytes)?, 0),
            "source dereference changed its referent, lifetime or mutability",
        ),
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
fn unique_write_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "writes")?;
    let foundation = foundation(&packet)?;
    let consumer = client(
        &packet,
        "client",
        "let mut n = 7u8; base::w005(base::Count { n: &mut n }); match n { 42 => 42, _ => 7 }",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut writes = String::new();
        for index in 0..width {
            writeln!(writes, "pub fn w{index:03}(c: Count<'_>) {{ *c.n = 42; }}")?;
        }
        let source = generated(
            &packet,
            &format!("writes{width}.rs"),
            &format!(
                "{foundation}// Generated scaling control.\npub struct Count<'a> {{ pub n: &'a mut u8 }}\n{writes}"
            ),
        )?;
        let metadata = packet.folder.join(format!("writes{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&consumer, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every function adds the same declaration, header
    // and body bytes.
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
fn unique_path_write_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
