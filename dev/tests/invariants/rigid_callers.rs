//! Rigid membership is tied to the completed source owner, not its numeric IDs.
use super::*;
use crate::{
    resolve::{expansion::Report, package::Error},
    sema::values::execution::{Executor, Progress, Request, SourceComputation, Target},
    types::{
        ItemId, Primitive, TypeId,
        parameters::{Parameter, ParameterKind},
    },
};
use std::{cell::Cell, rc::Rc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Observe {
    inner: Box<dyn Executor>,
    seen: Rc<Cell<Option<SourceComputation>>>,
}
impl Executor for Observe {
    fn check(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        target: Target,
    ) -> std::result::Result<(), Error> {
        if let Target::SourceAssociated { source, .. } = target {
            self.seen.set(Some(source));
        }
        self.inner.check(report, facts, target)
    }
    fn advance(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
        request: &Request,
    ) -> std::result::Result<Progress, Error> {
        self.inner.advance(report, facts, request)
    }
}
fn fixture(width: usize) -> Result<(Declarations, SourceComputation)> {
    let names = (0..width)
        .map(|i| format!("T{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let fields = (0..width)
        .map(|i| format!("T{i}"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "struct S<{names}>({fields});impl<{names}>S<{names}>{{const C:u8={{let n=7;n}};}}fn run<{names}>(n:u8)->bool{{match n{{S::<{names}>::C=>true,_=>false}}}}fn main(){{}}"
    );
    let report = crate::test_source::resolve(&source, true)?;
    let context = crate::pipeline::execution::Context::default();
    let seen = Rc::new(Cell::new(None));
    let facts = crate::sema::declarations::analyze_with_executor(
        &report,
        Box::new(Observe {
            inner: context.executor(),
            seen: Rc::clone(&seen),
        }),
    )?;
    let facts = crate::pipeline::analyze(&report, facts)?.facts;
    Ok((facts, seen.get().ok_or("missing rigid source request")?))
}
#[test]
fn rigid_context_rejects_foreign_catalog_owner_parameters_and_missing_publication() -> Result {
    let (mut facts, witness) = fixture(1)?;
    let (entry, key) = witness.entry(&facts)?;
    let context = facts
        .source_computations
        .as_deref()
        .ok_or("catalog")?
        .0
        .parameters(entry.caller)?
        .ok_or("rigid context")?;
    let (other, foreign) = fixture(1)?;
    let (foreign_entry, foreign_key) = foreign.entry(&other)?;
    assert!(
        context
            .validate(&other, foreign_entry.caller, foreign_key)
            .is_err()
    );
    let mut wrong = key;
    wrong.source.node.0 += 1;
    assert!(context.validate(&facts, entry.caller, wrong).is_err());
    let foreign = facts.types.intern_parameter(Parameter {
        owner: ItemId(entry.selected.definition.0),
        index: 777,
        kind: ParameterKind::Type { sized: true },
    })?;
    let ty = facts.types.intern(Kind::Parameter(foreign))?;
    let selected = Instance {
        definition: entry.selected.definition,
        arguments: facts.types.intern_arguments(&[Argument::Type(ty)])?,
    };
    assert!(
        SourceComputation::shape(&mut facts, selected, entry.original, Some(&context)).is_err()
    );
    assert!(entry.selected.result_type(&mut facts).is_err());
    let publication =
        catalog::Publication::new([0..0, 0..facts.region_equalities.len().saturating_add(1)])?;
    let signature = crate::sema::callee_premises::OriginalInputs::identity(
        &facts,
        key.source,
        key.declaration,
    )?
    .0;
    assert!(matches!(
        prepare(
            &mut facts,
            key,
            publication,
            Inputs {
                source: key.source,
                universals: key.universals,
                signature,
                parameters: &context.values,
                assumptions: &[],
                type_assumptions: &[],
                require_result: false,
            },
            entry.selected,
        ),
        Err("rigid source parameters lack their exact completed caller publication")
    ));
    let result = TypeId::primitive(Primitive::U8);
    assert!(witness.resolve(&mut facts, result).is_ok());
    let catalog = facts.source_computations.take();
    assert!(witness.resolve(&mut facts, result).is_err());
    facts.source_computations = catalog;
    assert_eq!(witness.rigid_parameters(&mut facts)?.len(), 1);
    Ok(())
}
#[test]
fn rigid_context_shares_one_bounded_owner_index_and_counts_each_selected_argument() -> Result {
    let mut previous = None;
    for width in [1, 8, 32, 128] {
        let (mut facts, witness) = fixture(width)?;
        let (entry, key) = witness.entry(&facts)?;
        let catalog = &facts.source_computations.as_deref().ok_or("catalog")?.0;
        let original = catalog.parameters(entry.caller)?.ok_or("parameters")?;
        let publication = catalog.publication(entry.caller)?;
        let signature = crate::sema::callee_premises::OriginalInputs::identity(
            &facts,
            key.source,
            key.declaration,
        )?
        .0;
        let values = original.values.clone();
        let before = facts.body_work.canonical_steps;
        let warm = prepare(
            &mut facts,
            key,
            publication,
            Inputs {
                source: key.source,
                universals: key.universals,
                signature,
                parameters: &values,
                assumptions: &[],
                type_assumptions: &[],
                require_result: false,
            },
            entry.selected,
        )?
        .ok_or("warm parameters")?;
        assert!(Arc::ptr_eq(&original, &warm));
        assert_eq!(facts.body_work.canonical_steps - before, width + 2);
        let before = facts.body_work.canonical_steps;
        assert_eq!(witness.rigid_parameters(&mut facts)?.len(), width);
        assert_eq!(facts.body_work.canonical_steps - before, width + 1);
        let bytes = original.retained_bytes();
        eprintln!(
            "rigid arguments={width} cold-bytes={bytes} warm-work={} selected-work={}",
            width + 2,
            width + 1
        );
        if let Some((old_width, old_bytes)) = previous {
            assert!(bytes <= old_bytes * (width / old_width));
        }
        previous = Some((width, bytes));
    }
    Ok(())
}

#[test]
fn rigid_parameter_membership_preserves_the_actual_caller_type_outlives_debt() -> Result {
    for (name, accepted) in [("type-outlives", true), ("unpaid-outlives", false)] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/rigid-pattern-context")
            .join(format!("{name}.rs"));
        let report = crate::test_source::resolve(&std::fs::read_to_string(path)?, true)?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let function = *facts
            .functions
            .iter()
            .find(|function| {
                facts
                    .generics
                    .parameters_in_scope(function.definition)
                    .any(|p| matches!(p.argument, Argument::Type(_)))
            })
            .ok_or("missing generic caller")?;
        let contract = crate::body::regions::Collector::default()
            .function(&facts, &function)?
            .clone();
        assert_eq!(contract.parameters.len(), 1);
        let (_, shorter) = contract
            .obligations
            .iter()
            .find_map(|obligation| match obligation.kind {
                crate::body::regions::Kind::TypeOutlives { ty, shorter }
                    if matches!(facts.types.kind(ty), Some(Kind::Parameter(id)) if contract.parameters.contains(&id)) =>
                {
                    Some((ty, shorter))
                }
                _ => None,
            })
            .ok_or("missing actual caller parameter outlives debt")?;
        let mut equations = crate::types::substitute::RegionEquations::default();
        equations.extend(contract.obligations.iter().filter_map(|obligation| {
            if let crate::body::regions::Kind::Equality(pair) = obligation.kind {
                Some(pair)
            } else {
                None
            }
        }))?;
        assert_eq!(
            equations.canonical_anchor(shorter, &mut 0)?,
            crate::types::RegionId::bound(0, 0)?,
        );
        let template =
            crate::body::templates::Prepared::default().function(&report, &facts, &function)?;
        contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
        let mut queries = crate::types::outlives::Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let proof = crate::borrow::regions::check_one(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            &contract,
            &template.body,
        );
        assert_eq!(proof.is_ok(), accepted, "{name}");
        if let Err(error) = proof {
            assert!(error.message.contains("outlive"));
        }
    }
    Ok(())
}
