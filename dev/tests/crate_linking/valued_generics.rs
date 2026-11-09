//! Version thirteen exports const parameters over integer, bool and char
//! types with their concrete and rigid arguments, in signatures and in body
//! applications. Fresh consumers instantiate imported const-generic
//! nominals, impls and functions, and the ignored native test runs them.
//! Older profiles and readers reject the new rows by name, and forged terms
//! fail at decode.
use super::fresh_packets::{Packet, Result, relabel};
use rustgm::support::crates;
use std::path::PathBuf;
const SPEC: &str = "pa29/tests/spec/const-parameters";
const GROUP: &str = "const-parameters";
const PROFILE: &str = "module-bodies-v13";
const OLDER: [&str; 2] = ["module-branches-v12", "module-nominals-v10"];
/// Frozen outcomes from `pa29/tests/const-parameter-outcomes.json`.
const ACCEPTED: [&str; 4] = ["tag", "mixed", "scalars", "applied"];
const REJECTED: [(&str, &str); 4] = [
    ("wrong-value", "incompatible types"),
    ("wrong-sign", "incompatible types"),
    ("wrong-order", "unsupported value declaration"),
    ("misapplied", "incompatible types"),
];
const ALIAS: &str = "named, associated or generic alias transport requires its wider profile";
const CONTROLS: [(&str, &str, &str); 2] = [
    (
        "value-read",
        "pub fn n<const N: usize>() -> usize { N }\n",
        "source rvalue requires a complete wider transport row",
    ),
    (
        "const-default",
        "pub struct D<const N: usize = 3>;\n",
        "source parameter defaults are outside the initial profile",
    ),
];
fn generated(packet: &Packet, label: &str, control: &str) -> Result<PathBuf> {
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("// Authored const-parameter transport controls.")
        .next()
        .ok_or("const-parameter foundation")?;
    let path = packet.folder.join(format!("{label}.rs"));
    std::fs::write(&path, format!("{foundation}{control}"))?;
    Ok(path)
}
/// The one value term `2 | 0 | ty | bits` whose 16 bits are `bits`.
fn value_term(bytes: &[u8], bits: u128) -> Result<usize> {
    let bits = bits.to_le_bytes();
    let found: Vec<usize> = (0..bytes.len().saturating_sub(22))
        .filter(|&at| bytes[at..at + 2] == [2, 0] && bytes[at + 6..at + 22] == bits)
        .collect();
    let [at] = *found.as_slice() else {
        return Err(format!("value terms {found:?}").into());
    };
    Ok(at)
}
fn forged(bytes: &[u8], at: usize, value: &[u8]) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[at..at + value.len()].copy_from_slice(value);
    changed
}

#[test]
fn const_parameter_packets_export_to_fresh_consumers() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "frontend", "provider", PROFILE)?;
    let bytes = std::fs::read(&packet.metadata)?;
    // A plain version-thirteen core: no macro envelope.
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    for (name, diagnostic) in REJECTED {
        packet.rejected(name, &mut packet.client(name), diagnostic)?;
    }
    let source = packet.fixture("provider");
    for profile in OLDER {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(profile, &mut packet.export(profile, &source, &older), ALIAS)?;
    }
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn const_parameter_clients_execute_natively() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "native", "provider", PROFILE)?;
    for name in ACCEPTED {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        assert_eq!(packet.native(name)?.status.code(), Some(42), "{name}");
    }
    Ok(())
}

#[test]
fn const_parameter_controls_older_readers_and_forged_terms_fail_by_name() -> Result {
    let packet = Packet::publish(GROUP, SPEC, "forged", "provider", PROFILE)?;
    for (label, control, diagnostic) in CONTROLS {
        let source = generated(&packet, label, control)?;
        let metadata = packet.folder.join(format!("{label}.rgcrate"));
        packet.rejected(
            label,
            &mut packet.export(PROFILE, &source, &metadata),
            diagnostic,
        )?;
    }
    let bytes = std::fs::read(&packet.metadata)?;
    // A version-twelve reader meets declaration tag 12 before any slot.
    packet.rejected(
        "relabelled",
        &mut packet.forged("relabelled", relabel(&bytes, 12), "tag")?,
        "unknown source declaration kind",
    )?;
    // 'z', usize 3 and i8 -1 (masked to 0xFF) each travel as one value term.
    let letter = value_term(&bytes, 0x7A)?;
    let three = value_term(&bytes, 3)?;
    value_term(&bytes, 0xFF)?;
    let cases = [
        (
            "term-tag",
            forged(&bytes, letter + 1, &[2]),
            "unknown source const term",
        ),
        (
            "surrogate",
            forged(&bytes, letter + 6, &0xD800_u128.to_le_bytes()),
            "non-canonical source const value",
        ),
        (
            "wide-usize",
            forged(&bytes, three + 6, &(1_u128 << 64 | 3).to_le_bytes()),
            "non-canonical source const value",
        ),
    ];
    for (label, changed, diagnostic) in cases {
        packet.rejected(
            label,
            &mut packet.forged(label, changed, "tag")?,
            diagnostic,
        )?;
    }
    Ok(())
}
