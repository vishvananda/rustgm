use rustgm::{
    body::{
        self,
        ir::{Action, Body, Rvalue},
        storage::Location,
    },
    borrow::ownership::{
        self,
        initialization::{self, Disposition, Moment, Status},
    },
};
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

fn verify_points(body: &Body, facts: &initialization::Facts<'_>) -> Result<usize, &'static str> {
    let mut seen = 0;
    let mut after = std::collections::HashMap::new();
    for effect in facts.consumption.entries() {
        if effect.moment == Moment::Statement {
            let Location::At { block, before } = effect.location else {
                return Err("edge consumption");
            };
            assert!(before < body.blocks[block.0 as usize].end);
            after
                .entry(Location::At {
                    block,
                    before: before + 1,
                })
                .or_insert_with(Vec::new)
                .push(effect.path);
        }
    }
    facts.visit(|location, state| {
        if let Some(paths) = after.get(&location) {
            for path in paths {
                assert_eq!(
                    state.status(&facts.ownership.paths.entries[path.0 as usize])?,
                    Status::Never
                );
            }
        }
        for effect in facts.consumption.at(location) {
            let expected = if effect.moment == Moment::Statement {
                Status::Always
            } else {
                Status::Never
            };
            assert_eq!(
                state.status(&facts.ownership.paths.entries[effect.path.0 as usize])?,
                expected
            );
            seen += 1;
        }
        Ok(())
    })?;
    assert_eq!(seen, facts.consumption.entries().len());
    Ok(seen)
}

#[test]
fn owned_operands_transfer_once_across_stores_calls_aggregates_and_returns()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("returned-value", "fn f()->D{D}"),
        ("argument", "fn take(_:D){}fn f(){take(D);}"),
        ("returned-call", "fn make()->D{D}fn f(){let _x=make();}"),
        ("tuple", "fn f(){let _x=(D,D);}"),
        ("constructor", "struct W(D);fn f(){let _x=W(D);}"),
        ("branch-result", "fn f(flag:bool)->D{if flag{D}else{D}}"),
        (
            "early-return",
            "fn f(flag:bool)->D{match flag{true=>D,false=>return D}}",
        ),
        (
            "nested-move",
            "fn f(){let _x=(D,D);let _y=_x.0;let _z=_x.1;}",
        ),
        ("replacement", "fn f(){let mut _x=D;_x=D;}"),
        (
            "loop-arguments",
            "fn take(_:D){}fn f(flag:bool){while flag{take(D);}}",
        ),
    ];
    for (name, case) in cases {
        let source = format!("struct D;impl Drop for D{{fn drop(&mut self){{}}}}{case}");
        rustc::qualify(name, &source, true)?;
        let (report, mut declarations) = semantic::analyze(&source)?;
        let program = body::build(&report, &declarations)?;
        let mut seen = 0;
        for body in &program.bodies {
            let facts = initialization::analyze(
                &declarations.types,
                &declarations.aggregates,
                &declarations.derives,
                &mut declarations.copies,
                body,
            )
            .map_err(|e| e.message)?;
            seen += verify_points(body, &facts)?;
            assert!(
                facts
                    .consumption
                    .entries()
                    .iter()
                    .all(|e| e.disposition == Disposition::Transfer)
            );
        }
        assert!(seen > 0, "{name}");
    }
    Ok(())
}

#[test]
fn repeat_operands_distinguish_disposal_transfer_and_constant_instantiation()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("zero-constructor", "D", 0, Some(Disposition::Discard)),
        ("zero-call", "make()", 0, Some(Disposition::Discard)),
        ("one-call", "make()", 1, Some(Disposition::Transfer)),
        ("zero-constant", "ITEM", 0, None),
        ("zero-grouped-constant", "(ITEM)", 0, None),
        (
            "zero-block-constant",
            "{ITEM}",
            0,
            Some(Disposition::Discard),
        ),
        ("one-constant", "ITEM", 1, Some(Disposition::Instantiate)),
        (
            "many-constant",
            "ITEM",
            1_000_000_000,
            Some(Disposition::Instantiate),
        ),
    ];
    for (name, operand, length, expected) in cases {
        let source = format!(
            "struct D;impl Drop for D{{fn drop(&mut self){{}}}}const ITEM:D=D;fn make()->D{{D}}fn f(){{let _x=[{operand};{length}];}}"
        );
        rustc::qualify(name, &source, true)?;
        let (report, mut declarations) = semantic::analyze(&source)?;
        let program = body::build(&report, &declarations)?;
        let mut seen = 0;
        for body in &program.bodies {
            if expected.is_none() {
                // A zero-length constant repeat never evaluates its operand;
                // suppressing only the Repeat action would still be incorrect.
                assert!(
                    body.values
                        .iter()
                        .all(|value| !matches!(value.kind, Rvalue::Constant(_)))
                );
            }
            let facts = initialization::analyze(
                &declarations.types,
                &declarations.aggregates,
                &declarations.derives,
                &mut declarations.copies,
                body,
            )
            .map_err(|e| e.message)?;
            verify_points(body, &facts)?;
            for effect in facts.consumption.entries() {
                let Location::At { before, .. } = effect.location else {
                    continue;
                };
                if effect.moment == Moment::Statement
                    && let Action::Evaluate(id) = body.statements[before as usize].action
                    && matches!(body.values[id.0 as usize].kind, Rvalue::Repeat { .. })
                {
                    assert_eq!(Some(effect.disposition), expected, "{name}");
                    seen += 1;
                }
            }
        }
        assert_eq!(seen, usize::from(expected.is_some()), "{name}");
    }
    Ok(())
}

