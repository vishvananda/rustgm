//! Fresh CLI processes consume version-thirteen operator language roles, trait
//! parameter defaults, checked Add/Sub/Mul rows and `Add` with its declared
//! `Output`, and apply imported roles themselves; forged role rows reject.
use super::fresh_packets::{Packet, Result, relabel, unique, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, os::unix::process::ExitStatusExt};
const SPEC: &str = "pa29/tests/spec/language-roles";
const GROUP: &str = "language-roles";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/language-role-outcomes.json`.
const IN_RANGE: [&str; 3] = ["bump-in-range", "lower-in-range", "scale-in-range"];
const OVERFLOW: [&str; 3] = ["bump-overflow", "lower-overflow", "scale-overflow"];
/// The spelled `AddAssign` role row: escape tag, u32 length, then the name.
const SPELLED: &[u8] = b"\xff\x0a\x00\x00\x00add_assign";
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
/// Offsets of value nodes `Binary [Value, Value, Flag(operator), Flag]`.
fn binary_nodes(bytes: &[u8], operator: u8) -> Vec<usize> {
    (0..bytes.len().saturating_sub(22))
        .filter(|&at| {
            bytes[at..at + 3] == [27, 4, 0]
                && bytes[at + 7] == 0
                && bytes[at + 12] == 8
                && bytes[at + 13..at + 17] == [operator, 0, 0, 0]
                && bytes[at + 17] == 8
        })
        .collect()
}
fn spelled(bytes: &[u8], at: usize, name: &[u8; 10]) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[at + 5..at + 15].copy_from_slice(name);
    changed
}
#[test]
fn operator_roles_defaults_and_checked_rows_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    for name in [&b"add_assign"[..], b"sub_assign", b"mul_assign"] {
        let mut row = b"\xff\x0a\x00\x00\x00".to_vec();
        row.extend_from_slice(name);
        unique(&bytes, &row)?;
    }
    // bump and AddAssign for i32; lower twice and SubAssign; scale and MulAssign.
    for (operator, count) in [(9, 2), (10, 3), (11, 2)] {
        assert_eq!(binary_nodes(&bytes, operator).len(), count, "{operator}");
    }
    for name in IN_RANGE.iter().chain(&OVERFLOW) {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    packet.rejected(
        "mismatch",
        &mut packet.client("mismatch"),
        "incompatible types",
    )?;
    // Amended boundary: the consumer's own `+=` resolves through the imported
    // role's validated declaration identity and lowers one checked add.
    super::artifacts::checked(
        "own-operator",
        &packet.folder,
        &mut packet.client("own-operator"),
    )?;
    let text = std::fs::read_to_string(packet.folder.join("own-operator.lowir"))?;
    assert_eq!(text.matches(" = binary add i32 ").count(), 2);
    // Amended boundary: `Add` with its associated `Output` now exports through
    // version-thirteen declared results, and its consumer instantiates `sum`.
    let output = Packet::publish(GROUP, SPEC, "output", "output", PROFILE)?;
    super::artifacts::checked(
        "sum-in-range",
        &output.folder,
        &mut output.client("sum-in-range"),
    )?;
    own_controls(&output)?;
    let source = packet.fixture("provider");
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &source, &older),
            "source compiler language contract transport is not implemented",
        )?;
        assert!(!older.exists() || std::fs::metadata(older)?.len() == 0);
    }
    Ok(())
}
/// Generated controls from the frozen clients, run against `output.rs`. The
/// consumer's own `+` uses the imported `Add` role and its declared `Output`
/// (rustc 1.98.0 exits 42). Without an imported `add_assign` role its own `+=`
/// stays rejected by name, whatever the spelling (rustc E0368).
fn own_controls(output: &Packet) -> Result {
    let add = output.folder.join("own-add.rs");
    std::fs::write(
        &add,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::sum(1i32, 40i32, 0i32).count + 1i32 + 1i32 }\n",
    )?;
    let mut consumer = output.consumer(&add, "own-add", &output.metadata, &output.owner());
    super::artifacts::checked("own-add", &output.folder, &mut consumer)?;
    let text = std::fs::read_to_string(output.folder.join("own-add.lowir"))?;
    // Two own checked adds and the `sum` instance's own.
    assert_eq!(text.matches(" = binary add i32 ").count(), 3);
    let missing = output.folder.join("own-missing.rs");
    let source = std::fs::read_to_string(output.fixture("own-operator"))?;
    std::fs::write(&missing, source.replace("base::bump", "base::sum"))?;
    output.rejected(
        "own-missing",
        &mut output.consumer(&missing, "own-missing", &output.metadata, &output.owner()),
        "source binary operator requires its actual language declaration",
    )
}
#[test]
fn forged_role_rows_and_older_wires_reject_in_fresh_consumers() -> Result {
    let packet = publish("wire")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let role = unique(&bytes, SPELLED)?;
    let [add, ..] = binary_nodes(&bytes, 9)[..] else {
        return Err("no Add node".into());
    };
    let (client, own) = (
        packet.fixture("bump-in-range"),
        packet.fixture("own-operator"),
    );
    for (name, wire, diagnostic) in [
        (
            "unknown-role",
            spelled(&bytes, role, b"add_assigx"),
            "unknown source language role spelling",
        ),
        (
            "duplicate-role",
            spelled(&bytes, role, b"sub_assign"),
            "source role binding is duplicated or belongs to a foreign provider",
        ),
        (
            "wrong-kind",
            spelled(&bytes, role, b"panic_impl"),
            "imported declaration changed its exact identity or kind",
        ),
        (
            "missing-declaration",
            with_id(&bytes, role + 19, 0xffff_fff0),
            "source role binding names an unknown declaration",
        ),
        (
            "division",
            with_id(&bytes, add + 13, 12),
            "encoded checked division or shift Rvalue::Binary is not reconstructable by this consumer",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let mut consumer = packet.consumer(&client, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
        // The consumer's own `+=` trusts the forged role no more than imports do.
        let label = format!("{name}-own");
        let mut consumer = packet.consumer(&own, &label, &path, &packet.owner());
        packet.rejected(&label, &mut consumer, diagnostic)?;
    }
    assert!(crates::decode(&relabel(&bytes, 12)).is_err());
    let relabelled = packet.folder.join("relabelled.rgcrate");
    std::fs::write(&relabelled, relabel(&bytes, 12))?;
    let mut consumer = packet.consumer(&client, "relabelled", &relabelled, &packet.owner());
    assert_eq!(consumer.output()?.status.code(), Some(1));
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}

#[test]
fn division_round_trips_and_preserves_checked_consumers() -> Result {
    let packet = Packet::new(GROUP, SPEC, "primitive-division", "division")?;
    let fixture = std::fs::read_to_string(packet.fixture("output"))?;
    let (foundation, _) = fixture
        .split_once("// Authored boundary control")
        .ok_or("division role foundation is missing")?;
    let source = packet.folder.join("provider.rs");
    let body = r#"
#[lang = "sub"]
pub trait Sub<Rhs = Self> { type Output; fn sub(self, rhs: Rhs) -> Self::Output; }
impl Sub for f64 { type Output = f64; fn sub(self, rhs: f64) -> f64 { self - rhs } }
#[lang = "div"]
pub const trait Div<Rhs = Self> { type Output; fn div(self, rhs: Rhs) -> Self::Output; }
const impl Div for f64 { type Output = f64; fn div(self, rhs: f64) -> f64 { self / rhs } }
const impl Div for i64 { type Output = i64; fn div(self, rhs: i64) -> i64 { self / rhs } }
pub fn mantissa<T>(bits: u32) -> i64 {
    let log2_5 = 3.32192809488736234787031942948939018_f64 - 1.0;
    (bits as f64 / log2_5) as i64
}
pub fn checked<T>(numerator: i64, denominator: i64) -> i64 { numerator / denominator }
pub const fn quotient<T>(numerator: u32, denominator: f64) -> i64 {
    (numerator as f64 / denominator) as i64
}
"#;
    std::fs::write(
        &source,
        format!("#![feature(const_trait_impl)]\n{foundation}{body}"),
    )?;
    let legacy = packet.folder.join("legacy.rgcrate");
    packet.rejected(
        "legacy",
        packet
            .export(PROFILE, &source, &legacy)
            .arg("--cap-lints=allow"),
        "source Div contract transport is not implemented",
    )?;
    super::artifacts::checked(
        "provider",
        &packet.folder,
        packet
            .export("module-body-operations-v14", &source, &packet.metadata)
            .arg("--cap-lints=allow"),
    )?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(crates::encode(&crates::decode(&bytes)?)?, bytes);
    assert_eq!(binary_nodes(&bytes, 12).len(), 5);
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        r#"
#![feature(no_core)]
#![no_core]
extern crate base;
const IMPORTED: i64 = base::quotient::<u8>(55, 2.5);
#[unsafe(no_mangle)]
pub extern "C" fn mantissa(bits: u32) -> i64 { base::mantissa::<u8>(bits) }
#[unsafe(no_mangle)]
pub extern "C" fn checked(numerator: i64, denominator: i64) -> i64 {
    base::checked::<u32>(numerator, denominator)
}
#[unsafe(no_mangle)]
pub extern "C" fn from_const() -> i64 { IMPORTED }
"#,
    )?;
    super::artifacts::checked(
        "client",
        &packet.folder,
        packet
            .consumer(&client, "client", &packet.metadata, &packet.owner())
            .arg("--cap-lints=allow"),
    )?;
    let text = std::fs::read_to_string(packet.folder.join("client.lowir"))?;
    assert!(text.contains(" = binary div f64 "), "{text}");
    assert!(text.contains(" = binary div i64 "), "{text}");
    assert!(text.contains("call void @abort()"), "{text}");
    assert!(text.contains("return i64 22"), "{text}");
    Ok(())
}

#[test]
fn checked_rows_and_fresh_consumers_grow_linearly_with_operators() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "steps")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("pub struct Counted")
        .next()
        .ok_or("operator foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::steps(1i32, 1i32, 0i32) }\n",
    )?;
    let (mut sizes, mut lowir, mut owned) = (Vec::new(), Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let steps = (0..width).try_fold(String::new(), |mut text, _| {
            write!(text, "total += step; ").map(|()| text)
        })?;
        let source = packet.folder.join(format!("steps{width}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}// Generated scaling control.\npub fn steps<T: Copy>(value: T, step: i32, total: i32) -> i32 {{ let mut total = total; {steps}total }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("steps{width}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        // One Add node per statement, plus the AddAssign for i32 method body.
        assert_eq!(binary_nodes(&std::fs::read(&metadata)?, 9).len(), width + 1);
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        // Each fresh instance lowers one checked add and its guard per statement.
        assert_eq!(text.matches(" = binary add i32 ").count(), width, "{label}");
        assert!(text.contains("call void @abort()"), "{label}");
        sizes.push(std::fs::metadata(&metadata)?.len());
        lowir.push(text.len());
        // The consumer's own `+=` statements resolve the imported role per use.
        let own = packet.folder.join(format!("own{width}.rs"));
        std::fs::write(
            &own,
            format!(
                "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 {{ let mut total = base::steps(1i32, 1i32, 0i32); {}total }}\n",
                steps.replace("step", "1i32")
            ),
        )?;
        let label = format!("own{width}");
        let mut consumer = packet.consumer(&own, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        assert_eq!(
            text.matches(" = binary add i32 ").count(),
            2 * width,
            "{label}"
        );
        owned.push(text.len());
    }
    // Fixed-width nodes: every statement adds the same wire bytes.
    let ([small, middle, large], [first, second, third]) = (
        <[u64; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(third - second <= 5 * (second - first), "{lowir:?}");
    let [first, second, third] = <[usize; 3]>::try_from(owned.as_slice())?;
    assert!(third - second <= 5 * (second - first), "{owned:?}");
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn operator_role_instances_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in IN_RANGE.iter().chain(&OVERFLOW) {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        if IN_RANGE.contains(name) {
            assert_eq!(output.status.code(), Some(42), "{name}");
        } else {
            // The shared abort exit: SIGABRT, as the rustc oracle's observer.
            assert_eq!(output.status.signal(), Some(6), "{name}");
        }
        assert!(output.stdout.is_empty(), "{name}");
    }
    let output = Packet::publish(GROUP, SPEC, "output-native", "output", PROFILE)?;
    super::artifacts::checked(
        "sum-in-range",
        &output.folder,
        &mut output.client("sum-in-range"),
    )?;
    assert_eq!(output.native("sum-in-range")?.status.code(), Some(42));
    super::artifacts::checked(
        "own-operator",
        &packet.folder,
        &mut packet.client("own-operator"),
    )?;
    assert_eq!(packet.native("own-operator")?.status.code(), Some(42));
    Ok(())
}
