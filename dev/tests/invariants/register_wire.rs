//! Version thirteen's inline assembly columns decode only the initial
//! assembly profile. Forged copies of the provider's own `store` columns are
//! rejected at decode by the profile's own texts, and no reserved payload is
//! read; older writers refuse the value by name.
use super::*;
use crate::{lower::instances::SourcePublication, support::initial_assembly as profile};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Row<T> = std::result::Result<T, &'static str>;

const PROVIDER: &str = "pa41/tests/spec/asm-transport/provider.rs";
const BODIES: s::Profile = s::Profile::Source2024ModuleBodies;

/// The provider's own `store` original: one instruction, one template, one
/// piece and one pointer input.
fn store() -> Result<s::Original> {
    let source = crate::test_source::fixture(PROVIDER)?;
    let interface = crate::module_packets::exported_profile(
        &source,
        SourcePublication::ModuleBodies,
        "asm_transport",
    )?;
    let decoded = crates::decode(&crates::encode(&interface)?)?;
    let (rows, _, _) = crate::module_packets::raw(&decoded)?;
    let found: Vec<s::Original> = rows
        .originals
        .into_iter()
        .filter(|original| {
            original.assembly.as_deref().is_some_and(|rows| {
                rows.templates
                    .first()
                    .is_some_and(|template| template.text.starts_with("mov "))
            })
        })
        .collect();
    let [original] = <[s::Original; 1]>::try_from(found).map_err(|_| "one store original")?;
    Ok(original)
}
fn columns(original: &s::Original) -> Result<s::Assembly> {
    Ok(original.assembly.as_deref().ok_or("store columns")?.clone())
}
fn written(rows: &s::Assembly) -> Result<Vec<u8>> {
    let mut out = Writer::default();
    instruction_columns::assembly_write(&mut out, rows)?;
    Ok(out.finish())
}
fn read(bytes: &[u8]) -> Row<s::Assembly> {
    let mut reader = Reader::new(bytes);
    let rows = instruction_columns::assembly_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile: BODIES,
    })?;
    if !reader.done() {
        return Err("assembly columns left trailing bytes");
    }
    Ok(rows)
}
/// A typed forgery of one column row, written and decoded again.
fn forged(rows: &s::Assembly, forge: impl FnOnce(&mut s::Assembly)) -> Result<Row<s::Assembly>> {
    let mut forgery = rows.clone();
    forge(&mut forgery);
    Ok(read(&written(&forgery)?))
}
/// Offsets inside the honest single-row columns: the instruction count, then
/// `site | templates | operands | options`, then the safety byte; the operand
/// is the last row, `class | input | output | ty | late`.
const SAFETY: usize = 4 + 4 + 8 + 8 + 2;
const CLASS_FROM_END: usize = 1 + 5 + 1 + 4 + 1;

