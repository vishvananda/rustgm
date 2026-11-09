//! Version thirteen carries a free `unsafe fn`'s declared target features as
//! function header bit 3. Fresh consumers re-prove each row on their own
//! installed header and table and call the imported functions through their
//! ordinary unsafe signatures. Forms outside the slice keep named fences at
//! export and older profiles keep theirs. Since the guarded-methods item an
//! unsafe inherent method carries its row as a free function does.
use super::fresh_packets::{Packet, Result, inner, unique};
use rustgm::support::crates;
const SPEC: &str = "pa41/tests/spec/target-feature-transport";
const NOMINALS: &str = "pa29/tests/spec/source-nominals";
const NATIVE: &str = "pa41/tests/spec/asm-native";
const GROUP: &str = "target-feature-transport";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa41/tests/target-feature-transport-outcomes.json`.
const WIDER: &str =
    "source named/foreign/target-feature function metadata requires its wider profile";
const FORM: &str = "source target-feature function form requires its wider profile";
/// Amendment 3: the declared `incompatible types` was a failed prediction for
/// the client and its twin alike.
const SAFETY: &str = "incompatible callable convention or safety";
const MEMBER: &str =
    "source trait member target features require their trait implementation transport";
/// The lines every generated control keeps: the provider's first 73, the
/// pinned foundation with `__m128`.
const FOUNDATION: usize = 73;
const SECTION: &str = "// Authored target-feature transport section.\n";
/// Each export opponent is the foundation, the section comment and one line.
const OPPONENTS: [(&str, &str, &str); 8] = [
    (
        "safe",
        "#[target_feature(enable = \"sse2\")] pub fn s() {}",
        "source safe target-feature function export requires its contextual call policy",
    ),
    (
        "trait-impl",
        "pub trait T { unsafe fn m(); } pub struct X; impl T for X { #[target_feature(enable = \"sse2\")] unsafe fn m() {} }",
        MEMBER,
    ),
    (
        "trait-default",
        "pub trait D { #[target_feature(enable = \"sse2\")] unsafe fn m() {} }",
        MEMBER,
    ),
    (
        "extern-c",
        "#[target_feature(enable = \"sse2\")] pub unsafe extern \"C\" fn c() {}",
        FORM,
    ),
    (
        "safe-target",
        "pub struct Lane; impl Lane { #[target_feature(enable = \"sse\")] pub fn s() {} }",
        "source safe target-feature function export requires its contextual call policy",
    ),
    (
        "const",
        "#[target_feature(enable = \"sse2\")] pub const unsafe fn k() {}",
        FORM,
    ),
    (
        "no-mangle",
        "#[unsafe(no_mangle)] #[target_feature(enable = \"sse2\")] pub unsafe fn n() {}",
        WIDER,
    ),
    (
        "inline-always",
        "#[inline(always)] #[target_feature(enable = \"sse2\")] pub unsafe fn a() {}",
        "cannot use `#[inline(always)]` with `#[target_feature]`",
    ),
];
/// The former `inherent` opponent, which exports since the guarded-methods
/// item: outcomes amendment 6, mover M2.
const INHERENT: &str = "pub struct Lane; impl Lane { #[target_feature(enable = \"sse\")] pub unsafe fn get(value: u32) -> u32 { value } }";
const KEEP: &str =
    "#[target_feature(enable = \"sse2,sse\")] unsafe fn keep<T: Copy>(value: T) -> T { value }\n";
const SECOND: &str =
    "#[target_feature(enable = \"sse2\")] pub unsafe fn second(a: u32, b: u32) -> u32 { b }\n";

/// The stream witness shares native.rs's lines before its observers: the
/// real `_mm_stream_ps` body.
const STREAM_PRELUDE: usize = 95;
/// Amendment 5: the witness's C surface is one scalar function, since a
/// fresh client's own interface discovery admits neither the imported
/// `__m128` nor a raw pointer in an exported signature.
const STREAM_MAIN: &str =
    "int rg_stream_check(void);\nint main(void) { return rg_stream_check(); }\n";
