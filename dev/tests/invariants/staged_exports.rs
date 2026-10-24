//! Failed original collection never reaches final staged export publication.
use super::super::*;
use crate::{
    body::ir::{BodyKey, Rvalue},
    support::hash::HashSet,
};
use std::{cell::Cell, rc::Rc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
type Collector = fn(
    &Declarations,
    &borrow::incremental::Proved,
    &body::regions::Contract,
) -> std::result::Result<(), Error>;
const _: [(); 2 * size_of::<usize>()] =
    [(); size_of::<super::Context<'static, 'static, Collector>>()];

#[derive(Default)]
struct Counts {
    visits: Cell<usize>,
    drops: Cell<usize>,
    published: Cell<bool>,
}
struct Stage {
    rows: HashSet<BodyKey>,
    counts: Rc<Counts>,
}
impl Drop for Stage {
    fn drop(&mut self) {
        self.counts.drops.set(self.counts.drops.get() + 1);
    }
}
#[derive(Clone, Copy)]
enum Damage {
    None,
    LateOriginal,
    Collector,
}

#[derive(Debug, PartialEq, Eq)]
struct Lowered {
    text: Vec<u8>,
    functions: usize,
    cfg: usize,
    canonical: usize,
}

fn selected(
    report: &Report,
    facts: &Declarations,
) -> std::result::Result<(Option<DefId>, super::super::super::roots::Exports), Error> {
    let entry = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == "main"
        })
        .map(|function| function.definition);
    let exports = super::super::super::roots::exports(facts, &[])
        .map_err(|message| report.body_optional_error(None, message))?;
    Ok((entry, exports))
}

fn lower(
    report: &Report,
    facts: &mut Declarations,
    assembled: borrow::incremental::Assembled,
    roots: super::super::super::roots::Selected<'_>,
) -> std::result::Result<Lowered, Error> {
    let program = super::super::super::build_assembled(facts, assembled, roots)
        .map_err(|error| diagnostic(report, error.node, error.message.into()))?;
    let mut text = Vec::new();
    crate::lowir::writer::write(&program, &mut text)
        .map_err(|error| report.body_optional_error(None, error.to_string()))?;
    Ok(Lowered {
        text,
        functions: program
            .symbols
            .iter()
            .filter(|symbol| symbol.body.is_some())
            .count(),
        cfg: facts.body_work.cfg_functions.get(),
        canonical: facts.body_work.canonical_steps,
    })
}

fn run(source: &str, damage: Damage, counts: &Rc<Counts>) -> Result<(usize, Lowered)> {
    let report = crate::test_source::resolve(source, true)?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    if matches!(damage, Damage::LateOriginal) {
        let program = body::build_concrete(&report, &analysis.facts)?;
        let mut late = program
            .bodies
            .into_iter()
            .find(|body| {
                matches!(body.owner, body::ir::Owner::Function { definition, .. }
                    if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == "late")
            })
            .ok_or("late actual body")?;
        let value = late
            .values
            .iter_mut()
            .find(|value| matches!(value.kind, Rvalue::Scalar(_)))
            .ok_or("late original scalar")?;
        value.kind = Rvalue::Read(body::ir::Place::Local(body::ir::LocalId(0)));
        analysis.prepared.concrete.insert(late.key(), late);
    }
    let outcome = inspect_staged(
        &report,
        &mut analysis.facts,
        analysis.prepared,
        |facts, _| {
            let (entry, exports) = selected(&report, facts)?;
            Ok((
                entry,
                exports,
                Stage {
                    rows: HashSet::default(),
                    counts: Rc::clone(counts),
                },
            ))
        },
        |stage, _, proved, input| {
            let body = proved.body();
            assert!(body.instance.is_none());
            assert_eq!(body.key(), input.key);
            assert_eq!(body.source, input.source);
            stage.counts.visits.set(stage.counts.visits.get() + 1);
            if matches!(damage, Damage::Collector) && stage.counts.visits.get() == 2 {
                return Err(report.body_error(body.source, "bounded original collector failed"));
            }
            assert!(
                stage.rows.insert(body.key()),
                "ordinary owners are collected once"
            );
            Ok(())
        },
        |facts, assembled, roots, stage| {
            let result = lower(&report, facts, assembled, roots)?;
            let count = stage.rows.len();
            stage.counts.published.set(true);
            Ok::<_, Error>((count, result))
        },
    )?;
    Ok(outcome?)
}

#[test]
fn ordinary_collector_failure_discards_partial_rows_before_finalization() {
    let counts = Rc::new(Counts::default());
    let result = run(
        "fn a()->u8{7}fn b()->u8{9}fn main(){let _=a();let _=b();}",
        Damage::Collector,
        &counts,
    );
    assert!(result.is_err_and(|error| {
        error
            .to_string()
            .contains("bounded original collector failed")
    }));
    assert_eq!(counts.visits.get(), 2);
    assert_eq!(counts.drops.get(), 1);
    assert!(!counts.published.get());
}

#[test]
fn real_late_group_and_child_failures_publish_no_staged_prefix() -> Result {
    for fixture in ["paid-lifetime", "faulting-child"] {
        let mut source = crate::test_source::fixture(&format!(
            "pa25/tests/spec/inline-query-context/{fixture}.rs"
        ))?;
        let damage = if fixture == "paid-lifetime" {
            source.push_str("fn late()->u8{let x:u8=9;x}");
            Damage::LateOriginal
        } else {
            Damage::None
        };
        let counts = Rc::new(Counts::default());
        let result = run(&source, damage, &counts);
        let message = result
            .err()
            .ok_or("actual failure must reject")?
            .to_string();
        if fixture == "paid-lifetime" {
            assert!(message.contains("uninitialized"), "{message}");
        } else {
            assert!(message.contains("divides by zero"), "{message}");
        }
        assert_eq!(
            counts.visits.get(),
            0,
            "no callback before complete group success"
        );
        assert_eq!(counts.drops.get(), 1);
        assert!(!counts.published.get());
    }
    Ok(())
}

#[test]
fn completed_group_collects_each_actual_original_after_parent_retirement() -> Result {
    let source =
        crate::test_source::fixture("pa25/tests/spec/inline-query-context/paid-lifetime.rs")?;
    let counts = Rc::new(Counts::default());
    let (rows, lowered) = run(&source, Damage::None, &counts)?;
    assert!(rows > 1);
    assert!(lowered.functions > 0);
    assert_eq!(counts.visits.get(), rows);
    assert_eq!(counts.drops.get(), 1);
    assert!(counts.published.get());
    Ok(())
}

#[test]
fn legacy_unit_state_keeps_actual_lowering_and_cfg_work() -> Result {
    let source = "fn id<T>(x:T)->T{x}fn main(){let _=id(7u8);}";
    let counts = Rc::new(Counts::default());
    let (rows, lowered) = run(source, Damage::None, &counts)?;
    assert_eq!(
        rows, 1,
        "the selected instance is not another defining original"
    );
    let report = crate::test_source::resolve(source, true)?;
    let mut analysis = crate::pipeline::analyze_source(&report)?;
    let legacy = inspect_selected(
        &report,
        &mut analysis.facts,
        analysis.prepared,
        |facts, _| selected(&report, facts),
        |facts, assembled, roots| lower(&report, facts, assembled, roots),
    )??;
    assert_eq!(legacy, lowered);
    Ok(())
}
