//! Distinct conditions share structural folds and resolved parents without sharing local hypotheses.
use super::{
    BYTE, Declarations, State, TestResult, owner, quantified_tests, quantify, root, tests,
};
use crate::{
    sema::generics::Predicate as Located,
    types::{
        Kind, RegionId,
        arguments::Argument,
        parameters::clauses::{Clause, Quantified},
    },
};
use std::fmt::Write as _;

struct CaptureGraph {
    facts: Declarations,
    proof: crate::types::lifetime_proof::Obligations,
    expected: crate::types::TypeId,
    work: usize,
    growth: usize,
    failure: Option<String>,
}

fn shared_capture_type(
    types: &mut crate::types::Store,
    width: usize,
    depth: u16,
) -> TestResult<crate::types::TypeId> {
    let mut reference = |region| {
        types.intern(Kind::Reference {
            mutable: false,
            region,
            referent: BYTE,
        })
    };
    let outer = reference(RegionId::bound(depth, 1)?)?;
    let local = reference(RegionId::bound(0, 0)?)?;
    let captured = reference(RegionId::bound(depth + 1, 1)?)?;
    let parameters = types.intern_list(&[local, captured])?;
    let function = types.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: crate::types::TypeId::UNIT,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let fields = types.intern_list(&[outer, function])?;
    let mut tail = types.intern(Kind::Tuple(fields))?;
    for _ in 0..width {
        let fields = types.intern_list(&[tail, tail])?;
        tail = types.intern(Kind::Tuple(fields))?;
    }
    Ok(tail)
}

fn capture_graph(width: usize, local: bool, limited: bool) -> TestResult<CaptureGraph> {
    use super::super::{Node, Search, Source};
    use crate::types::{infer::Context, lifetime_proof::ScopeId, parameters::predicates::Bound};
    let (_, mut facts) = tests::declarations("trait Empty{}")?;
    let ty = shared_capture_type(&mut facts.types, width, 0)?;
    let assumptions = crate::sema::traits::assumptions::Assumptions::default();
    let mut search = Search::new(
        tests::caller(&assumptions, None),
        crate::sema::traits::implementations::Availability::Declarations,
    );
    let mut roots = Vec::new();
    for _ in 0..width {
        let first = search.query.allocations.reserve_scoped();
        let scope = search
            .query
            .scopes
            .get_or_insert_with(Default::default)
            .introduce(ScopeId::ROOT, 1, first)?;
        search.query.allocations.record(first + 1);
        let end = search.query.allocations.end();
        let shorter = if local {
            RegionId(first)
        } else {
            RegionId::bound(0, 1)?
        };
        roots.push(search.query.push(
            Node {
                debt_free: false,
                cycle_dependent: false,
                provisional_size: false,
                goal: crate::types::parameters::predicates::Predicate {
                    ty: BYTE,
                    bound: Bound::Sized,
                },
                source: Some(Source::Builtin),
                children: Vec::new(),
                first: end,
                end,
                equalities: Vec::new(),
                outlives: Vec::new(),
                type_outlives: vec![(ty, shorter)],
                sized: Vec::new(),
            },
            scope,
            end..end,
        ));
    }
    if limited {
        search.query.steps = 999_999;
    }
    let end = search.query.allocations.end();
    let mut proof = std::mem::replace(
        &mut search.query.allocations,
        super::super::allocations::Allocations::new(1),
    )
    .finish(end);
    let mut context = Context::default();
    let mut sized = Vec::new();
    let before = facts.types.len();
    let result = search.pay_extraction(&mut context, |search| {
        search.collect_graph(
            &mut facts,
            (roots, super::super::GraphTraversal::Ordinary),
            None,
            &mut proof,
            &mut sized,
            None,
        )
    });
    let growth = facts.types.len() - before;
    let expected = shared_capture_type(&mut facts.types, width, 1)?;
    Ok(CaptureGraph {
        facts,
        proof,
        expected,
        work: context.steps,
        growth,
        failure: result.err(),
    })
}

#[test]
fn scoped_capture_reclosure_shares_type_prefixes_and_pays_the_caller() -> TestResult {
    let mut previous = None;
    for width in [8, 32, 128] {
        let graph = capture_graph(width, false, false)?;
        assert_eq!(graph.failure, None);
        assert_eq!(graph.proof.scope_count(), width);
        assert_eq!(graph.proof.type_roots().count(), width);
        assert!(graph.proof.type_roots().all(|(_, ty)| ty == graph.expected));
        assert!(quantified_tests::valid(&graph.facts, &graph.proof, 2)?);
        assert!(
            graph.work > 0 && graph.work < 64 * width + 256,
            "{width}: {}",
            graph.work
        );
        assert!(graph.growth < 4 * width + 32, "{width}: {}", graph.growth);
        if let Some(before) = previous {
            assert!(graph.work <= 4 * before + 64);
        }
        previous = Some(graph.work);
    }
    Ok(())
}

#[test]
fn scoped_capture_reclosure_keeps_type_debts_and_nested_binders() -> TestResult {
    let graph = capture_graph(8, true, false)?;
    assert_eq!(graph.failure, None);
    assert_eq!(graph.proof.type_roots().count(), 8);
    assert!(graph.proof.type_roots().all(|(_, ty)| ty == graph.expected));
    assert!(!quantified_tests::valid(&graph.facts, &graph.proof, 2)?);
    Ok(())
}

#[test]
fn scoped_capture_reclosure_failure_still_pays_the_caller() -> TestResult {
    let graph = capture_graph(8, false, true)?;
    assert_eq!(
        graph.failure.as_deref(),
        Some("trait selection work budget exceeded")
    );
    assert!(graph.work > 0);
    Ok(())
}

