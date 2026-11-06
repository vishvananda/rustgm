//! Real original body/row ownership survives consumption; malformed batches cannot start.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<(Declarations, Program)> {
    let text = crate::test_source::fixture("pa31/tests/spec/source-const-cleanup/mfield.rs")?;
    let report = crate::test_source::source_provider(&text, "actual_original_rows")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let program = crate::body::build_concrete(&report, &facts)?;
    Ok((facts, program))
}
#[test]
fn constructor_free_struct_reads_keep_their_actual_original_requirement() -> Result {
    let (facts, mut program) = fixture()?;
    let count = program.bodies.len();
    let mut original = Originals::take(&facts, &mut program)?;
    assert_eq!(original.bodies().len(), count);
    assert!(program.bodies.is_empty());
    assert!(program.drops.as_ref().ok_or("pending")?.bodies.is_empty());
    let mut found = false;
    while let Some((body, requirements)) = original.pop()? {
        let row = requirements.ok_or("actual ordinary body row")?;
        for (index, value) in body.values.iter().enumerate() {
            if matches!(value.kind, crate::body::ir::Rvalue::Read(_))
                && matches!(facts.types.kind(value.ty), Some(crate::types::Kind::Adt(item, _))
                    if facts.aggregates.get(item).is_some_and(|d|d.kind==crate::types::aggregate::Kind::Struct))
                && row.values.get(index) == Some(&true)
            {
                found = true;
            }
        }
    }
    assert!(
        found,
        "no constructor-free struct read retained its paid row"
    );
    assert!(original.pop()?.is_none());
    Ok(())
}
#[test]
fn missing_extra_and_wrong_extent_rows_fail_before_body_consumption() -> Result {
    for mode in ["missing", "extra", "extent"] {
        let (facts, mut program) = fixture()?;
        let count = program.bodies.len();
        let rows = &mut program.drops.as_mut().ok_or("pending")?.bodies;
        match mode {
            "missing" => {
                rows.pop().ok_or("row")?;
            }
            "extra" => rows.push(rows.first().ok_or("row")?.clone()),
            _ => rows.first_mut().ok_or("row")?.values.push(false),
        }
        assert!(Originals::take(&facts, &mut program).is_err(), "{mode}");
        assert_eq!(program.bodies.len(), count);
    }
    Ok(())
}
