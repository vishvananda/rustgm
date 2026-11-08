//! Fresh CLI processes re-prove version-thirteen raw-pointer and slice type
//! rows. A raw pointer carries no region and admits an unsized pointee; each
//! consumer proves a slice element sized from its own facts, never from the
//! wire.
use super::fresh_packets::{Packet, Result, relabel, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, path::Path};
const SPEC: &str = "pa29/tests/spec/pointer-slices";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const GROUP: &str = "pointer-slices";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/pointer-slice-outcomes.json`.
const ACCEPTED: [&str; 7] = [
    "same",
    "same-mut",
    "generic",
    "raw-field",
    "raw-literal",
    "text-ptr",
    "slices",
];
const OPPONENTS: [(&str, &str); 4] = [
    ("wrong-pointee", "incompatible types"),
    (
        "const-for-mut",
        "raw pointer coercion cannot strengthen mutability",
    ),
    (
        "deref-safe",
        "raw pointer dereference requires an unsafe context",
    ),
    ("slice-mismatch", "incompatible types"),
];
const UNSUPPORTED: &str = "source type form requires an explicitly supported transport row";
/// Older-profile gates over `source-nominals`, each with a client of it.
const GATES: [(&str, &str, &str); 2] = [
    (
        "pointer",
        "// Generated raw-pointer gate.\npub fn same(p: *const u8) -> *const u8 { p }\n",
        // `source-nominals` has no `Copy`, so this client cannot read `*p`.
        "pub extern \"C\" fn entry() -> i32 { let x = 42u8; let _p = base::same(&x as *const u8); 42 }",
    ),
    (
        "slice",
        "// Generated slice gate.\npub fn width(_s: &[u8]) -> u8 { 42 }\n",
        "pub extern \"C\" fn entry() -> i32 { 42 }\nfn relay(s: &[u8]) -> u8 { base::width(s) }",
    ),
];
const WIRE: &str = "// Generated raw-pointer and slice wire control.\npub fn same(p: *const u8) -> *const u8 { p }\npub fn text_ptr(p: *const str) -> *const str { p }\npub fn width(_s: &[u8]) -> u8 { 42 }\n";
const CROSSES: &str = "source type reference crosses its typed table namespace";
fn source(packet: &Packet, name: &str, text: &str) -> Result<std::path::PathBuf> {
    let path = packet.folder.join(name);
    std::fs::write(&path, text)?;
    Ok(path)
}
fn export(packet: &Packet, source: &Path, metadata: &Path) -> Result {
    let mut command = packet.export(PROFILE, source, metadata);
    super::artifacts::checked("provider", &packet.folder, &mut command).map(drop)
}
/// The provider's lines before its authored pointer and slice section.
fn shared(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let (lines, _) = fixture
        .split_once("// Authored raw-pointer and slice transport controls.")
        .ok_or("pointer-slice foundation")?;
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
#[test]
fn raw_pointers_and_slices_survive_wire_and_fresh_consumers() -> Result {
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
    let provider = packet.fixture("provider");
    for profile in OLDER {
        let metadata = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &provider, &metadata),
            "named, associated or generic alias transport requires its wider profile",
        )?;
    }
    Ok(())
}
#[test]
fn older_profiles_fence_pointer_and_slice_rows_by_name() -> Result {
    let packet = Packet::new(GROUP, SPEC, "gates", "provider")?;
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    for (label, control, body) in GATES {
        let provider = source(
            &packet,
            &format!("{label}.rs"),
            &format!("{nominals}{control}"),
        )?;
        for profile in OLDER {
            let name = format!("{label}-{profile}");
            let metadata = packet.folder.join(format!("{name}.rgcrate"));
            let mut command = packet.export(profile, &provider, &metadata);
            packet.rejected(&name, &mut command, UNSUPPORTED)?;
        }
        let metadata = packet.folder.join(format!("{label}.rgcrate"));
        export(&packet, &provider, &metadata)?;
        let name = format!("{label}-client");
        let client = entry(&packet, &name, body)?;
        let mut fresh = packet.consumer(&client, &name, &metadata, &packet.owner());
        super::artifacts::checked(&name, &packet.folder, &mut fresh)?;
    }
    Ok(())
}
/// The wire control's three consecutive rows: `same`'s `*const u8`,
/// `text_ptr`'s `*const str` and `width`'s `[u8]`, whose element is the first
/// pointer's pointee. Returns the first row's offset.
fn rows(bytes: &[u8]) -> Result<usize> {
    let found: Vec<usize> = bytes
        .windows(17)
        .enumerate()
        .filter(|(_, row)| {
            row[..2] == [12, 0] && row[6..8] == [12, 0] && row[12] == 13 && row[13..17] == row[2..6]
        })
        .map(|(offset, _)| offset)
        .collect();
    let [offset] = *found.as_slice() else {
        return Err(format!("pointer and slice rows {found:?}").into());
    };
    Ok(offset)
}
type Forgery = (&'static str, Vec<u8>, &'static str);
fn forgeries(honest: &[u8]) -> Result<Vec<Forgery>> {
    let same = rows(honest)?;
    let mut mutability = honest.to_vec();
    mutability[same + 1] = 2;
    let text = u32::from_le_bytes(honest[same + 8..same + 12].try_into()?);
    let element = same + 13;
    Ok(vec![
        ("mutability", mutability, "invalid source boolean"),
        ("pointee", with_id(honest, same + 2, u32::MAX >> 8), CROSSES),
        ("element", with_id(honest, element, u32::MAX >> 8), CROSSES),
        (
            "text-element",
            with_id(honest, element, text),
            "source slice element requires a sized type",
        ),
        (
            "version-12",
            relabel(honest, 12),
            "unknown source declaration kind",
        ),
    ])
}
#[test]
fn forged_pointer_and_slice_rows_reject_in_fresh_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "wire", "provider")?;
    let provider = source(&packet, "wire.rs", &format!("{}{WIRE}", shared(&packet)?))?;
    export(&packet, &provider, &packet.metadata)?;
    let user = packet.fixture("same");
    let mut honest = packet.consumer(&user, "same", &packet.metadata, &packet.owner());
    super::artifacts::checked("same", &packet.folder, &mut honest)?;
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
const RELAXED: &str = "// Generated relaxed slice-element control.\npub fn pair<T, U: ?Sized>(_s: &[T], _p: *const U) -> u8 { 42 }\n";
/// `pair`'s rows in order: `T`, `[T]`, `&[T]`, `U` and `*const U`. Returns
/// the slice row's element offset and `U`'s row id.
fn relaxed(bytes: &[u8]) -> Result<(usize, u32)> {
    let found: Vec<usize> = bytes
        .windows(31)
        .enumerate()
        .filter(|(_, row)| {
            row[0] == 1
                && row[5] == 13
                && row[10..12] == [2, 0]
                && row[20] == 1
                && row[25..27] == [12, 0]
        })
        .map(|(offset, _)| offset)
        .collect();
    let [offset] = *found.as_slice() else {
        return Err(format!("relaxed slice rows {found:?}").into());
    };
    let pointee = bytes.get(offset + 27..offset + 31).ok_or("`U` row id")?;
    Ok((offset + 6, u32::from_le_bytes(pointee.try_into()?)))
}
/// Validation sees only a parameter element; the consumer's own facts know
/// that `U` is `?Sized`.
#[test]
fn relaxed_slice_elements_reject_from_consumer_facts() -> Result {
    let packet = Packet::new(GROUP, SPEC, "relaxed", "provider")?;
    let provider = source(
        &packet,
        "relaxed.rs",
        &format!("{}{RELAXED}", shared(&packet)?),
    )?;
    export(&packet, &provider, &packet.metadata)?;
    let user = entry(
        &packet,
        "pair",
        "pub extern \"C\" fn entry() -> i32 { 42 }\nfn relay(s: &[u8], p: *const str) -> u8 { base::pair::<u8, str>(s, p) }",
    )?;
    let mut honest = packet.consumer(&user, "pair", &packet.metadata, &packet.owner());
    super::artifacts::checked("pair", &packet.folder, &mut honest)?;
    let bytes = std::fs::read(&packet.metadata)?;
    let (element, relaxed) = relaxed(&bytes)?;
    let metadata = packet.folder.join("relaxed-element.rgcrate");
    std::fs::write(&metadata, with_id(&bytes, element, relaxed))?;
    let mut consumer = packet.consumer(&user, "relaxed-element", &metadata, &packet.owner());
    packet.rejected(
        "relaxed-element",
        &mut consumer,
        "source slice element requires a sized type",
    )
}
#[test]
fn pointer_and_slice_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "pairs")?;
    let lines = shared(&packet)?;
    let client = entry(
        &packet,
        "client",
        "pub extern \"C\" fn entry() -> i32 { let x = 42u8; match unsafe { *base::p005(&x as *const u8) } { 42 => 42, _ => 7 } }",
    )?;
    let mut observed = Vec::new();
    for width in [6, 24, 96] {
        let mut pairs = String::from("// Generated scaling control.\n");
        for index in 0..width {
            writeln!(
                pairs,
                "pub fn p{index:03}(p: *const u8) -> *const u8 {{ p }}\npub fn s{index:03}(s: &[u8]) -> &[u8] {{ s }}"
            )?;
        }
        let provider = source(
            &packet,
            &format!("pairs{width}.rs"),
            &format!("{lines}{pairs}"),
        )?;
        let metadata = packet.folder.join(format!("pairs{width}.rgcrate"));
        export(&packet, &provider, &metadata)?;
        let label = format!("client{width}");
        let mut fresh = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut fresh)?;
        let lowir = std::fs::metadata(packet.folder.join(format!("{label}.lowir")))?.len();
        observed.push((std::fs::metadata(&metadata)?.len(), lowir));
    }
    // Fixed-length names: every pair adds the same declaration, header,
    // recipe and original bytes, and the client's LowIR names one pointer.
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
fn pointer_and_slice_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
