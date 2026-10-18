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
        .source;
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