/// Backend-contract control: `shared/lowir.md` states no alignment for an
/// `i128` slot, and the pinned v3 backend has placed u128 locals at 16
/// bytes. Declared in `obj/parallel/target-feature-transport/f-followup`.
const U128_SLOTS: &str = r#"#![feature(no_core, asm_native_fixture)]
#![no_core]
#![allow(internal_features)]
extern crate base;
// Backend-contract control: shared/lowir.md states no alignment for an i128
// slot, and the pinned v3 backend has placed u128 locals at 16 bytes. Each
// function returns the address of a u128 local; the C main checks it mod 16.
#[unsafe(no_mangle)]
pub extern "C" fn rg_u128_alone() -> usize {
    let mut x: u128 = 0;
    let p = &raw mut x;
    p as usize
}
#[unsafe(no_mangle)]
pub extern "C" fn rg_u128_after_small() -> usize {
    let mut a: u8 = 0;
    let mut b: u32 = 0;
    let _a = &raw mut a;
    let _b = &raw mut b;
    let mut x: u128 = 0;
    let p = &raw mut x;
    p as usize
}
"#;
/// Returns 1 or 2 if either u128 local's address is not a multiple of 16.
const U128_MAIN: &str = r"unsigned long rg_u128_alone(void);
unsigned long rg_u128_after_small(void);
int main(void) {
    if (rg_u128_alone() % 16 != 0) return 1;
    if (rg_u128_after_small() % 16 != 0) return 2;
    return 42;
}
";

fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
/// Writes `source` under the packet's folder and exports it under `profile`.
fn generated(
    packet: &Packet,
    name: &str,
    profile: &str,
    source: &str,
) -> Result<std::process::Command> {
    let path = packet.folder.join(format!("{name}.rs"));
    std::fs::write(&path, source)?;
    let metadata = packet.folder.join(format!("{name}.rgcrate"));
    Ok(packet.export(profile, &path, &metadata))
}
/// The provider's first `FOUNDATION` lines and its `Copy` impls for `u8` and
/// `u32`, the two lines after the section comment.
fn foundation(packet: &Packet) -> Result<(String, String)> {
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let lines: Vec<&str> = provider.lines().collect();
    let foundation = format!("{}\n", lines[..FOUNDATION].join("\n"));
    assert_eq!(lines[FOUNDATION], SECTION.trim_end());
    let copies = format!("{}\n", lines[FOUNDATION + 1..FOUNDATION + 3].join("\n"));
    Ok((foundation, copies))
}
/// The `keep::<u8>` function block with its symbol spelling replaced: the one
/// function besides `entry`, whose symbol spells its crate and name in hex
/// when imported and is internal when local.
fn keep_block(lowir: &str) -> Result<String> {
    let start = lowir
        .match_indices("function @")
        .map(|(offset, _)| offset)
        .filter(|&offset| !lowir[offset..].starts_with("function @entry("))
        .collect::<Vec<_>>();
    let [start] = *start.as_slice() else {
        return Err(format!("keep blocks at {start:?}").into());
    };
    let block = &lowir[start..];
    let end = block.find("\n}\n").ok_or("keep block end")? + 3;
    let block = &block[..end];
    let symbol_end = block.find('(').ok_or("keep signature")?;
    Ok(format!("function @SYMBOL{}", &block[symbol_end..]))
}

