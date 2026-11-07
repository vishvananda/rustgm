//! Fresh CLI processes re-prove version-thirteen `&'static str` and `!` type
//! rows, static text literals and plain reborrow inputs, and run the shapes of
//! core's `panicking::panic` and `panicking::assert_failed`.
use super::fresh_packets::{Packet, Result, relabel, unique, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/static-str-never";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "static-str-never";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/static-str-never-outcomes.json`.
const ACCEPTED: [&str; 8] = [
    "check", "untaken", "arm", "name", "literal", "spin", "expect", "pick",
];
const OPPONENTS: [(&str, &str); 3] = [
    ("name-type", "incompatible types"),
    ("panic-arity", "call argument count mismatch"),
    ("str-method", "receiver has no inherent nominal method"),
];
const NAME: &str =
    "// Generated older-profile str control.\npub fn name() -> &'static str { \"base\" }\n";
const SPIN: &str = "// Generated older-profile never control.\npub fn spin() -> ! { loop {} }\n";
const WIRE: &str = "// Generated text wire control.\npub fn name() -> &'static str { \"base\" }\npub fn pick(first: bool) -> &'static str { if first { \"first\" } else { \"second\" } }\n";
/// Body IR value node 7 (`Text`) with one Flag operand: `pick`'s second literal.
const SECOND_TEXT: [u8; 7] = [7, 1, 8, 1, 0, 0, 0];
/// A plain reborrow's tail: no unsizing, one dereference, not two-phase.
const REBORROW: [u8; 8] = [0, 1, 1, 0, 0, 0, 0, 0];
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
fn foundation(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    Ok(fixture
        .split("// Authored static-str-never transport controls.")
        .next()
        .ok_or("text foundation")?
        .to_owned())
}
#[test]
fn static_text_and_never_survive_wire_and_fresh_consumers() -> Result {
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
fn older_profiles_fence_str_and_never_rows_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    for (label, control, diagnostic) in [
        (
            "name",
            NAME,
            "source scalar type is outside the supported profile",
        ),
        (
            "spin",
            SPIN,
            "source type form requires an explicitly supported transport row",
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
    }
    Ok(())
}
/// Same-length wire forgeries of the generated `name` and `pick` packet,
/// with their consumer diagnostics, and the sound rewritten literal.
type Forgery = (&'static str, Vec<u8>, &'static str);
fn forgeries(bytes: &[u8]) -> Result<(Vec<Forgery>, Vec<u8>)> {
    let second = unique(bytes, &SECOND_TEXT)?;
    let literal = unique(bytes, b"\x06\x00\x00\x00second")? + 4;
    // The one `str` row (tag 10) is followed by its `&'static str` row
    // (tag 2, shared), which `pick`'s literals reborrow.
    let text = unique(bytes, &[10, 2, 0])?;
    let reference = word(bytes, text + 7)? + 1;
    let reborrows: Vec<usize> = (0..bytes.len().saturating_sub(17))
        .filter(|&offset| {
            bytes[offset] == 0
                && word(bytes, offset + 1).ok() == Some(reference)
                && bytes[offset + 9..offset + 17] == REBORROW
        })
        .collect();
    assert_eq!(reborrows.len(), 2, "{reborrows:?}");
    let mut utf8 = bytes.to_vec();
    utf8[literal] = 0xFF;
    let mut never = bytes.to_vec();
    never[text] = 11;
    let (mut two_phase, mut underef) = (bytes.to_vec(), bytes.to_vec());
    for &row in &reborrows {
        two_phase[row + 16] = 1;
        underef = with_id(&underef, row + 11, 0);
    }
    let mut rewritten = bytes.to_vec();
    rewritten[literal + 1] = b'x';
    Ok((
        vec![
            (
                "text-ordinal",
                with_id(bytes, second + 3, 0),
                "source text ordinal leaves its value order",
            ),
            ("text-utf8", utf8, "source text literal is not UTF-8"),
            (
                "str-never",
                never,
                "source text value is not a static str reference",
            ),
            (
                "two-phase",
                two_phase,
                "source adjusted coercion requires a wider source profile",
            ),
            (
                "no-dereference",
                underef,
                "source adjusted/unsizing coercion requires a complete wider input profile",
            ),
            (
                "version-12",
                relabel(bytes, 12),
                "unknown source declaration kind",
            ),
        ],
        rewritten,
    ))
}
#[test]
fn forged_text_and_reborrow_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let foundation = foundation(&packet)?;
    let source = generated(&packet, "wire.rs", &format!("{foundation}{WIRE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let client = packet.fixture("pick");
    let mut honest = packet.consumer(&client, "honest", &packet.metadata, &packet.owner());
    super::artifacts::checked("honest", &packet.folder, &mut honest)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let (forged, rewritten) = forgeries(&bytes)?;
    for (name, wire, diagnostic) in forged {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let mut consumer = packet.consumer(&client, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // Sound: another valid literal of the same length is another program.
    let path = packet.folder.join("rewritten.rgcrate");
    std::fs::write(&path, rewritten)?;
    let mut consumer = packet.consumer(&client, "rewritten", &path, &packet.owner());
    super::artifacts::checked("rewritten", &packet.folder, &mut consumer)?;
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn text_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "texts")?;
    let foundation = foundation(&packet)?;
    let client = generated(
        &packet,
        "client.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { let p = base::t005(true) as *const str as *const u8; match unsafe { *p } { 116 => 42, _ => 7 } }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut texts = String::new();
        for index in 0..width {
            writeln!(
                texts,
                "pub fn t{index:03}(ok: bool) -> &'static str {{ if ok {{ \"t{index:03}\" }} else {{ panicking::panic(\"f{index:03}\") }} }}"
            )?;
        }
        let source = generated(
            &packet,
            &format!("texts{width}.rs"),
            &format!(
                "{foundation}// Generated scaling control.\npub mod panicking {{ pub fn panic(expr: &'static str) -> ! {{ loop {{}} }} }}\n{texts}"
            ),
        )?;
        let metadata = packet.folder.join(format!("texts{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names and literals: every function adds the same
    // declaration, header, body, input and text-column bytes.
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
fn static_text_and_never_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
