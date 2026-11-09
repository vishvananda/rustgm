//! Version thirteen exports crates whose `macro_rules!` expansions leave every
//! item name at the root and bind no local, parameter or label. Fresh
//! consumers run the provider's bodies natively and expand its exported
//! `$crate` macro. Every other marked form keeps the provenance fence.
use super::fresh_packets::{Packet, Result, inner};
use rustgm::support::crates;
use std::path::PathBuf;
const SPEC: &str = "pa29/tests/spec/macro-contexts";
const GROUP: &str = "macro-contexts";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/macro-context-outcomes.json`.
const ACCEPTED: [&str; 4] = ["forty-two", "plus-forty", "relay", "answer"];
const PROVENANCE: &str = "source macro/semantic-name provenance metadata is not implemented";
const GENERATED: &str = "macro-generated exported macros are outside the macro metadata profile";
const DECL_MACRO: &str = "#![feature(decl_macro)]\n";
/// Controls rustc accepts that keep the provenance fence: a marked local,
/// label and parameter, an opaque expansion and a contextual item.
const FENCED: [(&str, bool, &str); 5] = [
    (
        "hygienic-local",
        false,
        "macro_rules! keep { ($e:expr) => {{ let x = $e; x }} }\npub fn kept() -> u8 { keep!(42) }\n",
    ),
    (
        "hygienic-label",
        false,
        "macro_rules! spin { () => { 'out: loop { break 'out 42 } } }\npub fn spun() -> u8 { spin!() }\n",
    ),
    (
        "macro-parameter",
        false,
        "macro_rules! define { () => { pub fn made(x: u8) -> u8 { x } } }\ndefine!();\n",
    ),
    (
        "opaque-expansion",
        true,
        "macro seven() { 42 }\npub fn seven_up() -> u8 { seven!() }\n",
    ),
    (
        "contextual-item",
        true,
        "macro def() { fn hidden() -> u8 { 42 } pub fn shown() -> u8 { hidden() } }\ndef!();\n",
    ),
];
const MADE: &str = "#[macro_export] macro_rules! made { () => { 1 } }";
const UNUSED: &str = "pub fn forty_two() -> u8 { 42 }\n#[macro_export]\nmacro_rules! answer { () => { $crate::forty_two() } }\n";
fn foundation(packet: &Packet) -> Result<String> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    Ok(fixture
        .split("// Authored macro-context transport controls.")
        .next()
        .ok_or("macro-context foundation")?
        .to_owned())
}
fn generated(packet: &Packet, label: &str, prefix: &str, control: &str) -> Result<PathBuf> {
    let path = packet.folder.join(format!("{label}.rs"));
    std::fs::write(&path, format!("{prefix}{}{control}", foundation(packet)?))?;
    Ok(path)
}
#[test]
fn private_macro_rules_bodies_export_and_run_in_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // Lane B's macro envelope (tag 14, version 1) wraps the version-13 core.
    assert_eq!(bytes.get(12..15), Some([14, 1, 1].as_slice()));
    let core = inner(&bytes)?;
    assert_eq!(bytes.get(core + 13..core + 15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    let macros = interface.macros().ok_or("missing exported macros")?;
    let names: Vec<_> = macros
        .rows()
        .iter()
        .map(crates::exported_macros::Macro::name)
        .collect();
    assert_eq!(names, ["answer"]);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        assert_eq!(packet.native(name)?.status.code(), Some(42), "{name}");
    }
    packet.rejected(
        "mismatch",
        &mut packet.client("mismatch"),
        "incompatible types",
    )?;
    let source = packet.fixture("provider");
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &source, &older),
            PROVENANCE,
        )?;
    }
    Ok(())
}
#[test]
fn marked_locals_opaque_marks_and_generated_exported_macros_stay_fenced() -> Result {
    let packet = Packet::new(GROUP, SPEC, "fences", "provider")?;
    for (label, opaque, control) in FENCED {
        let prefix = if opaque { DECL_MACRO } else { "" };
        let source = generated(&packet, label, prefix, control)?;
        let metadata = packet.folder.join(format!("{label}.rgcrate"));
        packet.rejected(
            label,
            &mut packet.export(PROFILE, &source, &metadata),
            PROVENANCE,
        )?;
    }
    let make = format!("macro_rules! make {{ () => {{ {MADE} }} }}\nmake!{{}}\n");
    let source = generated(&packet, "generated-macro", "", &make)?;
    let metadata = packet.folder.join("generated-macro.rgcrate");
    packet.rejected(
        "generated-macro",
        &mut packet.export(PROFILE, &source, &metadata),
        GENERATED,
    )?;
    // The rejection names the generated definition, whose tokens come from
    // `make!`'s body in the physical provider source.
    let text = std::fs::read_to_string(&source)?;
    let start = text.find(MADE).ok_or("generated definition text")?;
    let stderr = std::fs::read_to_string(packet.folder.join("generated-macro.stderr"))?;
    let offset: usize = stderr
        .split(&format!(": {GENERATED}"))
        .next()
        .and_then(|prefix| prefix.rsplit("file 0: ").next())
        .ok_or("located generated-macro diagnostic")?
        .trim()
        .parse()?;
    assert!(
        (start..start + MADE.len()).contains(&offset),
        "{offset}: {stderr}"
    );
    // Root `$crate` tokens in an exported macro the provider never invokes.
    let source = generated(&packet, "crate-path", "", UNUSED)?;
    let metadata = packet.folder.join("crate-path.rgcrate");
    super::artifacts::checked(
        "crate-path",
        &packet.folder,
        &mut packet.export(PROFILE, &source, &metadata),
    )?;
    Ok(())
}