#[test]
fn target_feature_packets_export_and_fresh_consumers_reprove_their_rows() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "feature-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // The foundation's builtin `arch::asm` row needs Lane B's macro envelope.
    assert_eq!(bytes.get(12..14), Some([14, 2].as_slice()));
    let core = inner(&bytes)?;
    assert_eq!(bytes.get(core + 13..core + 15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    // Each bit-3 row: flags 8, the count, then each declared name.
    for (count, names) in [
        (1_u8, &["sse2"][..]),
        (2, &["sse", "sse2"][..]),
        (1, &["avx"][..]),
    ] {
        let mut row = vec![8, count, 0, 0, 0];
        for name in names {
            row.extend_from_slice(&u32::try_from(name.len())?.to_le_bytes());
            row.extend_from_slice(name.as_bytes());
        }
        unique(&bytes, &row)?;
    }
    for client in ["call", "generic", "pointer", "wide"] {
        super::artifacts::checked(client, &packet.folder, &mut packet.client(client))?;
    }
    packet.rejected(
        "unsafe-free",
        &mut packet.client("unsafe-free"),
        "call to unsafe function requires an unsafe context",
    )?;
    packet.rejected("safe-pointer", &mut packet.client("safe-pointer"), SAFETY)?;
    // Local twins: the same rules, the same texts.
    let (foundation, copies) = foundation(&packet)?;
    let twin = format!("{foundation}{SECTION}{copies}");
    for (name, entry, diagnostic) in [
        (
            "twin-unsafe-free",
            "pub extern \"C\" fn entry() -> i32 { second(0, 42) as i32 }",
            "call to unsafe function requires an unsafe context",
        ),
        (
            "twin-safe-pointer",
            "pub extern \"C\" fn entry() -> i32 { let f: fn(u32, u32) -> u32 = second; f(0, 42) as i32 }",
            SAFETY,
        ),
    ] {
        let source = format!("{twin}{SECOND}#[unsafe(no_mangle)]\n{entry}\n");
        packet.rejected(
            name,
            &mut generated(&packet, name, PROFILE, &source)?,
            diagnostic,
        )?;
    }
    // LowIR carries no target-feature attribute, so the imported instance
    // equals the local one once the symbol spelling is one placeholder.
    let declared = format!(
        "{twin}{KEEP}#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {{ unsafe {{ keep::<u8>(42) as i32 }} }}\n"
    );
    super::artifacts::checked(
        "twin-keep",
        &packet.folder,
        &mut generated(&packet, "twin-keep", PROFILE, &declared)?,
    )?;
    let local = std::fs::read_to_string(packet.folder.join("twin-keep.lowir"))?;
    let imported = std::fs::read_to_string(packet.folder.join("generic.lowir"))?;
    assert_eq!(keep_block(&imported)?, keep_block(&local)?);
    Ok(())
}
#[test]
fn target_feature_forms_outside_the_slice_keep_their_fences_at_export() -> Result {
    let packet = Packet::new(GROUP, SPEC, "feature-controls", "provider")?;
    let (foundation, _) = foundation(&packet)?;
    for (name, line, diagnostic) in OPPONENTS {
        let source = format!("{foundation}{SECTION}{line}\n");
        let mut export = generated(&packet, &format!("opponent-{name}"), PROFILE, &source)?;
        packet.rejected(name, &mut export, diagnostic)?;
        let metadata = packet.folder.join(format!("opponent-{name}.rgcrate"));
        assert!(unwritten(&metadata)?, "{name} wrote a packet");
    }
    // The older profiles reach the target-feature gate on the source-nominals
    // foundation, which does not stop them earlier.
    let nominals = std::fs::read_to_string(packet.root.join(format!("{NOMINALS}/provider.rs")))?;
    let gate = format!("{nominals}{SECOND}");
    for profile in OLDER {
        let mut export = generated(&packet, &format!("older-gate-{profile}"), profile, &gate)?;
        packet.rejected(profile, &mut export, WIDER)?;
    }
    let mut export = generated(&packet, "older-gate", PROFILE, &gate)?;
    super::artifacts::checked("older-gate", &packet.folder, &mut export)?;
    // A gated feature is checked only in the defining crate.
    let (first, rest) = foundation.split_once('\n').ok_or("foundation")?;
    let gated = format!(
        "{first}\n#![feature(x87_target_feature)]\n{rest}{SECTION}#[target_feature(enable = \"x87\")] pub unsafe fn legacy() {{}}\n"
    );
    let metadata = packet.folder.join("gated.rgcrate");
    let mut export = generated(&packet, "gated", PROFILE, &gated)?;
    super::artifacts::checked("gated-provider", &packet.folder, &mut export)?;
    let mut client = packet.consumer(
        &packet.fixture("gated"),
        "gated",
        &metadata,
        &packet.owner(),
    );
    super::artifacts::checked("gated", &packet.folder, &mut client).map(drop)
}
/// Mover M2: the unsafe inherent `Lane::get` exports bit 3 with `sse`, and
/// the returned `inherent.rs` calls it from a fresh consumer (M2a).
fn inherent_packet(label: &str) -> Result<Packet> {
    let packet = Packet::new(GROUP, SPEC, label, "provider")?;
    let (foundation, _) = foundation(&packet)?;
    let source = format!("{foundation}{SECTION}{INHERENT}\n");
    let mut export = generated(&packet, "inherent-provider", PROFILE, &source)?;
    super::artifacts::checked("inherent-provider", &packet.folder, &mut export)?;
    let metadata = packet.folder.join("inherent-provider.rgcrate");
    unique(
        &std::fs::read(&metadata)?,
        &[8, 1, 0, 0, 0, 3, 0, 0, 0, b's', b's', b'e'],
    )?;
    let fixture = packet.fixture("inherent");
    let mut client = packet.consumer(&fixture, "inherent", &metadata, &packet.owner());
    super::artifacts::checked("inherent", &packet.folder, &mut client)?;
    Ok(packet)
}
#[test]
fn unsafe_inherent_target_feature_methods_export_their_rows() -> Result {
    inherent_packet("feature-inherent").map(drop)
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn the_inherent_client_executes_natively() -> Result {
    let packet = inherent_packet("feature-inherent-native")?;
    let output = packet.native("inherent")?;
    assert_eq!(output.status.code(), Some(42));
    assert!(output.stdout.is_empty());
    Ok(())
}
/// The one assembly template in `lowir`, with its label spelled `@asm0`.
fn stream_template(lowir: &str) -> Result<String> {
    const HEADER: &str = "assembly @asm";
    let starts: Vec<usize> = lowir
        .match_indices(HEADER)
        .map(|(offset, _)| offset)
        .collect();
    let [start] = *starts.as_slice() else {
        return Err(format!("assembly templates at {starts:?}").into());
    };
    let block = &lowir[start + HEADER.len()..];
    let end = block.find("\n}\n").ok_or("assembly template end")? + 3;
    let label = block.find(' ').ok_or("assembly template header")?;
    Ok(format!("{HEADER}0{}", &block[label..end]))
}
/// The real `_mm_stream_ps` body crosses the crate boundary with its declared
/// `sse` and inline hint. The packet is named `provider`, since its name is
/// the crate disambiguator's stem.
fn stream_provider(label: &str) -> Result<Packet> {
    let packet = Packet::new(GROUP, SPEC, label, "provider")?;
    let source = packet.fixture("stream-provider");
    super::artifacts::checked(
        "stream-provider",
        &packet.folder,
        &mut packet.export(PROFILE, &source, &packet.metadata),
    )?;
    let native = std::fs::read_to_string(packet.root.join(format!("{NATIVE}/native.rs")))?;
    let provider = std::fs::read_to_string(&source)?;
    let prelude = |text: &str| {
        text.lines()
            .take(STREAM_PRELUDE)
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(prelude(&provider), prelude(&native));
    assert_eq!(
        provider.lines().nth(STREAM_PRELUDE),
        Some("// Authored stream witness section.")
    );
    Ok(packet)
}
/// The stream packet and its accepted witness client.
fn stream_packet(label: &str) -> Result<Packet> {
    let packet = stream_provider(label)?;
    super::artifacts::checked(
        "stream-client",
        &packet.folder,
        &mut packet.client("stream-client"),
    )?;
    Ok(packet)
}
/// The witness client lowers the imported body's one template and one use.
#[test]
fn the_stream_witness_client_lowers_the_imported_template() -> Result {
    let packet = stream_packet("feature-stream")?;
    let lowir = std::fs::read_to_string(packet.folder.join("stream-client.lowir"))?;
    let declared =
        std::fs::read_to_string(packet.root.join(format!("{NATIVE}/stream-template.lowir")))?;
    assert_eq!(stream_template(&lowir)?, declared);
    let uses = lowir
        .lines()
        .filter(|line| line.trim_start().starts_with("asm @asm"))
        .count();
    assert_eq!(uses, 1);
    Ok(())
}
/// The imported `movntps` body runs in the client: a 16-byte-aligned local
/// holds every streamed lane.
#[test]
#[ignore = "requires configured locked CPPGM object backend, host C compiler and GNU objdump"]
fn the_stream_witness_client_executes_natively() -> Result {
    let packet = stream_packet("feature-stream-native")?;
    let output = packet.native_with("stream-client", STREAM_MAIN)?;
    assert_eq!(output.status.code(), Some(42));
    assert!(output.stdout.is_empty());
    let disassembly = super::artifacts::checked(
        "objdump",
        &packet.folder,
        std::process::Command::new("objdump")
            .args(["-d", "-M", "intel", "--no-show-raw-insn"])
            .arg(packet.folder.join("stream-client.o")),
    )?;
    let text = String::from_utf8(disassembly.stdout)?;
    std::fs::write(packet.folder.join("stream-client.disassembly"), &text)?;
    let stores = text
        .lines()
        .filter(|line| line.contains("movntps XMMWORD PTR [") && line.contains("],xmm"))
        .count();
    assert_eq!(stores, 1);
    Ok(())
}
/// Each u128 local lowers to one `i128` slot, and the pinned backend places
/// both at 16 bytes. A regression of that unstated layout fails here.
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn u128_locals_stay_sixteen_byte_aligned_natively() -> Result {
    let packet = stream_provider("feature-u128-native")?;
    let source = packet.folder.join("u128-slots.rs");
    std::fs::write(&source, U128_SLOTS)?;
    let mut client = packet.consumer(&source, "u128-slots", &packet.metadata, &packet.owner());
    super::artifacts::checked("u128-slots", &packet.folder, &mut client)?;
    let lowir = std::fs::read_to_string(packet.folder.join("u128-slots.lowir"))?;
    for function in ["rg_u128_alone", "rg_u128_after_small"] {
        let start = lowir
            .find(&format!("function @{function}("))
            .ok_or(function)?;
        let body = &lowir[start..];
        let body = &body[..body.find("\n}\n").ok_or(function)?];
        let slots = body
            .lines()
            .filter(|line| line.trim_start().starts_with("slot ") && line.ends_with(": i128"))
            .count();
        assert_eq!(slots, 1, "{function}");
    }
    let slots = lowir
        .lines()
        .filter(|line| line.trim_start().starts_with("slot ") && line.ends_with(": i128"))
        .count();
    assert_eq!(slots, 2);
    let output = packet.native_with("u128-slots", U128_MAIN)?;
    assert_eq!(output.status.code(), Some(42));
    assert!(output.stdout.is_empty());
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn target_feature_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "feature-native", "provider", PROFILE)?;
    for client in ["call", "generic", "pointer"] {
        super::artifacts::checked(client, &packet.folder, &mut packet.client(client))?;
        let output = packet.native(client)?;
        assert_eq!(output.status.code(), Some(42), "{client}");
        assert!(output.stdout.is_empty(), "{client}");
    }
    Ok(())
}
