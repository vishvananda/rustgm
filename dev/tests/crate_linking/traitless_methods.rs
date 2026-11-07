//! Fresh CLI processes re-prove version-thirteen inherent impls on provider
//! nominals, their receiver lookup and path calls, and free originals whose
//! late region formals the consumer proves against their declared results.
use super::fresh_packets::{Packet, Result, relabel, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/inherent-methods";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "inherent-methods";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/inherent-methods-outcomes.json`.
const ACCEPTED: [&str; 6] = [
    "map-call",
    "auto-ref",
    "unwrap-or",
    "path-call",
    "slot",
    "present",
];
const OPPONENTS: [(&str, &str); 4] = [
    ("missing-method", "no inherent method with this name"),
    ("wrong-default", "incompatible types"),
    ("moved-receiver", "uninitialized or moved"),
    ("foreign-instance", "method for this receiver"),
];
const NOMINAL_METHOD: &str = "// Generated older-profile inherent control.\nimpl<T> Packet<T> { pub fn get(self) -> T { self.value } }\n";
const NOMINAL_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::make(42i32).get() }\n";
const LATE: &str = "// Generated late-region control.\n";
const FIRST: &str = "pub fn first<'a, 'b>(a: &'a u8, b: &'b u8) -> &'a u8 { a }\n";
const SECOND: &str = "pub fn first<'a, 'b>(a: &'a u8, b: &'b u8) -> &'b u8 { b }\n";
const GATE: &str =
    "pub fn first<'a, 'b, T>(a: &'a Packet<T>, b: &'b Packet<T>) -> &'a Packet<T> { a }\n";
