//! Mutation opponents require the actual source method and exact no-call factory.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn prepare() -> TestResult<(Report, Declarations, Address)> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/function-pointers/native-address.rs")?;
    let report = crate::test_source::source_provider(&source, "fn_ptr_shim")?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    let method = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(crate::support::bindings::LangItem::FnPtrAddr)
        .ok_or("actual address member")?
        .definition;
    let callable = facts
        .methods
        .calls
        .values()
        .find_map(|selection| {
            (selection.method == method
                && facts.types.contains_parameters(selection.callable) == Some(false))
            .then_some(selection.callable)
        })
        .ok_or("actual closed source call")?;
    let instance = Instance::new(&facts.types, callable)?;
    let selected = address(&mut facts, instance)?.ok_or("actual pointer shim")?;
    Ok((report, facts, selected))
}
#[test]
fn exact_shim_passes_and_forged_body_or_lifetime_contract_cannot_get_a_permit() -> TestResult {
    let (report, mut facts, selected) = prepare()?;
    let (body, input) = build(&report, &mut facts, selected)?;
    let permit = validate(&mut facts, selected, &body, &input)?;
    assert_eq!(
        super::super::factories::Permit::FunctionPointer(permit).instance(),
        selected.instance
    );
    for case in 0..8 {
        let (mut body, mut input) = build(&report, &mut facts, selected)?;
        match case {
            0 => body.values[1].kind = Rvalue::Read(Place::Local(LocalId(0))),
            1 => {
                body.values[1].kind = Rvalue::Cast {
                    direct_variant: false,
                    kind: Cast::IntegerToPointer,
                    operand: ValueId(0),
                }
            }
            2 => body.locals[0].mutable = true,
            3 => body.result = TypeId::primitive(crate::types::Primitive::U8),
            4 => input.signature = Some(TypeId::UNIT),
            5 => input.universals += 1,
            6 => input.require_result_formation = true,
            _ => body.const_context = true,
        }
        assert!(
            validate(&mut facts, selected, &body, &input).is_err(),
            "case={case}"
        );
    }
    Ok(())
}
#[test]
fn changed_descriptor_or_source_function_header_cannot_authorize_the_factory() -> TestResult {
    let (report, mut facts, selected) = prepare()?;
    let (body, input) = build(&report, &mut facts, selected)?;
    let mut wrong = selected;
    wrong.receiver = TypeId::primitive(crate::types::Primitive::U8);
    assert!(validate(&mut facts, wrong, &body, &input).is_err());
    let index = *facts
        .function_indices
        .get(&selected.definition)
        .ok_or("source header")?;
    facts.functions[index].signature = TypeId::UNIT;
    assert!(validate(&mut facts, selected, &body, &input).is_err());
    Ok(())
}
