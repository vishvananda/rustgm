//! Fresh CLI processes consume named constants exported under version
//! thirteen. Each consumer re-proves every constant's canonical value for its
//! declared primitive type, installs that type as the declaration's cell and
//! seeds the value as a completed definition, so client reads, array lengths,
//! patterns, const evaluation and imported code-17 bodies all use it. Wider
//! values, associated constants and statics keep their fences by name, and
//! older profiles keep the kind fence.
use super::fresh_packets::{Packet, Result};
use rustgm::support::crates;
const SPEC: &str = "pa29/tests/spec/named-constants";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/named-constant-outcomes.json`.
const ACCEPTED: [&str; 10] = [
    "read",
    "local-const",
    "length",
    "pattern",
    "body",
    "secret",
    "nested",
    "float",
    "wide",
    "inner-pair",
];
/// Amendment 5: rustc accepts `const-call`, but client const evaluation of an
/// imported const fn from a provider with primitive Copy impls stops at this
/// existing fence, which the base pin reproduces without any constant.
const FENCED: (&str, &str) = (
    "const-call",
    "implementation validation has no source scheduling context",
);
/// Each rejected client and its local twin, which the frontend rejects alike.
const REJECTED: [&str; 3] = ["wrong-length", "wrong-type", "private"];
const AGGREGATE: &str = "source aggregate const value requires its wider profile";
const REFERENCE: &str = "source reference const value requires its wider profile";
const ASSOCIATED: &str = "source associated constants require their wider profile";
const KIND: &str = "source declaration kind is outside the initial source profile";
/// Each generated provider opponent appends these lines to the frozen
/// provider; rustc 1.98.0 compiles every one. The static opponent also
/// declares the oracle-only `drop_glue` that rustc's static item needs.
const OPPONENTS: [(&str, &str, &str); 9] = [
    ("pair", "pub const PAIR: (u8, u8) = (1, 2);\n", AGGREGATE),
    ("bytes", "pub const BYTES: [u8; 2] = [1, 2];\n", AGGREGATE),
    (
        "record",
        "pub struct Wrap(pub u8);\npub const WRAP: Wrap = Wrap(1);\n",
        AGGREGATE,
    ),
    ("name", "pub const NAME: &str = \"x\";\n", REFERENCE),
    ("borrowed", "pub const ONE: &u8 = &1;\n", REFERENCE),
    (
        "null",
        "pub const NULL: *const u8 = 0 as *const u8;\n",
        "source pointer const value requires its wider profile",
    ),
    (
        "inherent",
        "pub struct Wide;\nimpl Wide { pub const MAX: u16 = 9; }\n",
        ASSOCIATED,
    ),
    (
        "trait-const",
        "pub trait Limit { const TOP: u32; }\n",
        ASSOCIATED,
    ),
    (
        "static",
        "#[lang = \"sync\"]\npub unsafe auto trait Sync {}\n// Oracle-only shim: rustc's static item needs this item. RustGM never\n// configures `rustc_oracle`.\n#[cfg(rustc_oracle)]\n#[lang = \"drop_glue\"]\npub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}\npub static STORED: u8 = 1;\n",
        KIND,
    ),
];
/// Each local twin is the branch provider's first 57 lines, a marker comment
/// and these lines, compiled as one crate.
const TWINS: [(&str, &str); 3] = [
    (
        "wrong-length",
        "const LANES: usize = 4;\nfn width(_: &[u8; LANES]) -> i32 { 42 }\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { width(&[1, 2, 3]) }\n",
    ),
    (
        "wrong-type",
        "const LANES: usize = 4;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { let lanes: u8 = LANES; lanes as i32 }\n",
    ),
    (
        "private",
        "mod base { const SECRET: u8 = 42; }\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::SECRET as i32 }\n",
    ),
];
fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
/// The branch provider's first 57 lines, which every generated input keeps.
fn prefix(packet: &Packet) -> Result<String> {
    let branches = std::fs::read_to_string(
        packet
            .root
            .join("pa29/tests/spec/source-branches/provider.rs"),
    )?;
    let lines: Vec<&str> = branches.lines().take(57).collect();
    Ok(format!("{}\n", lines.join("\n")))
}
/// The diagnostic message after its location prefix.
fn message(stderr: &[u8]) -> Result<String> {
    let text = String::from_utf8_lossy(stderr);
    let line = text
        .lines()
        .find(|line| line.starts_with("ERROR"))
        .ok_or("rejection without a diagnostic")?;
    Ok(line.rsplit(": ").next().unwrap_or(line).to_owned())
}
#[test]
fn clients_read_named_constants_and_wider_values_keep_their_fences() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "constants-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    let (name, fence) = FENCED;
    packet.rejected(name, &mut packet.client(name), fence)?;
    let prefix = prefix(&packet)?;
    for (name, lines) in TWINS {
        let twin = packet.folder.join(format!("twin-{name}.rs"));
        std::fs::write(&twin, format!("{prefix}// Generated local twin.\n{lines}"))?;
        let metadata = packet.folder.join(format!("twin-{name}.rgcrate"));
        let twin = packet.export(PROFILE, &twin, &metadata).output()?;
        assert_eq!(twin.status.code(), Some(1), "twin {name}");
        let client = packet.client(name).output()?;
        assert_eq!(client.status.code(), Some(1), "{name}");
        assert_eq!(message(&client.stderr)?, message(&twin.stderr)?, "{name}");
    }
    assert!(
        REJECTED
            .iter()
            .all(|name| TWINS.iter().any(|(twin, _)| twin == name))
    );
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    for (name, lines, diagnostic) in OPPONENTS {
        let source = packet.folder.join(format!("{name}.rs"));
        std::fs::write(
            &source,
            format!("{provider}// Generated module-constant opponent.\n{lines}"),
        )?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        packet.rejected(name, &mut export, diagnostic)?;
        assert!(unwritten(&metadata)?, "{name} wrote a packet");
    }
    Ok(())
}
/// A packet whose provider is the fixture `provider`, under the shared
/// `provider` owner identity, so that two fixtures' packets compare exactly.
fn published(label: &str, provider: &str) -> Result<Packet> {
    let packet = Packet::new(GROUP, SPEC, label, "provider")?;
    let mut export = packet.export(PROFILE, &packet.fixture(provider), &packet.metadata);
    super::artifacts::checked(provider, &packet.folder, &mut export)?;
    Ok(packet)
}
#[test]
fn unnamed_constants_export_no_row_and_item_local_constants_stay_closed() -> Result {
    let packet = published("constants-controls", "unnamed-provider")?;
    // Dropping the unnamed row shifts no declaration: the packet equals the
    // same provider's without its `const _`.
    let twin = packet.folder.join("unnamed-twin.rgcrate");
    let mut export = packet.export(PROFILE, &packet.fixture("unnamed-twin"), &twin);
    super::artifacts::checked("unnamed-twin", &packet.folder, &mut export)?;
    assert_eq!(std::fs::read(&packet.metadata)?, std::fs::read(&twin)?);
    super::artifacts::checked(
        "unnamed-after",
        &packet.folder,
        &mut packet.client("unnamed-after"),
    )?;
    let generic = published("constants-generic", "generic-local-provider")?;
    super::artifacts::checked(
        "generic-local",
        &generic.folder,
        &mut generic.client("generic-local"),
    )?;
    Ok(())
}
#[test]
fn older_profiles_keep_the_constant_kind_fence() -> Result {
    let packet = Packet::new(GROUP, SPEC, "constants-older", "provider")?;
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let metadata = packet.folder.join(format!("{profile}.rgcrate"));
        let mut export = packet.export(profile, &packet.fixture("provider"), &metadata);
        packet.rejected(profile, &mut export, KIND)?;
        assert!(unwritten(&metadata)?, "{profile} wrote a packet");
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn constant_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "constants-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    for (provider, client) in [
        ("unnamed-provider", "unnamed-after"),
        ("generic-local-provider", "generic-local"),
    ] {
        let packet = published(&format!("constants-native-{client}"), provider)?;
        super::artifacts::checked(client, &packet.folder, &mut packet.client(client))?;
        let output = packet.native(client)?;
        assert_eq!(output.status.code(), Some(42), "{client}");
        assert!(output.stdout.is_empty(), "{client}");
    }
    Ok(())
}
