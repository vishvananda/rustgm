//! A selected callable cannot authorize a different graph or caller permission.
use super::*;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn prepare() -> TestResult<(Report, Declarations, Method)> {
    let text =
        crate::test_source::fixture("pa31/tests/spec/async-calls/implicit-method-function.rs")?;
    let report = crate::test_source::source_provider(&text, "entry_wrappers")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let method = crate::lower::instances::inspect_prepared(
        &report,
        &mut facts,
        prepared,
        |facts, checked| {
            checked.program().bodies.iter().find_map(|body| {
                facts
                    .primitive_methods
                    .as_deref()?
                    .get_lending(body.instance?)
            })
        },
    )?
    .ok_or("missing actual completed async wrapper selection")?;
    Ok((report, facts, method))
}

#[test]
fn actual_invocation_graph_cannot_be_replaced_with_a_constant_or_forged_contract() -> TestResult {
    let (report, mut facts, selected) = prepare()?;
    let (body, input) = build(&report, &mut facts, selected)?;
    let permit = validate(&mut facts, selected, &body, &input)?;
    assert_eq!(
        super::super::factories::Permit::Lending(permit).instance(),
        Some(selected.instance())
    );
    for case in 0..12 {
        let (mut body, mut input) = build(&report, &mut facts, selected)?;
        match case {
            0 => body.values[2].kind = Rvalue::Unit,
            1 => body.arguments.swap(0, 1),
            2 => body.statements.swap(3, 4),
            3 => body.projections[0].base = Place::Local(LocalId(0)),
            4 => body.locals[1].storage_parent = None,
            5 => body.blocks[0].terminator = Terminator::Return(ValueId(0)),
            6 => input.universals += 1,
            7 => input.assumptions.push((RegionId::STATIC, RegionId(7))),
            8 => input.type_assumptions.push(
                crate::types::parameters::predicates::Predicate {
                    ty: TypeId::primitive(crate::types::Primitive::U8),
                    bound: crate::types::parameters::predicates::Bound::Copy,
                }
                .into(),
            ),
            9 => body.storage = None,
            10 => body.instance = None,
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
fn source_method_header_and_public_substitution_remain_authoritative() -> TestResult {
    let (report, mut facts, selected) = prepare()?;
    let (body, input) = build(&report, &mut facts, selected)?;
    let source = selected.validate(&mut facts)?;
    let index = *facts
        .function_indices
        .get(&source.definition)
        .ok_or("actual method header")?;
    facts.functions[index].signature = TypeId::UNIT;
    assert!(validate(&mut facts, selected, &body, &input).is_err());
    Ok(())
}
