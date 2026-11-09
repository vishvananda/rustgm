//! Fresh CLI processes consume version-thirteen record variants, built and
//! matched by field name in clients and in transported bodies and named once
//! in their enum's type namespace. A record variant used as a value or called,
//! and a missing or unknown field, are rejected as in their local twins; older
//! profiles keep their fences, and a re-exported record variant binds its own
//! type-namespace declaration. Since (ii), a record variant's zero-slot fn
//! field exports, and fresh consumers call through it.
use super::fresh_packets::{Packet, Result};
use rustgm::support::crates;
use std::fmt::Write as _;
const SPEC: &str = "pa29/tests/spec/record-variants";
const GROUP: &str = "record-variants";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/record-variant-outcomes.json`.
const ACCEPTED: [&str; 2] = ["construct", "nul"];
const OPPONENTS: [(&str, &str); 4] = [
    ("record-value", "missing value resolution"),
    ("record-call", "missing value resolution"),
    ("missing-field", "missing aggregate field initializer"),
    ("unknown-field", "unknown aggregate field"),
];
/// Amendment 2: the nul twin's exported body matches on a unit-variant value,
/// a pre-existing body-transport fence.
const PLACE: &str = "source place transport supports locals and reference projections only";
const REEXPORTS: &str =
    "// Generated re-export control.\npub use Shape::Pair;\npub use Shape::Rect;\n";
const REEXPORT_CLIENT: &str = "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::width(base::Rect { h: 2, w: 42 }) { 42 => match base::width(base::Pair(42, 1)) { 42 => 42, _ => 2 }, _ => 1 } }\n";
/// Outcomes amendment 4's witnesses: each provider plus its former opponent's
/// lines, and a client that calls through the record variant's fn field, in
/// its own body (W1) or in the imported `stored_fmt` (W2).
const WITNESSES: [(&str, &str, &str, &str); 2] = [
    (
        "w1",
        "pa29/tests/spec/callable-fields/provider.rs",
        "// Generated callable-field opponent.\n",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { let k = base::Kind::Placeholder { value: 20, formatter: base::render }; match k { base::Kind::Placeholder { value, formatter } => unsafe { formatter(value, 22) }, base::Kind::Count(_) => 7 } }\n",
    ),
    (
        "w2",
        "pa29/tests/spec/pointer-calls/provider.rs",
        "// Generated pointer-call opponent.\n",
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { unsafe { base::stored_fmt(&base::stored(20), 22) } }\n",
    ),
];
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
/// Exports both witness providers and runs each client through a fresh
/// consumer, whose `LowIR` is `{name}-client.lowir`.
fn witnesses(label: &str) -> Result<Packet> {
    let packet = Packet::new(GROUP, SPEC, label, "provider")?;
    let lines = [
        super::fielded_callables::RECORD_VARIANT,
        super::called_pointers::RECORD_VARIANT,
    ];
    for ((name, provider, marker, client), lines) in WITNESSES.into_iter().zip(lines) {
        let text = std::fs::read_to_string(packet.root.join(provider))?;
        let source = packet.folder.join(format!("{name}-provider.rs"));
        std::fs::write(&source, format!("{text}{marker}{lines}"))?;
        let metadata = packet.folder.join(format!("{name}-provider.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked(name, &packet.folder, &mut export)?;
        let entry = packet.folder.join(format!("{name}-client.rs"));
        std::fs::write(&entry, client)?;
        let label = format!("{name}-client");
        let mut consumer = packet.consumer(&entry, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
    }
    Ok(packet)
}
#[test]
fn record_variant_packets_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in OPPONENTS {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    // Each client's local twin: its entry exported beside the provider.
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let twins = ACCEPTED
        .into_iter()
        .zip([None, Some(PLACE)])
        .chain(OPPONENTS.map(|(name, diagnostic)| (name, Some(diagnostic))));
    for (name, diagnostic) in twins {
        let client = std::fs::read_to_string(packet.fixture(name))?;
        let (_, entry) = client
            .split_once("extern crate base;\n")
            .ok_or("client entry")?;
        let label = format!("twin-{name}");
        let source = packet.folder.join(format!("{label}.rs"));
        std::fs::write(
            &source,
            format!(
                "{provider}// Generated local twin.\n{}",
                entry.replace("base::", "crate::")
            ),
        )?;
        let mut export = packet.export(PROFILE, &source, &source.with_extension("rgcrate"));
        match diagnostic {
            None => {
                super::artifacts::checked(&label, &packet.folder, &mut export)?;
            }
            Some(diagnostic) => packet.rejected(&label, &mut export, diagnostic)?,
        }
    }
    Ok(())
}
#[test]
fn record_variant_reexports_bind_their_type_namespace_rows() -> Result {
    let packet = Packet::new(GROUP, SPEC, "reexports", "provider")?;
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let source = packet.folder.join("reexport.rs");
    std::fs::write(&source, format!("{provider}{REEXPORTS}"))?;
    let mut export = packet.export(PROFILE, &source, &packet.metadata);
    super::artifacts::checked("provider", &packet.folder, &mut export)?;
    let client = packet.folder.join("reexport-client.rs");
    std::fs::write(&client, REEXPORT_CLIENT)?;
    let mut consumer = packet.consumer(
        &client,
        "reexport-client",
        &packet.metadata,
        &packet.owner(),
    );
    super::artifacts::checked("reexport-client", &packet.folder, &mut consumer)?;
    Ok(())
}
#[test]
fn older_profiles_keep_their_fences_for_record_variants() -> Result {
    let packet = Packet::new(GROUP, SPEC, "older", "provider")?;
    let provider = std::fs::read_to_string(packet.fixture("provider"))?;
    let lines: Vec<_> = provider.lines().take(62).collect();
    let enumeration = packet.folder.join("older-enum.rs");
    std::fs::write(
        &enumeration,
        format!(
            "{}\n// Generated record-variant control.\npub enum Rec {{ A {{ x: u8 }}, B }}\n",
            lines.join("\n")
        ),
    )?;
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        for (label, source, diagnostic) in [
            (
                "provider",
                packet.fixture("provider"),
                "named, associated or generic alias transport requires its wider profile",
            ),
            (
                "older-enum",
                enumeration.clone(),
                "source declaration kind is outside the initial source profile",
            ),
        ] {
            let label = format!("{label}-{profile}");
            let metadata = packet.folder.join(format!("{label}.rgcrate"));
            let mut export = packet.export(profile, &source, &metadata);
            packet.rejected(&label, &mut export, diagnostic)?;
            assert!(!metadata.exists(), "{label} wrote a packet");
        }
    }
    Ok(())
}
#[test]
fn record_variant_packets_and_fresh_consumers_grow_linearly() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "variants")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("// Authored record-variant")
        .next()
        .ok_or("record-variant foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { match base::rank(base::lowest()) { 0 => 42, _ => 7 } }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let (mut variants, mut arms) = (String::new(), String::new());
        for index in 0..width {
            write!(variants, "V{index:03} {{ f{index:03}: u8 }}, ")?;
            write!(arms, "Wide::V{index:03} {{ .. }} => {index}, ")?;
        }
        let source = packet.folder.join(format!("variants{width}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}// Generated scaling control.\npub enum Wide {{ {variants}}}\npub fn lowest() -> Wide {{ Wide::V000 {{ f000: 0 }} }}\npub fn rank(w: Wide) -> u8 {{ match w {{ {arms}}} }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("variants{width}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
    }
    // Fixed-length names: every variant, field name and arm adds the same
    // packet bytes; the consumer re-proves and lowers each arm once.
    let ([small, middle, large], [first, second, third]) = (
        <[u64; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(
        (third - second) <= 5 * (second - first).max(64),
        "{lowir:?}"
    );
    Ok(())
}
#[test]
fn record_variant_fn_fields_reach_fresh_consumers() -> Result {
    witnesses("fn-fields").map(drop)
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn record_variant_fn_field_witnesses_execute_natively() -> Result {
    let packet = witnesses("fn-native")?;
    for (name, ..) in WITNESSES {
        let label = format!("{name}-client");
        let output = packet.native(&label)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn record_variant_clients_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        assert_eq!(output.status.code(), Some(42), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
