//! Completed source callers supply all positive rows; altered rows are opponents.
use super::*;
use crate::{
    body::ir::Rvalue,
    const_eval::{code::Catalog, use_inputs::Inputs},
    resolve::{expansion::Report, model::DefId},
    sema::values::execution::Target,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn inspect(
    name: &str,
    use_rows: impl FnOnce(&Report, &mut Declarations, &Catalog, &Inputs, usize) -> TestResult,
) -> TestResult {
    let source =
        crate::test_source::fixture(&format!("pa31/tests/spec/const-eval-select-ctfe/{name}.rs"))?;
    let report = crate::test_source::source_provider(&source, "opened_callback_rows")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let context = prepared.constants.as_ref().ok_or("source provider")?;
    let definition = definition(&report, "N")?;
    context.inspect_const_call(Target::Initializer(definition), |catalog, rows, root| {
        use_rows(&report, &mut facts, catalog, rows, root)
    })??;
    Ok(())
}
fn definition(report: &Report, name: &str) -> TestResult<DefId> {
    let index = report
        .resolution
        .definitions
        .iter()
        .position(|d| report.package.names.get(d.name) == name)
        .ok_or("actual source definition")?;
    Ok(DefId(u32::try_from(index)?))
}
fn selected(
    facts: &Declarations,
    catalog: &Catalog,
    inputs: &Inputs,
    root: usize,
) -> TestResult<(ValueId, crate::sema::compiler_items::Select, Opening)> {
    let code = catalog.code(root).ok_or("actual source code")?;
    let row = inputs.rows.get(&root).ok_or("retained source row")?;
    let (&call, &opening) = row
        .selectors
        .as_deref()
        .and_then(|rows| rows.calls.iter().next())
        .ok_or("actual reference call descriptor")?;
    let Rvalue::Call { callee, .. } = code.values[call.0 as usize].kind else {
        return Err("actual Call".into());
    };
    let raw = Instance::new(&facts.types, code.values[callee.0 as usize].ty)?;
    let selection = crate::sema::compiler_items::source_select(facts, raw)?
        .ok_or("actual selector instance")?;
    Ok((call, selection, opening))
}

#[test]
fn reference_descriptors_require_the_actual_catalog_code_and_retained_row() -> TestResult {
    inspect("reference-binder", |_, facts, catalog, inputs, root| {
        let code = catalog.code(root).ok_or("Code")?;
        let (call, selection, opening) = selected(facts, catalog, inputs, root)?;
        let callback = Instance::new(&facts.types, selection.types[1])?;
        opening.validate(facts, selection, callback)?;
        inputs.opening(catalog, root, code, call)?;
        assert!(
            inputs
                .opening(catalog, root, code, ValueId(u32::MAX))
                .is_err()
        );
        assert!(inputs.opening(catalog, usize::MAX, code, call).is_err());
        assert!(
            Inputs::default()
                .opening(catalog, root, code, call)
                .is_err()
        );
        inspect(
            "reference-binder",
            |_, _, foreign, foreign_inputs, other| {
                let foreign_code = foreign.code(other).ok_or("foreign Code")?;
                assert_eq!(other, root);
                assert_eq!(
                    foreign_code.owner.key(foreign_code.source),
                    code.owner.key(code.source)
                );
                assert_eq!(foreign_code.owner.signatures(), code.owner.signatures());
                assert_eq!(foreign_code.values.len(), code.values.len());
                assert!(inputs.opening(foreign, other, foreign_code, call).is_err());
                assert!(foreign_inputs.opening(catalog, root, code, call).is_err());
                assert!(inputs.opening(catalog, root, foreign_code, call).is_err());
                Ok(())
            },
        )
    })
}

#[test]
fn reference_descriptors_preserve_full_selector_callback_and_correlated_signature() -> TestResult {
    inspect(
        "reference-two-arguments",
        |_, facts, catalog, inputs, root| {
            let (_, selection, opening) = selected(facts, catalog, inputs, root)?;
            let callback = Instance::new(&facts.types, selection.types[1])?;
            opening.validate(facts, selection, callback)?;
            let other = inputs
                .rows
                .iter()
                .filter(|(index, _)| **index != root)
                .find_map(|(_, row)| row.selectors.as_deref()?.calls.values().next().copied())
                .ok_or("second genuine reference relation")?;
            assert!(other.validate(facts, selection, callback).is_err());
            for opponent in [
                Opening {
                    selector: callback,
                    ..opening
                },
                Opening {
                    constant: selection.instance,
                    ..opening
                },
                Opening {
                    signature: TypeId::UNIT,
                    ..opening
                },
            ] {
                assert!(opponent.validate(facts, selection, callback).is_err());
            }
            assert!(
                opening
                    .validate(facts, selection, selection.instance)
                    .is_err()
            );
            let Some(Kind::Function { parameters, .. }) = facts.types.kind(opening.signature)
            else {
                return Err("whole opened function type".into());
            };
            let signature = facts.types.intern(Kind::Function {
                bound_regions: 0,
                parameters,
                result: TypeId::UNIT,
                unsafe_: false,
                convention: Convention::RUST,
            })?;
            assert!(
                Opening {
                    signature,
                    ..opening
                }
                .validate(facts, selection, callback)
                .is_err()
            );
            println!(
                "Row={} Opening={} descriptor-payload={}",
                size_of::<super::super::Row>(),
                size_of::<Opening>(),
                inputs.rows[&root]
                    .selectors
                    .as_deref()
                    .ok_or("descriptors")?
                    .bytes()
            );
            Ok(())
        },
    )
}

fn retained_source(row: &super::super::Row) -> super::super::Source {
    // Copy only data from this real completed source relation. No proof or
    // descriptor is synthesized for the publication/storage owner controls.
    super::super::Source {
        contract: std::sync::Arc::clone(&row.contract),
        selectors: row.selectors.as_deref().map(|rows| {
            Box::new(Rows {
                calls: rows.calls.clone(),
            })
        }),
    }
}

#[test]
fn source_row_preflight_rejects_wrong_code_and_charges_retained_capacity_on_retry() -> TestResult {
    inspect("reference-binder", |_, facts, catalog, inputs, root| {
        let row = inputs.rows.get(&root).ok_or("actual Row")?;
        let executable = std::rc::Rc::clone(&catalog[root]);
        let mut owner = Inputs::default();
        let empty = owner.retained_bytes();
        let plan = owner.plan(
            facts,
            root,
            std::rc::Rc::clone(&executable),
            retained_source(row),
            None,
        )?;
        assert!(owner.rows.is_empty(), "preparation must publish no row");
        assert!(
            owner.retained_bytes() > empty,
            "allocated index capacity stays charged"
        );
        assert!(owner.check_plan(&plan, usize::MAX, &executable).is_err());
        let other = catalog
            .functions
            .values()
            .chain(catalog.definitions.values())
            .map(|&index| &catalog[index])
            .find(|code| !std::rc::Rc::ptr_eq(code, &executable))
            .ok_or("another actual Code")?;
        assert!(owner.check_plan(&plan, root, other).is_err());
        drop(plan);
        let warm = owner.retained_bytes();
        let plan = owner.plan(
            facts,
            root,
            std::rc::Rc::clone(&executable),
            retained_source(row),
            None,
        )?;
        assert_eq!(
            owner.retained_bytes(),
            warm,
            "warm index capacity is not recharged"
        );
        owner.check_plan(&plan, root, &executable)?;
        owner.commit(plan);
        let (call, _, _) = selected(facts, catalog, &owner, root)?;
        owner.opening(catalog, root, &executable, call)?;
        assert!(
            owner
                .plan(facts, root, executable, retained_source(row), None)
                .is_err()
        );
        Ok(())
    })
}

#[test]
fn source_code_and_reference_rows_publish_together_after_all_storage_checks() -> TestResult {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/const-eval-select-ctfe/reference-local-argument.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "paired_reference_publication")?;
    let choose = definition(&report, "choose")?;
    let (analysis, bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[choose], || {
            crate::pipeline::analyze_source(&report)
        })?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    let body = bodies.first().ok_or("actual completed caller")?;
    for _ in 0..2 {
        let before = facts.body_work.canonical_steps;
        assert!(Inputs::prepare_uses(&mut facts, body, 0).is_err());
        assert!(
            facts.body_work.canonical_steps > before,
            "failed descriptor preparation retains attempted work on retry"
        );
    }
    let context = prepared.constants.as_ref().ok_or("provider")?;
    context.inspect_const_call(
        Target::Initializer(definition(&report, "N")?),
        |catalog, inputs, _| -> TestResult {
            let root = *catalog.definitions.get(&choose).ok_or("caller Code")?;
            let row = inputs.rows.get(&root).ok_or("caller Row")?;
            let mut candidate = Catalog::default();
            let mut retained = Some(Box::new(Inputs {
                bytes: 64 * 1024 * 1024,
                ..Inputs::default()
            }));
            for _ in 0..2 {
                let code = crate::const_eval::code::Prepared::plain(body, 64 * 1024 * 1024)?;
                assert!(
                    candidate
                        .stage_ordinary(code)?
                        .publish_source(
                            &mut facts,
                            &mut retained,
                            Some(retained_source(row)),
                            body.instance
                        )
                        .is_err()
                );
                assert!(candidate.code(0).is_none());
                assert!(candidate.definitions.is_empty());
                assert!(candidate.functions.is_empty());
                assert_eq!(candidate.bytes, 0);
                assert!(retained.as_deref().ok_or("input owner")?.rows.is_empty());
            }
            retained.as_deref_mut().ok_or("input owner")?.bytes = 0;
            let code = crate::const_eval::code::Prepared::plain(body, 64 * 1024 * 1024)?;
            let index = candidate.stage_ordinary(code)?.publish_source(
                &mut facts,
                &mut retained,
                Some(retained_source(row)),
                body.instance,
            )?;
            let retained = retained.as_deref().ok_or("published input owner")?;
            let (call, _, _) = selected(&facts, &candidate, retained, index)?;
            retained.opening(
                &candidate,
                index,
                candidate.code(index).ok_or("published Code")?,
                call,
            )?;
            assert!(candidate.code(0).is_some());
            assert!(candidate.code(1).is_none());
            assert_eq!(retained.rows.len(), 1);
            Ok(())
        },
    )??;
    Ok(())
}

