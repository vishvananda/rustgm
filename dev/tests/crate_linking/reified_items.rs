//! Fresh CLI processes consume fn items reified to fn pointers in exported
//! bodies. Every consumer re-proves the item's applied signature against the
//! pointer, then links or instantiates the item, and the client calls the
//! pointer in its own code. Higher-ranked and C pointers, closures, tracked
//! functions and explicit casts keep their export fences by name; a call
//! through a pointer and constructor reification export since R2. The
//! version-twelve profile rejects reification.
use super::fresh_packets::{Packet, Result};
use rustgm::support::crates;
const SPEC: &str = "pa29/tests/spec/reified-items";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/reified-item-outcomes.json`.
const ACCEPTED: [&str; 9] = [
    "take",
    "pick",
    "erase",
    "getter",
    "method",
    "make-local",
    "make-provider",
    "as-unsafe",
    "raw-ptr",
];
const CALLABLE: &str = "source anonymous callable requires its supported closed Rust frame";
const TRANSPORT: &str = "source rvalue requires a complete wider transport row";
const TRACKED: &str = "source tracked function reification requires its caller-location ABI";
/// Each generated provider opponent appends these lines to the frozen
/// provider; rustc 1.98.0 compiles every one. A tracked opponent also
/// declares the oracle-only `Location` that rustc's reify shim needs.
const OPPONENTS: [(&str, &str, &str); 6] = [
    (
        "hr",
        "pub fn first(x: &u8) -> &u8 { x }\npub fn hr() -> for<'a> fn(&'a u8) -> &'a u8 { first }\n",
        CALLABLE,
    ),
    (
        "c",
        "pub extern \"C\" fn cfn(_v: u8) -> i32 { 42 }\npub fn c() -> extern \"C\" fn(u8) -> i32 { cfn }\n",
        CALLABLE,
    ),
    // Closure construction stops before its reification is reached.
    (
        "closure",
        "pub fn clo() -> fn(u8) -> i32 { |_v| 42 }\n",
        TRANSPORT,
    ),
    (
        "tracked",
        "#[cfg(rustc_oracle)]\n#[lang = \"panic_location\"]\npub struct Location<'a> { file: &'a str, line: u32, col: u32 }\n#[track_caller]\npub fn tr(_v: u8) -> i32 { 42 }\npub fn tracked() -> fn(u8) -> i32 { tr }\n",
        TRACKED,
    ),
    (
        "tracked-generic",
        "#[cfg(rustc_oracle)]\n#[lang = \"panic_location\"]\npub struct Location<'a> { file: &'a str, line: u32, col: u32 }\n#[track_caller]\npub fn tr(_v: u8) -> i32 { 42 }\npub fn tracked<T>() -> fn(u8) -> i32 { tr }\n",
        TRACKED,
    ),
    (
        "cast",
        "pub fn cast() -> fn(u8) -> i32 { hold as fn(u8) -> i32 }\n",
        "source cast operand requires a closed primitive type",
    ),
];
/// Former opponents that R2 exports: amendment 1's call through a pointer and
/// amendment 2's constructors, whose consumers define their own instances.
const AMENDED: [(&str, &str); 4] = [
    (
        "indirect",
        "pub fn call_ptr(f: fn(u8) -> i32) -> i32 { f(7) }\n",
    ),
    (
        "struct-ctor",
        "pub struct P(pub u8);\npub fn ctor() -> fn(u8) -> P { P }\n",
    ),
    (
        "variant-ctor",
        "pub enum E { A(u8), B }\npub fn variant() -> fn(u8) -> E { E::A }\n",
    ),
    (
        "generic-ctor",
        "pub enum Opt<T> { Some(T), None }\npub fn some<T>() -> fn(T) -> Opt<T> { Opt::Some }\n",
    ),
];
/// The lines the older-profile control appends to the branch provider.
const OLDER: &str = "// Generated reification control.\npub fn hold(_v: u8) -> i32 { 42 }\npub fn take() -> fn(u8) -> i32 { hold }\n";
fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
#[test]
fn clients_consume_reified_items_and_other_forms_keep_their_fences() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "reified-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    for (name, lines, diagnostic) in OPPONENTS {
        let source = packet.folder.join(format!("{name}.rs"));
        std::fs::write(
            &source,
            format!("{provider}// Generated reification opponent.\n{lines}"),
        )?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        packet.rejected(name, &mut export, diagnostic)?;
        assert!(unwritten(&metadata)?, "{name} wrote a packet");
    }
    for (name, lines) in AMENDED {
        let source = packet.folder.join(format!("{name}.rs"));
        std::fs::write(
            &source,
            format!("{provider}// Generated reification opponent.\n{lines}"),
        )?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked(name, &packet.folder, &mut export)?;
        assert!(!unwritten(&metadata)?, "{name} wrote no packet");
    }
    Ok(())
}
#[test]
fn version_twelve_rejects_reification_and_version_thirteen_exports_it() -> Result {
    let packet = Packet::new(GROUP, SPEC, "reified-older", "provider")?;
    let branches = std::fs::read_to_string(
        packet
            .root
            .join("pa29/tests/spec/source-branches/provider.rs"),
    )?;
    let source = packet.folder.join("older.rs");
    std::fs::write(&source, format!("{branches}{OLDER}"))?;
    let twelve = packet.folder.join("twelve.rgcrate");
    let mut export = packet.export("module-branches-v12", &source, &twelve);
    packet.rejected("twelve", &mut export, TRANSPORT)?;
    assert!(unwritten(&twelve)?, "version twelve wrote a packet");
    let mut export = packet.export(PROFILE, &source, &packet.metadata);
    super::artifacts::checked("thirteen", &packet.folder, &mut export)?;
    super::artifacts::checked("take", &packet.folder, &mut packet.client("take"))?;
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn reified_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "reified-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
