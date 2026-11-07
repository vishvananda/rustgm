//! Fresh CLI processes re-prove version-thirteen lifetime-parameterized
//! structs and enums: leading nominal region slots, fields that name them in
//! their own frame, and early nominal region arguments. Each consumer derives
//! variance and implied outlives from the transported fields itself.
use super::fresh_packets::{Packet, Result, relabel, unique};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/lifetime-nominals";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "lifetime-nominal-transport";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/lifetime-nominal-outcomes.json`.
const ACCEPTED: [&str; 9] = [
    "args",
    "shrink",
    "first",
    "field",
    "fixed",
    "invariant",
    "reference",
    "text",
    "word",
];
/// Region rows are interned, so a forged row that a nominal's own Self recipe
/// also names is caught by the Self identity check first.
const SELF_RECIPE: &str = "source nominal Self changed its exact formal identity or argument order";
const OUTLIVES: &str = "required lifetime does not outlive its target";
const OPPONENTS: [&str; 5] = ["inv-shrink", "args-grow", "text-grow", "escape", "outlives"];
const GATE: &str = "// Generated lifetime gate.\npub struct Gate<'a> { pub text: &'a str }\n";
const BOUNDED: &str = "// Generated lifetime-bound gate.\npub struct Bounded<'a, 'b: 'a> { pub a: &'a u8, pub b: &'b u8 }\n";
/// Inherent impl methods that capture lifetimes transport since the
/// impl-region-methods checkpoint; a trait impl's captured lifetimes do not.
const HELD: &str = "// Generated captured-lifetime gate.\npub struct Held<'a> { pub text: &'a str }\npub trait Get { fn get(self) -> u8; }\nimpl<'a> Get for Held<'a> { fn get(self) -> u8 { 42 } }\n";
const WIRE: &str = "// Generated lifetime-nominal wire control.\npub struct Zinv<'a> { pub slot: &'a mut &'a u8 }\npub fn store<'a>(mut z: Zinv<'a>, v: &'a u8) -> u8 { *z.slot = v; **z.slot }\npub struct Zarg<'a> { pub text: &'a str }\npub fn take(a: Zarg<'_>) -> u8 { 42 }\n";
/// `Zinv`'s region slot row (owner 60, ordinal and index 0, definition 80)
/// and the start of `store`'s.
const ZINV_SLOT: &[u8] = &[
    60, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 80, 0, 0, 0, 1, 61, 0, 0, 0,
];
/// The first two region rows: `Parameter { slot: 38, depth: 0 }` for `Zinv`
/// and `Parameter { slot: 40, depth: 0 }` for `Zarg`.
const PARAMETERS: &[u8] = &[2, 38, 0, 0, 0, 0, 0, 2, 40, 0, 0, 0, 0, 0];
/// `Zinv`'s field `&'a mut &'a u8`: types 22 and 23 over region 0.
const FIELD: &[u8] = &[2, 0, 0, 0, 0, 0, 21, 0, 0, 0, 2, 1, 0, 0, 0, 0, 22, 0, 0, 0];
/// `str` and `Zarg`'s field `&'a str` over region 1.
const TEXT: &[u8] = &[10, 2, 0, 1, 0, 0, 0, 25, 0, 0, 0];
/// The nominal region arguments of `store`'s and `take`'s original and
/// callable signatures; the last is `take`'s callable `Zarg<'_>`.
const ARGUMENTS: &[u8] = &[
    1, 18, 0, 0, 0, 1, 19, 0, 0, 0, 1, 20, 0, 0, 0, 1, 21, 0, 0, 0,
];
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
        .split("// Authored lifetime-nominal transport controls.")
        .next()
        .ok_or("lifetime foundation")?
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
fn lifetime_nominals_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for name in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), OUTLIVES)?;
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
fn older_profiles_and_unsupported_lifetime_shapes_reject_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let source = generated(&packet, "gate.rs", &format!("{nominals}{GATE}"))?;
    for profile in OLDER {
        let older = packet.folder.join(format!("gate-{profile}.rgcrate"));
        packet.rejected(
            &format!("gate-{profile}"),
            &mut packet.export(profile, &source, &older),
            "source lifetime binding has no actual original slot",
        )?;
    }
    exported(&packet, &source, &packet.folder.join("gate.rgcrate"))?;
    for (label, control, diagnostic) in [
        (
            "bounded",
            BOUNDED,
            "source nominal lifetime bounds require their wider profile",
        ),
        (
            "held",
            HELD,
            "source function early/captured callable frame transport is not implemented",
        ),
    ] {
        let source = generated(
            &packet,
            &format!("{label}.rs"),
            &format!("{nominals}{control}"),
        )?;
        let metadata = packet.folder.join(format!("{label}.rgcrate"));
        packet.rejected(
            label,
            &mut packet.export(PROFILE, &source, &metadata),
            diagnostic,
        )?;
    }
    Ok(())
}
fn changed(bytes: &[u8], offset: usize, values: &[u8]) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[offset..offset + values.len()].copy_from_slice(values);
    changed
}
type Forgery = (&'static str, &'static str, Vec<u8>, &'static str);
fn forgeries(bytes: &[u8]) -> Result<Vec<Forgery>> {
    let slot = unique(bytes, ZINV_SLOT)?;
    let regions = unique(bytes, PARAMETERS)?;
    let field = unique(bytes, FIELD)?;
    let text = unique(bytes, TEXT)?;
    let arguments = unique(bytes, ARGUMENTS)?;
    Ok(vec![
        (
            "depth",
            "store",
            changed(bytes, regions + 5, &[1, 0]),
            SELF_RECIPE,
        ),
        (
            "slot-index",
            "store",
            changed(bytes, slot + 8, &[1]),
            "source nominal region slot changed its leading declared index",
        ),
        (
            "foreign-slot",
            "take",
            changed(bytes, regions + 8, &[38]),
            SELF_RECIPE,
        ),
        (
            "foreign-field",
            "take",
            changed(bytes, text + 3, &[0]),
            "source consuming type root captures another parameter owner",
        ),
        (
            "argument-kind",
            "take",
            changed(bytes, arguments + 15, &[0]),
            "source nominal argument changed its formal kind",
        ),
        (
            "shared-slot",
            "store",
            changed(bytes, field + 11, &[0]),
            "source projected field type differs after actual nominal substitution",
        ),
        (
            "version-12",
            "take",
            relabel(bytes, 12),
            "unknown source declaration kind",
        ),
    ])
}
#[test]
fn forged_lifetime_nominal_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let foundation = foundation(&packet)?;
    let source = generated(&packet, "wire.rs", &format!("{foundation}{WIRE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let clients = [
        (
            "store",
            "let a: u8 = 7; let b: u8 = 42; let mut r: &u8 = &a; match base::store(base::Zinv { slot: &mut r }, &b) { 42 => 42, _ => 7 }",
        ),
        (
            "take",
            "match base::take(base::Zarg { text: \"x\" }) { 42 => 42, _ => 7 }",
        ),
    ];
    for (name, body) in clients {
        client(&packet, name, body)?;
    }
    let path = |name: &str| packet.folder.join(format!("{name}.rs"));
    for (name, _) in clients {
        let mut honest = packet.consumer(&path(name), name, &packet.metadata, &packet.owner());
        super::artifacts::checked(name, &packet.folder, &mut honest)?;
    }
    let bytes = std::fs::read(&packet.metadata)?;
    for (name, user, wire, diagnostic) in forgeries(&bytes)? {
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&metadata, wire)?;
        let mut consumer = packet.consumer(&path(user), name, &metadata, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for length in 0..bytes.len() {
        assert!(crates::decode(&bytes[..length]).is_err(), "prefix {length}");
    }
    Ok(())
}
#[test]
fn lifetime_nominal_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "lifetimes")?;
    let foundation = foundation(&packet)?;
    let consumer = client(
        &packet,
        "client",
        "let v: u8 = 42; match base::l005(base::L005 { t: &v }) { 42 => 42, _ => 7 }",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut nominals = String::new();
        for index in 0..width {
            writeln!(
                nominals,
                "pub struct L{index:03}<'a> {{ pub t: &'a u8 }}\npub fn l{index:03}(x: L{index:03}<'_>) -> u8 {{ *x.t }}"
            )?;
        }
        let source = generated(
            &packet,
            &format!("lifetimes{width}.rs"),
            &format!("{foundation}{nominals}"),
        )?;
        let metadata = packet.folder.join(format!("lifetimes{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&consumer, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every nominal adds the same declaration, slot,
    // region, field, header and body bytes.
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
fn lifetime_nominal_clients_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
