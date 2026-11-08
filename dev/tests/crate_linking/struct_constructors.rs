//! Fresh CLI processes re-prove version-thirteen tuple and unit struct
//! constructors: each travels as its struct's only variant declaration, and
//! each consumer re-derives its shape, payload, name and visibility from the
//! struct's own rows.
use super::fresh_packets::{Packet, Result, relabel, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/struct-constructors";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "struct-constructors";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/struct-constructor-outcomes.json`.
const ACCEPTED: [&str; 11] = [
    "called", "direct", "pattern", "method", "generic", "unit", "imported", "braced", "constant",
    "held", "secret",
];
const OPPONENTS: [(&str, &str); 9] = [
    ("private-constructor", "private value name `Secret`"),
    ("private-field", "aggregate field is private"),
    ("private-pattern", "private value name `Secret`"),
    ("private-braced", "aggregate field is private"),
    ("crate-field", "private value name `Narrow`"),
    ("wrong-arity", "call argument count mismatch"),
    ("wrong-payload", "incompatible types"),
    ("unit-call", "call requires a known function type"),
    ("held-escape", "access conflicts with a live reference loan"),
];
const OUTSIDE: &str = "source declaration kind is outside the initial source profile";
const WRAP: &str = "// Generated tuple-struct gate.\npub struct Wrap(pub u8); pub fn wrap(x: u8) -> Wrap { Wrap(x) }\n";
const UNIT: &str =
    "// Generated unit-struct gate.\npub struct Unit; pub fn unit() -> Unit { Unit }\n";
const WRAP_CLIENTS: [(&str, &str); 2] = [
    ("wrap-called", "match base::wrap(42).0 { 42 => 42, _ => 7 }"),
    ("wrap-direct", "match base::Wrap(42).0 { 42 => 42, _ => 7 }"),
];
const FENCED: [(&str, &str, &str); 2] = [
    (
        "empty",
        "// Generated empty tuple-struct gate.\npub struct Empty(); pub fn empty() -> Empty { Empty() }\n",
        "source empty tuple struct constructors require their wider profile",
    ),
    (
        "non-exhaustive",
        "// Generated non_exhaustive tuple-struct gate.\n#[non_exhaustive] pub struct Open(pub u8); pub fn open(x: u8) -> Open { Open(x) }\n",
        "source non_exhaustive struct constructors require their wider profile",
    ),
];
/// Amended by the `PhantomData` role slice: the unit-struct role now exports
/// under version 13 (`pa29/tests/struct-constructor-outcomes.json`).
const PHANTOM: &str = "// Generated PhantomData gate.\n#[lang = \"phantom_data\"] pub struct PhantomData<T: PointeeSized>;\npub fn phantom() -> PhantomData<u8> { PhantomData }\n";
const WIRE: &str = "// Generated struct-constructor wire control.\npub struct Tup(pub u8);\npub struct Two(pub u8);\npub struct Rec { pub x: u8 }\npub enum Sum { One(u8) }\npub mod m { pub mod n { pub struct Sec(pub(super) u8); pub fn sec(x: u8) -> Sec { Sec(x) } } }\n";
const TUP: &str = "let base::Tup(x) = base::Tup(42); match x { 42 => 42, _ => 7 }";
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
fn nominals(packet: &Packet) -> Result<String> {
    Ok(std::fs::read_to_string(
        packet.root.join(format!("{NOMINALS}/provider.rs")),
    )?)
}
fn foundation(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    Ok(fixture
        .split("// Authored struct-constructor transport controls.")
        .next()
        .ok_or("struct-constructor foundation")?
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
fn struct_constructors_survive_wire_and_fresh_consumers() -> Result {
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
fn older_profiles_and_unsupported_struct_shapes_reject_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = nominals(&packet)?;
    for (label, control) in [("wrap", WRAP), ("unit", UNIT)] {
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
                OUTSIDE,
            )?;
        }
    }
    let source = packet.folder.join("wrap.rs");
    let wrap = packet.folder.join("wrap.rgcrate");
    exported(&packet, &source, &wrap)?;
    for (label, body) in WRAP_CLIENTS {
        let consumer = client(&packet, label, body)?;
        let mut fresh = packet.consumer(&consumer, label, &wrap, &packet.owner());
        super::artifacts::checked(label, &packet.folder, &mut fresh)?;
    }
    for (label, control, diagnostic) in FENCED {
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
    let source = generated(&packet, "phantom.rs", &format!("{nominals}{PHANTOM}"))?;
    let phantom = packet.folder.join("phantom.rgcrate");
    exported(&packet, &source, &phantom)?;
    let consumer = client(
        &packet,
        "phantom-called",
        "let _: base::PhantomData<u8> = base::phantom(); 42",
    )?;
    let mut fresh = packet.consumer(&consumer, "phantom-called", &phantom, &packet.owner());
    super::artifacts::checked("phantom-called", &packet.folder, &mut fresh)?;
    Ok(())
}
fn word(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(u32::from_le_bytes(
        bytes
            .get(offset..offset + 4)
            .ok_or("operand is outside the packet")?
            .try_into()?,
    ))
}
fn unique_offset(found: &[usize], what: &str) -> Result<usize> {
    match found {
        [offset] => Ok(*offset),
        _ => Err(format!("{what} {found:?}").into()),
    }
}
/// A length-prefixed name followed by one byte.
fn named(name: &str, next: u8) -> Vec<u8> {
    let mut pattern = u32::try_from(name.len())
        .unwrap_or(u32::MAX)
        .to_le_bytes()
        .to_vec();
    pattern.extend_from_slice(name.as_bytes());
    pattern.push(next);
    pattern
}
fn occurrences(bytes: &[u8], pattern: &[u8]) -> Vec<usize> {
    bytes
        .windows(pattern.len())
        .enumerate()
        .filter(|(_, window)| *window == pattern)
        .map(|(offset, _)| offset)
        .collect()
}
/// A named, parented declaration row of `kind`: kind, parent flag, parent,
/// ordinal, name flag, name and public flag. Returns the offset of its
/// parent.
fn declaration(bytes: &[u8], kind: u8, name: &str) -> Result<usize> {
    flagged(bytes, kind, name, 1)
}
fn flagged(bytes: &[u8], kind: u8, name: &str, public: u8) -> Result<usize> {
    let found: Vec<usize> = occurrences(bytes, &named(name, public))
        .into_iter()
        .filter(|&offset| {
            offset >= 11
                && bytes[offset - 11] == kind
                && bytes[offset - 10] == 1
                && bytes[offset - 1] == 1
        })
        .map(|offset| offset - 9)
        .collect();
    unique_offset(&found, &format!("declarations {kind} {name}"))
}
/// A direct module binding: module, name, namespace, artifact, target,
/// visibility. Returns the offset of its name. A declaration row's name
/// follows its name flag 1, a binding's the high byte of its module.
fn binding(bytes: &[u8], name: &str, namespace: u8) -> Result<usize> {
    let found: Vec<usize> = occurrences(bytes, &named(name, namespace))
        .into_iter()
        .filter(|&offset| offset >= 4 && bytes[offset - 1] == 0)
        .collect();
    unique_offset(&found, &format!("bindings {name} {namespace}"))
}
type Forgery = (&'static str, Vec<u8>, &'static str);
const LEXICAL: &str = "source lexical child has a foreign kind, gap or duplicate ordinal";
fn forgeries(honest: &[u8]) -> Result<Vec<Forgery>> {
    let tup = word(honest, declaration(honest, 11, "Tup")?)?;
    let two = declaration(honest, 11, "Two")?;
    let one = declaration(honest, 11, "One")?;
    // `Rec`'s one field row: owner, ordinal 0 and the name `x`.
    let field = unique_offset(
        &occurrences(honest, &[0, 0, 0, 0, 1, 0, 0, 0, b'x']),
        "record field rows",
    )?;
    let rec = word(honest, field - 4)?;
    let tup_type = binding(honest, "Tup", 0)?;
    let root = word(honest, tup_type - 4)?;
    // `Sec`'s value binding: its narrowed visibility follows its target.
    let sec = binding(honest, "Sec", 1)? + named("Sec", 1).len();
    if honest.get(sec + 8) != Some(&1) {
        return Err("`Sec`'s constructor binding is not module-restricted".into());
    }
    // The public form: `Sec`'s constructor row claims public, and its
    // binding drops the four-byte module of its narrowed visibility.
    let mut public = honest.to_vec();
    let flag = flagged(honest, 11, "Sec", 0)? + 13 + "Sec".len();
    public[flag] = 1;
    public.splice(sec + 8..sec + 13, [0]);
    let mut renamed = honest.to_vec();
    let name = declaration(honest, 11, "Tup")? + 13;
    renamed[name..name + 3].copy_from_slice(b"Tuq");
    let mut positional = honest.to_vec();
    positional[field + 8] = b'0';
    Ok(vec![
        (
            "record-constructor",
            with_id(honest, declaration(honest, 11, "Tup")?, rec),
            "source record struct has no constructor",
        ),
        // A moved declaration that keeps its lexical ordinal 0 duplicates
        // `Tup`'s own constructor; with the next ordinal it reaches the
        // constructor and variant checks.
        ("second-constructor", with_id(honest, two, tup), LEXICAL),
        (
            "second-constructor-ordinal",
            with_id(&with_id(honest, two, tup), two + 4, 1),
            "source struct has a second constructor",
        ),
        ("enum-variant", with_id(honest, one, tup), LEXICAL),
        (
            "enum-variant-ordinal",
            with_id(&with_id(honest, one, tup), one + 4, 1),
            "source variant changed its actual enum owner, ordinal or fields",
        ),
        (
            "renamed-constructor",
            renamed,
            "source struct constructor changed its struct's name",
        ),
        (
            "widened-visibility",
            with_id(honest, sec + 9, root),
            "source struct constructor visibility differs from its struct and fields",
        ),
        (
            "public-visibility",
            public,
            "source struct constructor visibility differs from its struct and fields",
        ),
        (
            "record-positional",
            positional,
            "source tuple struct fields require its constructor",
        ),
        (
            "version-12",
            relabel(honest, 12),
            "unknown source declaration kind",
        ),
    ])
}
#[test]
fn forged_struct_constructor_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let nominals = nominals(&packet)?;
    let source = generated(&packet, "wire.rs", &format!("{nominals}{WIRE}"))?;
    exported(&packet, &source, &packet.metadata)?;
    let user = client(&packet, "tup", TUP)?;
    let mut honest = packet.consumer(&user, "tup", &packet.metadata, &packet.owner());
    super::artifacts::checked("tup", &packet.folder, &mut honest)?;
    let bytes = std::fs::read(&packet.metadata)?;
    for (name, wire, diagnostic) in forgeries(&bytes)? {
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
fn struct_constructor_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "structs")?;
    let foundation = foundation(&packet)?;
    let consumer = client(
        &packet,
        "client",
        "match base::w005(42).0 { 42 => match base::W005(42).0 { 42 => 42, _ => 8 }, _ => 7 }",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut structs = String::new();
        for index in 0..width {
            writeln!(
                structs,
                "pub struct W{index:03}(pub u8); pub fn w{index:03}(x: u8) -> W{index:03} {{ W{index:03}(x) }}"
            )?;
        }
        let source = generated(
            &packet,
            &format!("structs{width}.rs"),
            &format!("{foundation}// Generated scaling control.\n{structs}"),
        )?;
        let metadata = packet.folder.join(format!("structs{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&consumer, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every struct adds the same declaration, binding,
    // nominal, field, header and body bytes.
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
fn struct_constructor_clients_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
