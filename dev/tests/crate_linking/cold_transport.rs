//! Fresh CLI processes re-prove version-thirteen cold and caller-tracking
//! header flags and the `panic` role, with core 1.98's `panicking::panic`
//! attributes. Calls that need a caller location stay fenced by name.
use super::fresh_packets::{Packet, Result, relabel, unique, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/panic-attributes";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "core-panic-path";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/panic-attribute-outcomes.json`.
const ACCEPTED: [&str; 3] = ["check", "role", "rare"];
const TRACKED: [&str; 2] = ["traced", "method"];
const LOCATION: &str = "imported caller tracking requires its provider's transported Location";
const OPPONENTS: [(&str, &str); 2] = [
    ("panic-type", "incompatible types"),
    ("rare-arity", "call argument count mismatch"),
];
const COLD: &str = "// Generated cold control.\n#[cold]\npub fn rare(x: u8) -> u8 { x }\n";
const TRACKING: &str =
    "// Generated tracking control.\n#[track_caller]\npub fn traced(x: u8) -> u8 { x }\n";
const ROLE: &str = "// Generated role control.\n#[lang = \"panic\"]\npub const fn panic(expr: &'static str) -> ! { loop {} }\n";
const PENDING: &str = "// Generated pending role control.\n#[lang = \"panic\"]\npub const fn panic(code: u8) -> u8 { code }\n";
const WIRE: &str = "// Generated panic-attribute wire control.\npub mod panicking {\n    #[lang = \"panic\"]\n    pub const fn panic(expr: &'static str) -> ! { loop {} }\n}\n#[cold]\npub fn rare(x: u8) -> u8 { x }\npub fn relay(x: u8) -> u8 { rare(x) }\n#[track_caller]\npub fn traced(x: u8) -> u8 { x }\n";
/// The `panic` role row: escape 255 and the spelling's length and bytes.
const SPELLED: &[u8] = b"\xff\x05\x00\x00\x00panic";
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
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
fn foundation(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    Ok(fixture
        .split("// Authored panic-attributes transport controls.")
        .next()
        .ok_or("attribute foundation")?
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
fn cold_tracking_and_panic_role_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    unique(&bytes, SPELLED)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for name in TRACKED {
        packet.rejected(name, &mut packet.client(name), LOCATION)?;
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
fn older_profiles_fence_cold_tracking_and_panic_rows_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    for (label, control, diagnostic) in [
        (
            "cold",
            COLD,
            "source cold advisory marker transport is not implemented",
        ),
        (
            "tracking",
            TRACKING,
            "source caller tracking policy transport is not implemented",
        ),
        (
            "role",
            ROLE,
            "source panic role requires the version-thirteen role profile",
        ),
    ] {
        let source = generated(
            &packet,
            &format!("{label}.rs"),
            &format!("{nominals}{control}"),
        )?;
        for profile in OLDER {
            let older = packet.folder.join(format!("{label}-{profile}.rgcrate"));
            packet.rejected(
                &format!("{label}-{profile}"),
                &mut packet.export(profile, &source, &older),
                diagnostic,
            )?;
        }
        exported(
            &packet,
            &source,
            &packet.folder.join(format!("{label}.rgcrate")),
        )?;
    }
    // A nonconforming role stays pending and keeps the earlier diagnostic.
    let source = generated(&packet, "pending.rs", &format!("{nominals}{PENDING}"))?;
    let pending = packet.folder.join("pending.rgcrate");
    packet.rejected(
        "pending",
        &mut packet.export(PROFILE, &source, &pending),
        "source trait role requires completed declaration validation",
    )
}
/// The three consecutive function header rows of `rare`, `relay` and
/// `traced`: declaration, site, no `must_use`, then the flag byte.
fn flag_rows(bytes: &[u8]) -> Result<usize> {
    let rows: Vec<usize> = (0..bytes.len().saturating_sub(30))
        .filter(|&offset| {
            let declaration = bytes[offset];
            let declarations = [
                declaration,
                declaration.wrapping_add(1),
                declaration.wrapping_add(2),
            ];
            (0..3).all(|row| {
                let start = offset + row * 10;
                bytes[start] == declarations[row]
                    && bytes[start + 1..start + 4] == [0, 0, 0]
                    && bytes[start + 8] == 0
                    && bytes[start + 9] == [2, 0, 4][row]
            })
        })
        .collect();
    let [offset] = rows.as_slice() else {
        return Err(format!("ambiguous header flag rows {rows:?}").into());
    };
    Ok(*offset)
}
type Forgery = (&'static str, &'static str, Vec<u8>, &'static str);
fn forgeries(bytes: &[u8]) -> Result<(Vec<Forgery>, Vec<u8>)> {
    let rows = flag_rows(bytes)?;
    let rare = u32::from(bytes[rows]);
    let role = unique(bytes, SPELLED)? + SPELLED.len() + 4;
    let flagged = |flags: u8| {
        let mut changed = bytes.to_vec();
        changed[rows + 9] = flags;
        changed
    };
    let mut untracked = bytes.to_vec();
    untracked[rows + 29] = 0;
    Ok((
        vec![
            (
                "unknown-flags",
                "relay",
                flagged(8),
                "unknown transported inline or callable advisory flags",
            ),
            (
                "tracked-callee",
                "relay",
                flagged(6),
                "tracked call lacks its actual sealed caller site",
            ),
            ("tracked-client", "rare", flagged(6), LOCATION),
            (
                "role-retarget",
                "relay",
                with_id(bytes, role, rare),
                "source panic role requires a const declaration with never result",
            ),
            (
                "version-12",
                "relay",
                relabel(bytes, 12),
                "unknown source declaration kind",
            ),
        ],
        untracked,
    ))
}
#[test]
fn forged_advisory_and_role_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let foundation = foundation(&packet)?;
    let source = generated(&packet, "wire.rs", &format!("{foundation}{WIRE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let clients = [
        ("relay", "match base::relay(42) { 42 => 42, _ => 7 }"),
        ("rare", "match base::rare(42) { 42 => 42, _ => 7 }"),
        ("traced", "match base::traced(42) { 42 => 42, _ => 7 }"),
    ];
    for (name, body) in clients {
        client(&packet, name, body)?;
    }
    let path = |name: &str| packet.folder.join(format!("{name}.rs"));
    for name in ["relay", "rare"] {
        let mut honest = packet.consumer(&path(name), name, &packet.metadata, &packet.owner());
        super::artifacts::checked(name, &packet.folder, &mut honest)?;
    }
    let mut traced = packet.consumer(&path("traced"), "traced", &packet.metadata, &packet.owner());
    packet.rejected("traced", &mut traced, LOCATION)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let (forged, untracked) = forgeries(&bytes)?;
    for (name, user, wire, diagnostic) in forged {
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&metadata, wire)?;
        let mut consumer = packet.consumer(&path(user), name, &metadata, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // Sound: the consumer re-lowers `traced` itself, and no transported body
    // can read the location the cleared flag drops.
    let metadata = packet.folder.join("untracked.rgcrate");
    std::fs::write(&metadata, untracked)?;
    let mut consumer = packet.consumer(&path("traced"), "untracked", &metadata, &packet.owner());
    super::artifacts::checked("untracked", &packet.folder, &mut consumer)?;
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn advisory_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "advisories")?;
    let foundation = foundation(&packet)?;
    let consumer = client(
        &packet,
        "client",
        "match base::rare(42) { 42 => 42, _ => 7 }",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut functions = String::new();
        for index in 0..width {
            writeln!(
                functions,
                "#[cold]\n#[track_caller]\npub fn t{index:03}(x: u8) -> u8 {{ x }}"
            )?;
        }
        let source = generated(
            &packet,
            &format!("advisories{width}.rs"),
            &format!("{foundation}{WIRE}{functions}"),
        )?;
        let metadata = packet.folder.join(format!("advisories{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&consumer, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every function adds the same declaration, header,
    // flag row and body bytes.
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
fn cold_and_panic_role_clients_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