const FIRST_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {\n    let x = 42u8;\n    let y = 7u8;\n    match *base::first(&x, &y) { 42 => 42, _ => 8 }\n}\n";
const GATE_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {\n    let p = base::make(42i32);\n    let q = base::make(7i32);\n    let _r = base::first(&p, &q);\n    42\n}\n";
/// An inherent impl header row is 48 bytes; a trait impl row adds its
/// application and is 52.
const ROW: usize = 48;
const TRAIT_ROW: usize = 52;
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
fn word(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}
fn id(bytes: &[u8], offset: usize) -> Result<u32> {
    Ok(word(bytes, offset).ok_or("operand is outside the packet")?)
}
/// An inherent impl header: Self definition and scope, Self type, a zero
/// trait flag and modifiers, no captures and no region bounds; its members.
fn members(bytes: &[u8], offset: usize) -> Option<(u32, u32)> {
    let header = bytes.get(offset..offset + ROW)?;
    (word(header, 0)? == word(header, 4)?
        && header[12..20].iter().all(|&byte| byte == 0)
        && word(header, 20)? == word(header, 24)?)
    .then_some((word(header, 40)?, word(header, 44)?))
}
/// The `Opt<T>`, `Slot<T>` and `Slot<u8>` headers: three consecutive rows
/// whose member ranges chain with three, two and one methods.
fn inherent_rows(bytes: &[u8]) -> Result<usize> {
    let found: Vec<usize> = (0..bytes.len())
        .filter(|&offset| {
            let rows: Option<Vec<(u32, u32)>> = (0..3)
                .map(|row| members(bytes, offset + row * ROW))
                .collect();
            rows.is_some_and(|rows| {
                rows.windows(2).all(|pair| pair[0].1 == pair[1].0)
                    && rows
                        .iter()
                        .map(|(start, end)| end.wrapping_sub(*start))
                        .eq([3, 2, 1])
            })
        })
        .collect();
    match found.as_slice() {
        [offset] => Ok(*offset),
        _ => Err(format!("inherent header rows {found:?}").into()),
    }
}
/// `first`'s function header: callable and original signatures over two
/// parameters, then a region scheme of two late formals. Returns the offsets
/// of the two signature result words.
fn late_results(bytes: &[u8]) -> Result<[usize; 2]> {
    let signature = |offset: usize| {
        word(bytes, offset + 8)
            .zip(word(bytes, offset + 4))
            .is_some_and(|(end, start)| end.wrapping_sub(start) == 2)
            && bytes.get(offset + 16..offset + 19) == Some([0, 0, 0].as_slice())
    };
    let regions = |offset: usize| {
        let field = |index: usize| word(bytes, offset + 4 * index);
        field(0) == Some(2)
            && field(1) == Some(0)
            && field(2).is_some()
            && field(2) == field(3)
            && field(5)
                .zip(field(4))
                .is_some_and(|(end, start)| end.wrapping_sub(start) == 2)
    };
    let found: Vec<usize> = (0..bytes.len())
        .filter(|&offset| signature(offset) && signature(offset + 19) && regions(offset + 38))
        .collect();
    match found.as_slice() {
        [offset] => Ok([offset + 12, offset + 31]),
        _ => Err(format!("late-region function headers {found:?}").into()),
    }
}
/// Decision 1: an imported constructor is synthesized at the consumer root,
/// but `Opt::Some::<u8>` is emitted once for its variant and instance
/// arguments and both of its uses call that one definition.
fn constructor_emitted_once(text: &str) -> Result {
    let entry = text.split("\nfunction ").next().ok_or("entry function")?;
    let call = entry
        .lines()
        .map(str::trim_start)
        .find(|line| line.starts_with("call void @") && line.contains("$internal$rg."))
        .ok_or("entry constructor call")?;
    let symbol = call
        .trim_start_matches("call void ")
        .split('(')
        .next()
        .ok_or("constructor symbol")?;
    assert_eq!(
        text.matches(&format!("function {symbol}(")).count(),
        1,
        "{symbol}"
    );
    assert_eq!(
        text.matches(&format!("call void {symbol}(")).count(),
        2,
        "{symbol}"
    );
    Ok(())
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
#[test]
fn inherent_methods_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    constructor_emitted_once(&std::fs::read_to_string(
        packet.folder.join("map-call.lowir"),
    )?)?;
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
fn forged_inherent_headers_reject_in_fresh_consumers() -> Result {
    let packet = publish("wire")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let rows = inherent_rows(&bytes)?;
    // `impl Add for u8` is the trait impl row immediately before them.
    let add = rows - TRAIT_ROW;
    assert_eq!(bytes.get(add + 12), Some(&1), "trait impl flag");
    let (scalar, slot) = (id(&bytes, add + 8)?, id(&bytes, rows + ROW + 8)?);
    let mut swapped = bytes.clone();
    swapped[rows + 40..rows + ROW].copy_from_slice(&bytes[rows + ROW + 40..rows + 2 * ROW]);
    swapped[rows + ROW + 40..rows + 2 * ROW].copy_from_slice(&bytes[rows + 40..rows + ROW]);
    for (name, wire, diagnostic) in [
        (
            "scalar-self",
            with_id(&bytes, rows + 8, scalar),
            "imported inherent impl Self is not a nominal of its own provider",
        ),
        (
            "slot-self",
            with_id(&bytes, rows + 8, slot),
            "source method receiver differs from its actual implementation Self",
        ),
        (
            "swapped-members",
            swapped,
            "source implementation member belongs to another actual Self owner",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let source = packet.fixture("map-call");
        let mut consumer = packet.consumer(&source, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}
#[test]
fn inherent_impls_on_source_nominals_keep_older_gates() -> Result {
    let packet = Packet::new(GROUP, NOMINALS, "nominals", "provider")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let source = generated(
        &packet,
        "nominals.rs",
        &format!("{fixture}{NOMINAL_METHOD}"),
    )?;
    let client = generated(&packet, "nominals-client.rs", NOMINAL_CLIENT)?;
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &source, &older),
            "source impl owner has no completed canonical conformance record",
        )?;
    }
    exported(&packet, &source, &packet.metadata)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    let mut consumer = packet.consumer(
        &client,
        "nominals-client",
        &packet.metadata,
        &packet.owner(),
    );
    super::artifacts::checked("nominals-client", &packet.folder, &mut consumer)?;
    // A version-twelve reader rejects the version-thirteen stream before any
    // descriptor exists; the descriptor's own by-name gate is a private control.
    let older = packet.folder.join("relabelled.rgcrate");
    std::fs::write(&older, relabel(&bytes, 12))?;
    let mut consumer = packet.consumer(&client, "relabelled", &older, &packet.owner());
    packet.rejected(
        "relabelled",
        &mut consumer,
        "unknown source ordinary value operation",
    )
}
#[test]
fn late_region_originals_reprove_their_declared_results() -> Result {
    let packet = Packet::new(GROUP, SPEC, "regions", "provider")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let gate = generated(&packet, "gate.rs", &format!("{nominals}{LATE}{GATE}"))?;
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &gate, &older),
            "source original region formals require their complete wider body profile",
        )?;
    }
    let gated = packet.folder.join("gate.rgcrate");
    exported(&packet, &gate, &gated)?;
    let client = generated(&packet, "gate-client.rs", GATE_CLIENT)?;
    let mut consumer = packet.consumer(&client, "gate-client", &gated, &packet.owner());
    super::artifacts::checked("gate-client", &packet.folder, &mut consumer)?;
    let first = generated(&packet, "first.rs", &format!("{fixture}{LATE}{FIRST}"))?;
    let second = generated(&packet, "second.rs", &format!("{fixture}{LATE}{SECOND}"))?;
    exported(&packet, &first, &packet.metadata)?;
    let variant = packet.folder.join("second.rgcrate");
    exported(&packet, &second, &variant)?;
    let client = generated(&packet, "first-client.rs", FIRST_CLIENT)?;
    let mut consumer = packet.consumer(&client, "first-client", &packet.metadata, &packet.owner());
    super::artifacts::checked("first-client", &packet.folder, &mut consumer)?;
    // Claim the variant's `-> &'b u8` over the unchanged body returning `a`.
    let (honest, claimed) = (std::fs::read(&packet.metadata)?, std::fs::read(&variant)?);
    let results = late_results(&honest)?;
    assert_eq!(late_results(&claimed)?, results);
    let claim = |offsets: &[usize]| -> Result<Vec<u8>> {
        offsets.iter().try_fold(honest.clone(), |wire, &offset| {
            Ok(with_id(&wire, offset, id(&claimed, offset)?))
        })
    };
    for (name, offsets, diagnostic) in [
        (
            "claimed-result",
            &results[..],
            "required lifetime does not outlive its target",
        ),
        (
            "claimed-callable",
            &results[..1],
            "source signature reference mutability or exact lifetime differs",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, claim(offsets)?)?;
        let mut consumer = packet.consumer(&client, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    Ok(())
}
#[test]
fn inherent_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "methods")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("// Authored inherent-method transport controls.")
        .next()
        .ok_or("inherent foundation")?;
    let client = generated(
        &packet,
        "client.rs",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::wide(42u8).m005() { 42 => 42, _ => 7 } }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let mut methods = String::new();
        for index in 0..width {
            write!(
                methods,
                "pub fn m{index:03}(self) -> u8 {{ match self {{ Wide::One(x) => x, Wide::Two(x) => x }} }} "
            )?;
        }
        let source = generated(
            &packet,
            &format!("methods{width}.rs"),
            &format!(
                "{foundation}// Generated scaling control.\npub enum Wide {{ One(u8), Two(u8) }}\npub fn wide(x: u8) -> Wide {{ Wide::One(x) }}\nimpl Wide {{ {methods}}}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("methods{width}.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every method adds the same declaration, header,
    // member link and body bytes. The client demands one method instance.
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
fn inherent_methods_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    for (label, provider, client) in [
        (
            "nominals-client",
            format!("{nominals}{NOMINAL_METHOD}"),
            NOMINAL_CLIENT,
        ),
        (
            "gate-client",
            format!("{nominals}{LATE}{GATE}"),
            GATE_CLIENT,
        ),
        (
            "first-client",
            format!("{fixture}{LATE}{FIRST}"),
            FIRST_CLIENT,
        ),
    ] {
        let source = generated(&packet, &format!("{label}-provider.rs"), &provider)?;
        let metadata = packet.folder.join(format!("{label}-provider.rgcrate"));
        exported(&packet, &source, &metadata)?;
        let client = generated(&packet, &format!("{label}.rs"), client)?;
        let mut consumer = packet.consumer(&client, label, &metadata, &packet.owner());
        super::artifacts::checked(label, &packet.folder, &mut consumer)?;
        let output = packet.native(label)?;
        assert_eq!(output.status.code(), Some(42), "{label}");
        assert!(output.stdout.is_empty(), "{label}");
    }
    Ok(())
}
