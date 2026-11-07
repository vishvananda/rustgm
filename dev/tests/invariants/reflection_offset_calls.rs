//! Warm storage cannot replace the exact proved source caller and its current call.
use super::*;
use crate::{
    resolve::expansion::Report,
    sema,
    types::{Kind, arguments::Argument},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn provider(input: &str) -> Result<Report> {
    let mut config = crate::expand::config::Config::default();
    config.set_lint_cap(crate::support::diagnostics::LintCap::Allow);
    Ok(crate::test_source::source_provider_config(
        input,
        "field_calls",
        &config,
    )?)
}
fn analyze(report: &Report) -> Result<crate::pipeline::Analysis> {
    let capture = crate::sema::copy_trace::Capture::new()?;
    let result = crate::pipeline::analyze_source(report);
    if result.is_err() {
        match capture.rows() {
            Ok(rows) => {
                for row in rows {
                    eprintln!("Field caller proof: {row}");
                }
            }
            Err(error) => eprintln!("Field proof trace: {error}"),
        }
    }
    Ok(result?)
}

#[test]
fn field_offsets_require_the_current_proved_caller_even_with_warm_layout() -> Result {
    let input = crate::test_source::fixture("pa31/tests/spec/field-operations/native.rs")?;
    let (prefix, _) = input
        .split_once("fn generic<T:Copy>")
        .ok_or("original generic boundary")?;
    let input = format!(
        "{prefix}\nstatic FIRST:usize=intrinsics::field_offset::<FRT<Record,0,0>>();static SECOND:usize=intrinsics::field_offset::<FRT<Record,0,1>>();"
    );
    let report = provider(&input)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = analyze(&report)?;
    let context = prepared.constants.as_ref().ok_or("constant provider")?;
    let mut observed = Vec::new();
    for name in ["FIRST", "SECOND"] {
        let definition = facts
            .initializers
            .iter()
            .find(|initializer| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                    == name
            })
            .ok_or("actual static")?
            .definition;
        let value = context.inspect_const_call(
            sema::values::execution::Target::Initializer(definition),
            |catalog, inputs, index| replay(&report, &mut facts, catalog, inputs, index),
        )??;
        observed.push(value);
    }
    assert_eq!(observed, [Scalar::Integer(0), Scalar::Integer(4)]);
    Ok(())
}

fn replay(
    report: &Report,
    facts: &mut Declarations,
    catalog: &Catalog,
    inputs: &Inputs,
    index: usize,
) -> Result<Scalar> {
    facts.layouts = crate::abi::layout::Layouts::default();
    let evaluator = sema::values::Evaluator::new(report, facts);
    let mut machine = Machine::source_catalog(
        evaluator,
        catalog,
        Some(inputs),
        super::super::Limits::default(),
    );
    machine.begin(index, &[]).map_err(|error| error.message)?;
    let sema::values::execution::Progress::Prepare(super::super::Requirement::FunctionBody(target)) =
        machine.advance().map_err(|error| error.message)?
    else {
        return Err("actual cold Field layout demand".into());
    };
    let (state, _) = machine.suspend();
    let frame = state.frames.last().ok_or("actual frame")?;
    let body = catalog.code(frame.body).ok_or("actual Code")?;
    let selected = hostile(facts, catalog, inputs, body, frame, target)?;
    facts
        .aggregates
        .prepare_storage_fields(&mut facts.types, selected.coordinates.base)?;
    facts
        .layouts
        .demand(&facts.types, &facts.aggregates, selected.coordinates.base)?;
    assert!(validate(facts, catalog, None, body, frame, frame.next, target).is_err());
    assert!(catalog.layout_result(target).is_none());
    let evaluator = sema::values::Evaluator::new(report, facts);
    let mut machine = Machine::source_catalog(
        evaluator,
        catalog,
        Some(inputs),
        super::super::Limits::default(),
    );
    machine.restore(state);
    let sema::values::execution::Progress::Complete(value) =
        machine.advance().map_err(|error| error.message)?
    else {
        return Err("actual warm Field result".into());
    };
    Ok(value.as_scalar().map_err(sema::values::Failure::message)?)
}

fn hostile(
    facts: &mut Declarations,
    catalog: &Catalog,
    inputs: &Inputs,
    body: &Code,
    frame: &Frame,
    target: Instance,
) -> Result<FieldOffsetSelection> {
    let selected = validate(
        facts,
        catalog,
        Some(inputs),
        body,
        frame,
        frame.next,
        target,
    )?
    .ok_or("actual Field offset call")?;
    assert!(validate(facts, catalog, None, body, frame, frame.next, target).is_err());
    assert!(
        validate(
            facts,
            &Catalog::default(),
            Some(inputs),
            body,
            frame,
            frame.next,
            target
        )
        .is_err()
    );
    assert!(
        validate(
            facts,
            catalog,
            Some(inputs),
            body,
            frame,
            frame.next + 1,
            target
        )
        .is_err()
    );
    let Some(Kind::FunctionItem {
        item, signature, ..
    }) = facts.types.kind(target.ty())
    else {
        return Err("actual intrinsic item".into());
    };
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U32))])?;
    let changed = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments,
    })?;
    assert!(
        validate(
            facts,
            catalog,
            Some(inputs),
            body,
            frame,
            frame.next,
            Instance::new(&facts.types, changed)?
        )
        .is_err()
    );
    Ok(selected)
}

#[test]
fn generic_field_projection_originals_preserve_nominal_formation() -> Result {
    let input = crate::test_source::fixture("pa31/tests/spec/field-operations/native.rs")?;
    analyze(&provider(&input)?)?;
    Ok(())
}