#[test]
fn store_columns_round_trip_and_older_writers_refuse_assembly() -> Result {
    let original = store()?;
    let rows = columns(&original)?;
    assert_eq!(rows.instructions.len(), 1);
    assert_eq!(rows.operands.len(), 1);
    assert_eq!(
        rows.instructions[0].options,
        profile::NOSTACK | profile::PRESERVES_FLAGS
    );
    assert_eq!(
        rows.instructions[0].safety,
        s::CallSafety::ExplicitUnsafeBlock
    );
    assert_eq!(rows.operands[0].class, s::AssemblyClass::General);
    let bytes = written(&rows)?;
    assert_eq!(read(&bytes)?, rows);
    assert_eq!(bytes[SAFETY], 1);
    assert_eq!(bytes[bytes.len() - CLASS_FROM_END], 0);
    // The whole original keeps its exact bytes under the body profile.
    let mut out = Writer::default();
    operation_columns::body_write(&mut out, &original)?;
    let whole = out.finish();
    let mut reader = Reader::new(&whole);
    let decoded = operation_columns::original_read(&mut Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile: BODIES,
    })?;
    assert!(reader.done());
    assert_eq!(decoded, original);
    let mut older = Writer::default();
    assert_eq!(
        operation_columns::branch_write(&mut older, &original).err(),
        Some("source inline assembly requires the version-thirteen body IR profile")
    );
    // Presence is derived from the values, never from trailing bytes.
    let mut detached = original.clone();
    detached.assembly = None;
    let mut out = Writer::default();
    assert_eq!(
        operation_columns::body_write(&mut out, &detached).err(),
        Some("source assembly columns differ from the assembly values of its body")
    );
    Ok(())
}
#[test]
fn forged_store_columns_reject_by_profile_text_at_decode() -> Result {
    let rows = columns(&store()?)?;
    // Rows 1 (tag 1) and 2 (tag 4), and the reserved byte tag 2: a reserved
    // class is refused before any payload, whatever bytes follow it, and the
    // writer has no layout for one. Row 3: an unallocated class tag.
    let honest = written(&rows)?;
    let at = honest.len() - CLASS_FROM_END;
    for (tag, class, text) in [
        (1, Some(s::AssemblyClass::Abcd), profile::ABCD),
        (2, Some(s::AssemblyClass::Byte), profile::BYTE),
        (4, Some(s::AssemblyClass::Explicit), profile::EXPLICIT),
        (5, None, "unknown source assembly register class"),
    ] {
        let mut bytes = honest.clone();
        bytes[at] = tag;
        assert_eq!(read(&bytes).err(), Some(text), "class tag {tag}");
        if let Some(class) = class {
            let mut forgery = rows.clone();
            forgery.operands[0].class = class;
            assert_eq!(
                written(&forgery).err().map(|e| e.to_string()),
                Some(text.into())
            );
        }
    }
    // Rows 5 and 6: an undefined option bit, then a defined one outside the
    // profile, which keeps its lowering text.
    assert_eq!(
        forged(&rows, |rows| rows.instructions[0].options |= 0x100)?.err(),
        Some("unknown source assembly option")
    );
    assert_eq!(
        forged(&rows, |rows| rows.instructions[0].options |= profile::NOMEM)?.err(),
        Some(profile::OPTIONS[1].1)
    );
    // Row 7: an output flag; no output place is read.
    assert_eq!(
        forged(&rows, |rows| rows.operands[0].output = true)?.err(),
        Some(profile::OUTPUT)
    );
    // Row 9 and the unallocated safety tags.
    assert_eq!(
        forged(&rows, |rows| rows.instructions[0].safety =
            s::CallSafety::Safe)?
        .err(),
        Some("source inline assembly has no unsafe context")
    );
    let mut bytes = written(&rows)?;
    bytes[SAFETY] = 3;
    assert_eq!(
        read(&bytes).err(),
        Some("unknown source inline assembly safety")
    );
    // Row 14: any modifier byte.
    assert_eq!(
        forged(&rows, |rows| rows.pieces[0].modifier = b'x')?.err(),
        Some(profile::MODIFIER)
    );
    Ok(())
}
#[test]
fn assembly_value_nodes_use_code_zero_with_flag_and_range_operands() -> Result {
    let value = s::Value {
        ty: g::TypeRef(3),
        kind: s::ValueKind::Assembly {
            instruction: 2,
            arguments: g::Range { start: 4, end: 6 },
        },
    };
    let mut out = Writer::default();
    ir_nodes::value_write(&mut out, &value)?;
    let bytes = out.finish();
    // The type, then code 0 with arity 2.
    assert_eq!(bytes.get(4..6), Some([0, 2].as_slice()));
    let mut reader = Reader::new(&bytes);
    let mut input = Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile: BODIES,
    };
    assert_eq!(ir_nodes::value_read(&mut input)?, value);
    // A code-zero node with other operands is malformed, not unknown.
    let mut malformed = bytes.clone();
    malformed[5] = 1;
    malformed.truncate(4 + 2 + 5);
    let mut reader = Reader::new(&malformed);
    let mut input = Input {
        reader: &mut reader,
        bytes: 0,
        base: 0,
        profile: BODIES,
    };
    assert_eq!(
        ir_nodes::value_read(&mut input).err(),
        Some("body IR value node has malformed operands")
    );
    Ok(())
}
