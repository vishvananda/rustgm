//! Fresh CLI processes consume version 13's unsafe associated functions.
//! Each consumer admits an unsafe inherent method, trait member or impl
//! member, and an unsafe call inside or to one, under direct Call's rule, and
//! its own calls need an unsafe context. A trait member implemented with the
//! other safety stops in the provider's own frontend, foreign conventions and
//! default bodies keep their fences, and versions 12 and 10 keep theirs.
use super::fresh_packets::{Packet, Result};
use rustgm::support::crates;
const SPEC: &str = "pa29/tests/spec/guarded-methods";
const FIELDS: &str = "pa29/tests/spec/callable-fields/provider.rs";
const NOMINALS: &str = "pa29/tests/spec/source-nominals/provider.rs";
/// The output tree shared with the auto-trait controls; each label is distinct.
const GROUP: &str = "auto-traits";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/guarded-method-outcomes.json`.
const ACCEPTED: [&str; 16] = [
    "method",
    "path",
    "assoc",
    "relay",
    "inside",
    "bare",
    "vtable",
    "generic",
    "late",
    "captured",
    "const",
    "trait",
    "bound",
    "client-impl",
    "reified",
    "formatter",
];
const NONORDINARY: &str =
    "source nonordinary callable effect/safety/convention requires a wider profile";
const ASSOCIATED: &str = "source unsafe associated Calls require their wider actual profile";
const MARK: &str = "// Generated associated-unsafe control.\n";
/// Each opponent appends its lines to the callable-fields provider.
const OPPONENTS: [(&str, &str, &str); 4] = [
    (
        "safe-impl",
        "pub trait Bad { unsafe fn m(&self) -> u8; }\nimpl Bad for u8 { fn m(&self) -> u8 { 1 } }\n",
        "incompatible types",
    ),
    (
        "unsafe-impl",
        "pub trait Good { fn m(&self) -> u8; }\nimpl Good for u8 { unsafe fn m(&self) -> u8 { 1 } }\n",
        "incompatible types",
    ),
    (
        "extern-c-method",
        "pub struct Ext;\nimpl Ext { pub unsafe extern \"C\" fn c() {} }\n",
        NONORDINARY,
    ),
    (
        "trait-default",
        "pub trait D { unsafe fn m(&self) -> u8 { 1 } }\n",
        "source provided associated bodies require a wider explicit profile",
    ),
];
/// Each older-profile control appends its lines to the nominals provider,
/// which versions 10, 12 and 13 all export; versions 12 and 10 reject it by
/// the named texts the base-pin exploration fixed, and version 13 exports it.
const OLDER: [(&str, &str, &str); 3] = [
    (
        "older-trait",
        "pub trait Get { unsafe fn fetch(&self); }\nimpl Get for u8 { unsafe fn fetch(&self) {} }\n",
        NONORDINARY,
    ),
    (
        "older-inherent",
        "pub struct Lane { pub n: u8 }\nimpl Lane { pub unsafe fn get(&self) {} }\n",
        "source impl owner has no completed canonical conformance record",
    ),
    (
        "older-call",
        "pub unsafe fn raw() {}\npub trait Via { fn via(&self); }\nimpl Via for u8 { fn via(&self) { unsafe { raw() } } }\n",
        ASSOCIATED,
    ),
];
const OLDER_PROFILES: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
fn unwritten(path: &std::path::Path) -> Result<bool> {
    Ok(!path.exists() || std::fs::metadata(path)?.len() == 0)
}
/// Writes `base` plus the marked `lines` as `name.rs` in the packet's folder.
fn generated(packet: &Packet, base: &str, name: &str, lines: &str) -> Result<std::path::PathBuf> {
    let text = std::fs::read_to_string(packet.root.join(base))?;
    let source = packet.folder.join(format!("{name}.rs"));
    std::fs::write(&source, format!("{text}{MARK}{lines}"))?;
    Ok(source)
}
#[test]
fn clients_call_unsafe_associated_functions_in_their_contexts() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "guarded-frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    packet.rejected(
        "missing",
        &mut packet.client("missing"),
        "call to unsafe function requires an unsafe context",
    )?;
    Ok(())
}
#[test]
fn mismatched_members_foreign_conventions_and_defaults_keep_their_fences() -> Result {
    let packet = Packet::new(GROUP, SPEC, "guarded-opponents", "provider")?;
    for (name, lines, diagnostic) in OPPONENTS {
        let source = generated(&packet, FIELDS, name, lines)?;
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        packet.rejected(
            name,
            &mut packet.export(PROFILE, &source, &metadata),
            diagnostic,
        )?;
        assert!(unwritten(&metadata)?, "{name} wrote a packet");
    }
    Ok(())
}
#[test]
fn older_profiles_keep_both_associated_fences() -> Result {
    let packet = Packet::new(GROUP, SPEC, "guarded-older", "provider")?;
    for (name, lines, diagnostic) in OLDER {
        let source = generated(&packet, NOMINALS, name, lines)?;
        for profile in OLDER_PROFILES {
            let label = format!("{name}-{profile}");
            let metadata = packet.folder.join(format!("{label}.rgcrate"));
            packet.rejected(
                &label,
                &mut packet.export(profile, &source, &metadata),
                diagnostic,
            )?;
            assert!(unwritten(&metadata)?, "{label} wrote a packet");
        }
        let metadata = packet.folder.join(format!("{name}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked(name, &packet.folder, &mut export)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn unsafe_associated_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "guarded-native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