fn install(
    facts: &mut Declarations,
    owner: crate::resolve::model::DefId,
    clauses: Vec<Clause>,
) -> TestResult {
    let source = facts
        .generics
        .implementations()
        .find(|header| header.owner == owner)
        .ok_or("missing scaling implementation")?
        .source
        .ok_or("source scaling fixture requires an actual source implementation")?;
    facts.generics.predicates.insert(
        owner,
        clauses
            .into_iter()
            .map(|value| Located { source, value })
            .collect(),
    );
    Ok(())
}

#[test]
fn implementation_conditions_share_growing_canonical_tails() -> TestResult {
    let mut observations = Vec::new();
    for width in [16, 32, 64, 128, 256, 512] {
        let (report, mut facts) =
            tests::declarations("trait A<T>{}impl<T>A<T>for u8{}trait B{}impl B for u8{}")?;
        let item = tests::definition(&report, "A")?;
        let owner = owner(&facts, tests::definition(&report, "B")?)?;
        let mut tail = BYTE;
        let mut clauses = Vec::new();
        for _ in 0..width {
            tail = facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(0, 0)?,
                referent: tail,
            })?;
            let arguments = facts.types.intern_arguments(&[Argument::Type(tail)])?;
            let predicate = tests::predicate(&mut facts, item, BYTE, arguments)?;
            clauses.push(Clause::ForAll(facts.types.intern_quantified(
                Quantified {
                    bound_regions: 1,
                    predicate,
                },
            )?));
        }
        install(&mut facts, owner, clauses)?;
        let root = root(&mut facts, &report, "B")?;
        let before = facts.types.len();
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?);
        let steps = state.selection_steps();
        let growth = facts.types.len() - before;
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert!(quantified_tests::valid(&facts, &proof, 0)?);
        println!("impl-tail width={width} steps={steps} types={growth}");
        observations.push((width, steps, growth));
        assert!(steps < 512 * width + 1024, "{width}: {steps}");
        assert!(growth < 16 * width + 32, "{width}: {growth}");
    }
    for pair in observations.windows(2) {
        assert!(pair[1].1 <= 2 * pair[0].1 + 256, "{pair:?}");
        assert!(pair[1].2 <= 2 * pair[0].2 + 32, "{pair:?}");
    }
    Ok(())
}

#[test]
fn wide_parent_arguments_resolve_once_for_all_quantified_conditions() -> TestResult {
    let mut observations = Vec::new();
    for width in [8, 16, 32, 64, 128] {
        let names = (0..width)
            .map(|i| format!("T{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<T>{{}}impl<T>A<T>for u8{{}}trait B<{names}>{{}}impl<{names}>B<{names}>for u8{{}}"
        ))?;
        let item = tests::definition(&report, "A")?;
        let b = tests::definition(&report, "B")?;
        let owner = owner(&facts, b)?;
        let parameters: Vec<_> = facts
            .generics
            .parameters_in_scope(owner)
            .map(|p| p.argument)
            .collect();
        let mut clauses = Vec::new();
        for argument in parameters {
            let arguments = facts.types.intern_arguments(&[argument])?;
            let predicate = tests::predicate(&mut facts, item, BYTE, arguments)?;
            clauses.push(Clause::ForAll(facts.types.intern_quantified(
                Quantified {
                    bound_regions: 1,
                    predicate,
                },
            )?));
        }
        install(&mut facts, owner, clauses)?;
        let arguments = facts
            .types
            .intern_arguments(&vec![Argument::Type(BYTE); width])?;
        let root = tests::predicate(&mut facts, b, BYTE, arguments)?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?);
        let steps = state.selection_steps();
        let proof = state.finish();
        assert!(quantified_tests::valid(&facts, &proof, 0)?);
        assert_eq!(proof.scope_count(), 1);
        println!("impl-parent width={width} steps={steps}");
        observations.push((width, steps));
    }
    for pair in observations.windows(2) {
        assert!(pair[1].1 <= 2 * pair[0].1 + 256, "{pair:?}");
    }
    Ok(())
}

#[test]
fn nested_impl_universes_use_bounded_iterative_search_and_proof_storage() -> TestResult {
    let mut observations = Vec::new();
    for depth in [8, 16, 32, 64, 128] {
        let mut source = String::from("trait A0<'a>{}impl<'a>A0<'a>for u8{}");
        for i in 1..=depth {
            write!(
                source,
                "trait A{i}<'a>{{}}impl<'a,T:A{}<'static>>A{i}<'a>for T{{}}",
                i - 1
            )?;
        }
        let (report, mut facts) = tests::declarations(&source)?;
        for i in 1..=depth {
            quantify(
                &mut facts,
                &report,
                &format!("A{i}"),
                &format!("A{}", i - 1),
                1,
                &[RegionId::bound(0, 0)?],
            )?;
        }
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?;
        let root = tests::predicate(
            &mut facts,
            tests::definition(&report, &format!("A{depth}"))?,
            BYTE,
            arguments,
        )?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove(&mut facts, root, true)?);
        let steps = state.selection_steps();
        let proof = state.finish();
        assert_eq!(proof.scope_count(), depth);
        assert!(quantified_tests::valid(&facts, &proof, 0)?);
        assert!(proof.regions.end < u32::try_from(4 * depth + 8)?);
        println!(
            "impl-depth depth={depth} steps={steps} regions={}",
            proof.regions.end
        );
        observations.push((depth, steps));
    }
    for pair in observations.windows(2) {
        assert!(pair[1].1 <= 2 * pair[0].1 + 256, "{pair:?}");
    }
    Ok(())
}
