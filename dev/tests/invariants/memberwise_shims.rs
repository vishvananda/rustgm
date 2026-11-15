//! Mutation opponents require the actual source `clone` and its exact member-wise body.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const PREFIX: &str = "pa31/tests/spec/source-copy-repeats/const-length.rs";
const ITEMS: &str = "\npub struct Step(u8);impl Clone for Step{fn clone(&self)->Self{Step(self.0)}}\n\
fn seven()->u8{7}\n\
fn pair(t:&(Step,Step))->(Step,Step){t.clone()}\n\
fn item()->u8{let g=seven.clone();g()}\n";

/// The actual closed `clone` call on a tuple, or the one on a function item.
fn prepare(memberwise: bool) -> TestResult<(Report, Declarations, Shim)> {
    let mut source = crate::test_source::fixture(PREFIX)?;
    source.push_str(ITEMS);
    let mut config =
        crate::expand::config::Config::in_edition(crate::support::edition::Edition::Rust2024);
    config.set_lint_cap(crate::support::diagnostics::LintCap::Allow);
    let report = crate::test_source::source_provider_config(&source, "clone_shim", &config)?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let method = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(crate::support::bindings::LangItem::CloneFn)
        .ok_or("actual clone member")?
        .definition;
    let callables = facts
        .methods
        .calls
        .values()
        .filter(|selection| selection.method == method)
        .map(|selection| selection.callable)
        .collect::<Vec<_>>();
    for callable in callables {
        if facts.types.contains_parameters(callable) != Some(false) {
            continue;
        }
        let instance = Instance::new(&facts.types, callable)?;
        if let Some(selected) = shim(&mut facts, instance)?
            && members(&facts.types, selected.subject)?.is_some() == memberwise
        {
            return Ok((report, facts, selected));
        }
    }
    Err("actual closed language Clone call".into())
}
#[test]
fn exact_shims_pass_and_forged_member_order_or_contract_cannot_get_a_permit() -> TestResult {
    let (report, mut facts, selected) = prepare(true)?;
    let (body, input) = build(&report, &mut facts, selected)?;
    let permit = validate(&mut facts, selected, &body, &input)?;
    assert_eq!(
        super::super::factories::Permit::Clone(permit).instance(),
        Some(selected.instance)
    );
    assert_eq!((body.values.len(), body.arguments.len()), (7, 4));
    for case in 0..10 {
        let (mut body, mut input) = build(&report, &mut facts, selected)?;
        match case {
            0 => body.arguments.swap(2, 3),
            1 => body.arguments.swap(0, 1),
            2 => body.values[2].kind = Rvalue::Read(Place::Projection(ProjectionId(1))),
            3 => body.values[6].kind = Rvalue::Unit,
            4 => body.projections.swap(1, 2),
            5 => body.locals[0].mutable = true,
            6 => body.result = TypeId::UNIT,
            7 => input.signature = Some(TypeId::UNIT),
            8 => input.universals += 1,
            _ => input.require_result_formation = true,
        }
        assert!(
            validate(&mut facts, selected, &body, &input).is_err(),
            "case={case}"
        );
    }
    let (report, mut facts, selected) = prepare(false)?;
    let (mut body, input) = build(&report, &mut facts, selected)?;
    validate(&mut facts, selected, &body, &input)?;
    assert_eq!((body.values.len(), body.arguments.len()), (1, 0));
    body.values[0].kind = Rvalue::Read(Place::Local(LocalId(0)));
    assert!(validate(&mut facts, selected, &body, &input).is_err());
    Ok(())
}
#[test]
fn changed_subject_or_source_header_cannot_authorize_the_shim() -> TestResult {
    let (report, mut facts, selected) = prepare(true)?;
    let (body, input) = build(&report, &mut facts, selected)?;
    let mut wrong = selected;
    wrong.subject = TypeId::UNIT;
    assert!(validate(&mut facts, wrong, &body, &input).is_err());
    let index = *facts
        .function_indices
        .get(&selected.definition)
        .ok_or("source header")?;
    facts.functions[index].signature = TypeId::UNIT;
    assert!(validate(&mut facts, selected, &body, &input).is_err());
    Ok(())
}