#[test]
fn warmed_reference_callback_keeps_caller_allocations_and_requires_its_input_owner() -> TestResult {
    inspect(
        "reference-local-argument",
        |report, facts, catalog, inputs, root| {
            for _ in 0..2 {
                let evaluator = crate::sema::values::Evaluator::new(report, facts);
                let mut machine = crate::const_eval::Machine::source_catalog(
                    evaluator,
                    catalog,
                    Some(inputs),
                    crate::const_eval::Limits::default(),
                );
                machine.begin(root, &[]).map_err(|error| error.message)?;
                let crate::sema::values::execution::Progress::Complete(value) =
                    machine.advance().map_err(|error| error.message)?
                else {
                    return Err("the already prepared F unexpectedly demanded a body".into());
                };
                assert_eq!(value.as_scalar()?, crate::sema::model::Scalar::Integer(7));
                assert_eq!(
                    machine.work.calls, 3,
                    "initializer, caller and actual F only"
                );
                assert_eq!(machine.work.peak_frames, 3);
            }
            let empty = Inputs::default();
            for provided in [None, Some(&empty)] {
                let evaluator = crate::sema::values::Evaluator::new(report, facts);
                let mut machine = crate::const_eval::Machine::source_catalog(
                    evaluator,
                    catalog,
                    provided,
                    crate::const_eval::Limits::default(),
                );
                machine.begin(root, &[]).map_err(|error| error.message)?;
                let error = machine
                    .advance()
                    .err()
                    .ok_or("a warmed callback bypassed caller input authority")?;
                assert_eq!(
                    error.message,
                    if provided.is_some() {
                        "source caller has no actual retained row"
                    } else {
                        "source const_eval_select callback binders require proved caller inputs"
                    }
                );
                assert_eq!(
                    machine.work.calls, 2,
                    "no F or selector placeholder frame after failure"
                );
            }
            Ok(())
        },
    )
}
