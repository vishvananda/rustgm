//! Fresh CLI processes re-derive version-thirteen declared results: trait
//! result headers, impl definitions and projections. Forged result rows reject.
use super::fresh_packets::{Packet, Result, relabel, with_id};
use rustgm::support::crates;
use std::{fmt::Write as _, os::unix::process::ExitStatusExt};
const SPEC: &str = "pa29/tests/spec/associated-results";
const GROUP: &str = "associated-results";
const PROFILE: &str = "module-bodies-v13";
/// Frozen outcomes from `pa29/tests/associated-result-outcomes.json`.
const IN_RANGE: [&str; 2] = ["fetch-hit", "sum-in-range"];
const OVERFLOW: &str = "sum-overflow";
const ASSOCIATED_ROW: usize = 39;
const DEFINITION_ROW: usize = 8;
const PROJECTION_ROW: usize = 24;
fn publish(label: &str) -> Result<Packet> {
    Packet::publish(GROUP, SPEC, label, "provider", PROFILE)
}
fn count(bytes: &[u8], at: usize) -> Option<usize> {
    let raw = bytes.get(at..at.checked_add(4)?)?;
    usize::try_from(u32::from_le_bytes(raw.try_into().ok()?)).ok()
}
/// The trailing result columns of a packet without a const-body trailer:
/// associated rows, then definition rows, then projection rows. Returns the
/// offset of the first definition row and the three row counts.
fn columns(bytes: &[u8]) -> Result<(usize, [usize; 3])> {
    let mut found = Vec::new();
    for projections in 0..=256 {
        let Some(end) = bytes.len().checked_sub(4 + PROJECTION_ROW * projections) else {
            break;
        };
        if count(bytes, end) != Some(projections) {
            continue;
        }
        for definitions in 1..=256 {
            let Some(start) = end.checked_sub(4 + DEFINITION_ROW * definitions) else {
                break;
            };
            if count(bytes, start) != Some(definitions) {
                continue;
            }
            for associated in 1..=256 {
                let Some(head) = start.checked_sub(4 + ASSOCIATED_ROW * associated) else {
                    break;
                };
                // The nearest consistent count; callers assert the exact counts.
                if count(bytes, head) == Some(associated) {
                    found.push((start + 4, [associated, definitions, projections]));
                    break;
                }
            }
        }
    }
    match found[..] {
        [one] => Ok(one),
        _ => Err(format!("result columns are absent or ambiguous: {found:?}").into()),
    }
}
#[test]
fn declared_results_survive_wire_and_fresh_consumers() -> Result {
    let packet = publish("frontend")?;
    let bytes = std::fs::read(&packet.metadata)?;
    assert_eq!(bytes.get(13..15), Some([13, 0].as_slice()));
    let interface = crates::decode(&bytes)?;
    assert_eq!(crates::encode(&interface)?, bytes);
    // Add::Output and Shape::Item; their i32/u8 impl definitions; the two
    // trait-frame projections and the generic `fetch`/`sum` results.
    assert_eq!(columns(&bytes)?.1, [2, 2, 4]);
    for name in IN_RANGE.iter().chain([&OVERFLOW]) {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
    }
    packet.rejected(
        "fetch-mismatch",
        &mut packet.client("fetch-mismatch"),
        "incompatible types",
    )?;
    packet.rejected(
        "fetch-unimplemented",
        &mut packet.client("fetch-unimplemented"),
        "associated type requires an unambiguous validated implementation",
    )?;
    let source = packet.fixture("provider");
    for profile in ["module-branches-v12", "module-nominals-v10"] {
        let older = packet.folder.join(format!("{profile}.rgcrate"));
        packet.rejected(
            profile,
            &mut packet.export(profile, &source, &older),
            "named, associated or generic alias transport requires its wider profile",
        )?;
        assert!(!older.exists() || std::fs::metadata(older)?.len() == 0);
    }
    Ok(())
}
#[test]
fn forged_result_rows_and_older_wires_reject_in_fresh_consumers() -> Result {
    let packet = publish("wire")?;
    let bytes = std::fs::read(&packet.metadata)?;
    let (first, [associated, ..]) = columns(&bytes)?;
    let field = |at: usize| count(&bytes, at).and_then(|value| u32::try_from(value).ok());
    let second = first + DEFINITION_ROW;
    let (add, shape) = (
        field(first).ok_or("Add row")?,
        field(second).ok_or("Shape row")?,
    );
    let value = field(second + 4).ok_or("Shape value")?;
    // Shape's own trait-frame result header, declared after Add::Output:
    // its declaration and its canonical projection type.
    let header = first - 4 - ASSOCIATED_ROW;
    let trait_result = field(header).ok_or("trait result")?;
    let projection = field(header + 12).ok_or("trait projection")?;
    let swapped = with_id(&with_id(&bytes, first, shape), second, add);
    let client = packet.fixture("fetch-hit");
    for (name, wire, diagnostic) in [
        (
            "swapped-definitions",
            swapped,
            "source associated definition belongs to another actual Self owner",
        ),
        (
            "trait-result-definition",
            with_id(&bytes, second, trait_result),
            "source declaration has duplicate or conflicting typed headers",
        ),
        (
            "trait-frame-value",
            with_id(&bytes, second + 4, projection),
            "source consuming type root captures another parameter owner",
        ),
        (
            "missing-value",
            with_id(&bytes, second + 4, 0xffff_fff0),
            "associated result definition lost its actual implementation owner",
        ),
        (
            "dropped-columns",
            bytes[..first - 4 - ASSOCIATED_ROW * associated - 4].to_vec(),
            "truncated metadata artifact",
        ),
    ] {
        let path = packet.folder.join(format!("{name}.rgcrate"));
        std::fs::write(&path, wire)?;
        let mut consumer = packet.consumer(&client, name, &path, &packet.owner());
        packet.rejected(name, &mut consumer, diagnostic)?;
    }
    // Every other in-range value recipe rejects: foreign frames structurally,
    // and the impl's own u8 Self type by the consumer's fresh conformance.
    let (mut conformance, mut values) = (0, 0_u32);
    loop {
        if values != value {
            let path = packet.folder.join(format!("value-{values}.rgcrate"));
            std::fs::write(&path, with_id(&bytes, second + 4, values))?;
            let label = format!("value-{values}");
            let output = packet
                .consumer(&client, &label, &path, &packet.owner())
                .output()?;
            let text = String::from_utf8_lossy(&output.stderr);
            assert_eq!(output.status.code(), Some(1), "{label}: {text}");
            if text.contains("associated result definition lost its actual implementation owner") {
                break;
            }
            conformance += usize::from(text.contains("incompatible types"));
        }
        values += 1;
        assert!(values < 1024, "value table end");
    }
    assert_eq!(conformance, 1);
    assert!(crates::decode(&relabel(&bytes, 12)).is_err());
    let relabelled = packet.folder.join("relabelled.rgcrate");
    std::fs::write(&relabelled, relabel(&bytes, 12))?;
    let mut consumer = packet.consumer(&client, "relabelled", &relabelled, &packet.owner());
    packet.rejected(
        "relabelled",
        &mut consumer,
        "unknown source declaration kind",
    )?;
    for end in 0..bytes.len() {
        assert!(crates::decode(&bytes[..end]).is_err(), "prefix {end}");
    }
    Ok(())
}
#[test]
fn result_columns_and_fresh_consumers_grow_linearly_with_declared_results() -> Result {
    let packet = Packet::new(GROUP, SPEC, "scaling", "shapes")?;
    let fixture = std::fs::read_to_string(packet.fixture("provider"))?;
    let foundation = fixture
        .split("pub trait Shape")
        .next()
        .ok_or("result foundation")?;
    let client = packet.folder.join("client.rs");
    std::fs::write(
        &client,
        "#![feature(no_core)]\n#![no_core]\nextern crate base;\n#[unsafe(no_mangle)]\npub extern \"C\" fn entry() -> i32 { base::fetch(1u8) }\n",
    )?;
    let (mut sizes, mut lowir) = (Vec::new(), Vec::new());
    for width in [6, 24, 96] {
        let traits = (0..width).try_fold(String::new(), |mut text, index| {
            writeln!(
                text,
                "pub trait Shape{index:03} {{ type Item; fn get(self) -> Self::Item; }}\nimpl Shape{index:03} for u8 {{ type Item = i32; fn get(self) -> i32 {{ 42 }} }}"
            )
            .map(|()| text)
        })?;
        let source = packet.folder.join(format!("shapes{width}.rs"));
        std::fs::write(
            &source,
            format!(
                "{foundation}// Generated scaling control.\n{traits}pub fn fetch<T: Shape000>(value: T) -> <T as Shape000>::Item {{ value.get() }}\n"
            ),
        )?;
        let metadata = packet.folder.join(format!("shapes{width}.rgcrate"));
        let mut export = packet.export(PROFILE, &source, &metadata);
        super::artifacts::checked("provider", &packet.folder, &mut export)?;
        let bytes = std::fs::read(&metadata)?;
        // One trait result, one impl definition and one trait-frame projection
        // per generated trait, plus Add::Output and `fetch`'s own projection.
        assert_eq!(columns(&bytes)?.1, [width + 1, width + 1, width + 2]);
        let label = format!("client{width}");
        let mut consumer = packet.consumer(&client, &label, &metadata, &packet.owner());
        super::artifacts::checked(&label, &packet.folder, &mut consumer)?;
        let text = std::fs::read_to_string(packet.folder.join(format!("{label}.lowir")))?;
        sizes.push(bytes.len());
        lowir.push(text.len());
    }
    // Fixed-width rows and fixed-length names: every trait adds the same bytes.
    let ([small, middle, large], [first, second, third]) = (
        <[usize; 3]>::try_from(sizes.as_slice())?,
        <[usize; 3]>::try_from(lowir.as_slice())?,
    );
    assert_eq!((middle - small) * 4, large - middle, "{sizes:?}");
    assert!(
        third.abs_diff(second) <= 5 * second.abs_diff(first).max(64),
        "{lowir:?}"
    );
    Ok(())
}
#[test]
#[ignore = "requires configured locked CPPGM object backend and host C compiler"]
fn declared_result_instances_execute_natively() -> Result {
    let packet = publish("native")?;
    for name in IN_RANGE.iter().chain([&OVERFLOW]) {
        super::artifacts::checked(name, &packet.folder, &mut packet.client(name))?;
        let output = packet.native(name)?;
        if *name == OVERFLOW {
            // The shared abort exit: SIGABRT, as the rustc oracle's observer.
            assert_eq!(output.status.signal(), Some(6), "{name}");
        } else {
            assert_eq!(output.status.code(), Some(42), "{name}");
        }
        assert!(output.stdout.is_empty(), "{name}");
    }
    Ok(())
}
