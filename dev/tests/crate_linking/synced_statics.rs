//! A shared static requires the `sync` lang item and a proof that its type
//! implements it, as rustc's well-formedness check requires. A source client
//! takes every lang item from its source core; `static mut` needs neither.
use super::fresh_packets::{Packet, Result};
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
const SPEC: &str = "pa29/tests/spec/static-sync";
const GROUP: &str = "static-sync";
const PROFILE: &str = "module-bodies-v13";
const MISSING: &str = "requires `sync` lang_item";
/// Frozen outcomes from `pa29/tests/static-sync-outcomes.json`.
const ACCEPTED: [&str; 2] = ["mut-static", "answer"];
const OPPONENTS: [(&str, &str); 2] = [("plain-static", MISSING), ("local-static", MISSING)];
const SYNC: &str = "#[lang = \"sync\"]\npub unsafe auto trait Sync {}\n";
const READ: &str = "static N: u8 = 42;\npub fn read_n() -> u8 { N }\n";
const RAW: &str = "pub struct Raw { pub p: *const u8 }\n";
const SHARED: &str = "static R: Raw = Raw { p: &N as *const u8 };\n";
const MUTABLE: &str = "static mut R: Raw = Raw { p: &N as *const u8 };\n";
const DENIED: &str = "impl !Sync for Raw {}\n";
const UNSATISFIED: &str = "shared static type must implement Sync";
#[test]
fn source_client_statics_require_the_sync_lang_item() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    Ok(())
}
/// The packet's shared lines, before its authored section.
fn foundation(negative: bool) -> Result<String> {
    let fixture = crate::resolve::fixture(&format!("{SPEC}/provider.rs"))?;
    let (lines, _) = fixture
        .split_once("// Authored static Sync controls")
        .ok_or("static Sync foundation")?;
    if !negative {
        return Ok(lines.to_owned());
    }
    let (features, rest) = lines.split_once(")]\n").ok_or("crate feature list")?;
    Ok(format!("{features},negative_impls)]\n{rest}"))
}
/// Analyzes one generated source core, returning its first diagnostic.
fn analyzed(source: &str, tag: &str) -> Result<Option<String>> {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = crate::resolve::source_provider_config(source, tag, &config)?;
    Ok(pipeline::analyze_source(&report)
        .err()
        .map(|error| error.to_string()))
}
#[test]
fn source_core_statics_prove_sync_from_their_own_declarations() -> Result {
    let plain = foundation(false)?;
    let negative = foundation(true)?;
    let controls: [(&str, String, Option<&str>); 7] = [
        ("nosync", format!("{plain}{READ}"), Some(MISSING)),
        ("sync", format!("{plain}{SYNC}{READ}"), None),
        ("raw", format!("{plain}{SYNC}{READ}{RAW}{SHARED}"), None),
        (
            "raw_impl",
            format!("{plain}{SYNC}{READ}{RAW}{SHARED}unsafe impl Sync for Raw {{}}\n"),
            None,
        ),
        (
            "raw_mut",
            format!("{plain}{SYNC}{READ}{RAW}{MUTABLE}"),
            None,
        ),
        (
            "negative",
            format!("{negative}{SYNC}{READ}{RAW}{DENIED}{SHARED}"),
            Some(UNSATISFIED),
        ),
        (
            "negative_mut",
            format!("{negative}{SYNC}{READ}{RAW}{DENIED}{MUTABLE}"),
            None,
        ),
    ];
    for (tag, source, expected) in controls {
        let observed = analyzed(&source, &format!("static_sync_{tag}"))?;
        match (expected, observed.as_deref()) {
            (None, None) => {}
            (Some(diagnostic), Some(message)) if message.contains(diagnostic) => {}
            (expected, observed) => {
                return Err(format!("{tag}: expected {expected:?}, observed {observed:?}").into());
            }
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn source_client_statics_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