#[test]
fn selected_constant_repeats_survive_lifetime_adjustments() -> Result<(), Box<dyn std::error::Error>>
{
    for length in [0, 1, 2] {
        let source = format!(
            "struct R<'a>(&'a u8);impl Drop for R<'_>{{fn drop(&mut self){{}}}}const ITEM:R<'static>=R(&1);fn f<'a>(_:&'a u8)->[R<'a>;{length}]{{[ITEM;{length}]}}"
        );
        rustc::qualify(&format!("repeat-adjustment-{length}"), &source, true)?;
        let (report, mut declarations) = semantic::analyze(&source)?;
        let program = body::build(&report, &declarations)?;
        let mut seen = 0;
        for body in &program.bodies {
            if length == 0 {
                assert!(
                    body.values
                        .iter()
                        .all(|value| !matches!(value.kind, Rvalue::Constant(_)))
                );
            }
            let facts = initialization::analyze(
                &declarations.types,
                &declarations.aggregates,
                &declarations.derives,
                &mut declarations.copies,
                body,
            )
            .map_err(|e| e.message)?;
            verify_points(body, &facts)?;
            for effect in facts.consumption.entries() {
                let Location::At { before, .. } = effect.location else {
                    continue;
                };
                if effect.moment == Moment::Statement
                    && let Action::Evaluate(id) = body.statements[before as usize].action
                    && let Rvalue::Repeat { value, mode } = body.values[id.0 as usize].kind
                {
                    assert_eq!(mode, rustgm::body::ir::RepeatMode::Constant);
                    assert!(matches!(
                        body.values[value.0 as usize].kind,
                        Rvalue::Constrain(_)
                    ));
                    assert_eq!(effect.disposition, Disposition::Instantiate);
                    seen += 1;
                }
            }
        }
        assert_eq!(seen, usize::from(length != 0));
    }
    Ok(())
}

#[test]
fn lifetime_constraints_transfer_the_owned_object_to_the_adjusted_value()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct R<'a>(&'a u8);impl Drop for R<'_>{fn drop(&mut self){}}fn f<'a>(r:R<'static>,_:&'a u8)->R<'a>{r}";
    rustc::qualify("owned-lifetime-adjustment", source, true)?;
    let (report, mut declarations) = semantic::analyze(source)?;
    let program = body::build(&report, &declarations)?;
    let mut seen = 0;
    for body in &program.bodies {
        let facts = initialization::analyze(
            &declarations.types,
            &declarations.aggregates,
            &declarations.derives,
            &mut declarations.copies,
            body,
        )
        .map_err(|e| e.message)?;
        verify_points(body, &facts)?;
        for effect in facts.consumption.entries() {
            let Location::At { before, .. } = effect.location else {
                continue;
            };
            if effect.moment == Moment::Statement
                && let Action::Evaluate(id) = body.statements[before as usize].action
                && matches!(body.values[id.0 as usize].kind, Rvalue::Constrain(_))
            {
                assert_eq!(effect.disposition, Disposition::Transfer);
                assert!(facts.ownership.paths.temporary_path(id).is_some());
                seen += 1;
            }
        }
    }
    assert!(seen > 0);
    Ok(())
}

#[test]
fn rvalue_paths_are_retained_only_for_initialization_analysis()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}fn f(){D;}";
    let (report, mut declarations) = semantic::analyze(source)?;
    let program = body::build(&report, &declarations)?;
    let mut seen = 0;
    for body in &program.bodies {
        let Some(owned) = &body.owned else { continue };
        let ordinary = ownership::analyze_body(
            &declarations.types,
            &declarations.aggregates,
            &declarations.derives,
            &mut declarations.copies,
            body,
        )
        .map_err(|e| e.message)?;
        let retained = initialization::analyze(
            &declarations.types,
            &declarations.aggregates,
            &declarations.derives,
            &mut declarations.copies,
            body,
        )
        .map_err(|e| e.message)?;
        for value in &owned.created {
            assert!(ordinary.paths.temporary_path(value.value).is_none());
            assert!(
                retained
                    .ownership
                    .paths
                    .temporary_path(value.value)
                    .is_some()
            );
            seen += 1;
        }
    }
    assert_eq!(seen, 1);
    Ok(())
}
