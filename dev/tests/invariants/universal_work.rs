//! Distinct universal obligations must share structural opening work, not permission.
use super::{Declarations, quantified_tests::valid, tests};
use crate::{
    resolve::model::DefId,
    sema::nominals::traits::State,
    types::{
        Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::Predicate,
        },
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn clause(facts: &mut Declarations, count: u32, predicate: Predicate) -> TestResult<Clause> {
    Ok(Clause::ForAll(facts.types.intern_quantified(
        Quantified {
            bound_regions: count,
            predicate,
        },
    )?))
}

fn lifetime_goal(
    facts: &mut Declarations,
    item: DefId,
    count: u32,
    regions: &[RegionId],
) -> TestResult<Clause> {
    let arguments: Vec<_> = regions.iter().copied().map(Argument::Lifetime).collect();
    let arguments = facts.types.intern_arguments(&arguments)?;
    let predicate = tests::predicate(facts, item, TypeId::primitive(Primitive::U8), arguments)?;
    clause(facts, count, predicate)
}

#[test]
fn distinct_universal_goals_share_growing_lifetime_tails() -> Result<(), Box<dyn std::error::Error>>
{
    let mut observations = Vec::new();
    for width in [16, 32, 64, 128, 256, 512] {
        let (report, mut facts) = tests::declarations("trait A<T>{}impl<T>A<T>for u8{}")?;
        let item = tests::definition(&report, "A")?;
        let byte = TypeId::primitive(Primitive::U8);
        let mut tail = byte;
        let mut clauses = Vec::new();
        for _ in 0..width {
            tail = facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(0, 0)?,
                referent: tail,
            })?;
            let arguments = facts.types.intern_arguments(&[Argument::Type(tail)])?;
            let predicate = tests::predicate(&mut facts, item, byte, arguments)?;
            clauses.push(Clause::ForAll(facts.types.intern_quantified(
                Quantified {
                    bound_regions: 1,
                    predicate,
                },
            )?));
        }
        let before = facts.types.len();
        let mut state = State::instance(&mut facts, 0)?;
        for clause in clauses {
            assert!(state.prove_clause(&mut facts, clause, true)?);
        }
        let steps = state.selection_steps();
        let proof = state.finish();
        assert!(valid(&facts, &proof, 0)?);
        let growth = facts.types.len() - before;
        println!(
            "universal-work width={width} steps={steps} types={growth} scopes={} regions={}",
            proof.scope_count(),
            proof.regions.end - proof.regions.start
        );
        observations.push((width, steps, growth));
    }
    for &(width, steps, growth) in &observations {
        assert!(
            growth <= 12 * width + 32,
            "{width}: {growth} canonical types"
        );
        assert!(
            steps <= 256 * width + 1024,
            "{width}: {steps} selection steps"
        );
    }
    for pair in observations.windows(2) {
        assert!(pair[1].1 <= 2 * pair[0].1 + 256, "{pair:?}");
        assert!(pair[1].2 <= 2 * pair[0].2 + 32, "{pair:?}");
    }
    Ok(())
}

#[test]
fn conjunctive_goals_keep_each_lifetime_debt_in_both_orders() -> TestResult {
    for (implementation, expected) in [
        ("impl<'a>B<'a>for u8{}", true),
        ("impl B<'static>for u8{}", false),
        ("impl<'a:'static>B<'a>for u8{}", false),
    ] {
        for reverse in [false, true] {
            let (report, mut facts) = tests::declarations(&format!(
                "trait A<'a>{{}}trait B<'a>{{}}impl<'a>A<'a>for u8{{}}{implementation}"
            ))?;
            let mut goals = Vec::new();
            for name in ["A", "B"] {
                goals.push(lifetime_goal(
                    &mut facts,
                    tests::definition(&report, name)?,
                    1,
                    &[RegionId::bound(0, 0)?],
                )?);
            }
            if reverse {
                goals.reverse();
            }
            let mut state = State::instance(&mut facts, 0)?;
            for goal in &goals {
                assert!(state.prove_clause(&mut facts, *goal, true)?);
            }
            // A completed selection still owns unpaid lifetime evidence.
            for goal in &goals {
                assert!(state.prove_clause(&mut facts, *goal, true)?);
            }
            let proof = state.finish();
            assert_eq!(proof.scope_count(), 1);
            assert_eq!(
                valid(&facts, &proof, 0)?,
                expected,
                "{implementation}; reverse={reverse}"
            );
        }
    }
    Ok(())
}

#[test]
fn shared_universes_preserve_slot_correlations_and_captures() -> TestResult {
    for (implementation, captured, expected) in [
        ("impl<'a,'b>B<'a,'b>for u8{}", false, true),
        ("impl<'a>B<'a,'a>for u8{}", false, false),
        ("impl<'a,'b>B<'a,'b>for u8{}", true, true),
        ("impl<'a>B<'a,'a>for u8{}", true, false),
    ] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a,'b>{{}}trait B<'a,'b>{{}}impl<'a,'b>A<'a,'b>for u8{{}}{implementation}"
        ))?;
        let regions = [
            RegionId::bound(0, 0)?,
            RegionId::bound(u16::from(captured), u16::from(!captured))?,
        ];
        let outer = u32::from(captured);
        let mut state = State::instance(&mut facts, outer)?;
        for name in ["A", "B"] {
            let goal = lifetime_goal(
                &mut facts,
                tests::definition(&report, name)?,
                2 - outer,
                &regions,
            )?;
            assert!(state.prove_clause(&mut facts, goal, true)?);
        }
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(
            valid(&facts, &proof, outer)?,
            expected,
            "{implementation}; captured={captured}"
        );
    }
    Ok(())
}

#[test]
fn empty_and_nonempty_universes_keep_distinct_scopes() -> TestResult {
    let (report, mut facts) = tests::declarations(
        "trait A<'a>{}trait B<'a,'b>{}impl<'a>A<'a>for u8{}impl<'a,'b>B<'a,'b>for u8{}",
    )?;
    let a = tests::definition(&report, "A")?;
    let b = tests::definition(&report, "B")?;
    let mut state = State::instance(&mut facts, 1)?;
    for (item, count, regions) in [
        (a, 0, vec![RegionId::bound(1, 0)?]),
        (a, 1, vec![RegionId::bound(0, 0)?]),
        (b, 2, vec![RegionId::bound(0, 0)?, RegionId::bound(0, 1)?]),
        (a, 0, vec![RegionId::STATIC]),
        (a, 1, vec![RegionId::bound(1, 0)?]),
        (b, 2, vec![RegionId::bound(0, 1)?, RegionId::bound(0, 0)?]),
    ] {
        let goal = lifetime_goal(&mut facts, item, count, &regions)?;
        assert!(state.prove_clause(&mut facts, goal, true)?);
    }
    let proof = state.finish();
    assert_eq!(proof.scope_count(), 3);
    assert!(valid(&facts, &proof, 1)?);
    Ok(())
}

#[test]
fn ordinary_and_absent_goals_do_not_reuse_quantified_witnesses() -> TestResult {
    for static_only in [false, true] {
        let implementation = if static_only {
            "impl C<'static>for u8{}"
        } else {
            "impl<'a>C<'a>for u8{}"
        };
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{}}trait B<'a>{{}}trait C<'a>{{}}trait Missing<'a>{{}}impl<'a>A<'a>for u8{{}}impl<'a>B<'a>for u8{{}}{implementation}"
        ))?;
        let mut state = State::instance(&mut facts, 1)?;
        for name in ["Missing", "A", "Missing", "B"] {
            let goal = lifetime_goal(
                &mut facts,
                tests::definition(&report, name)?,
                1,
                &[RegionId::bound(0, 0)?],
            )?;
            assert_eq!(
                state.prove_clause(&mut facts, goal, true)?,
                name != "Missing"
            );
            if name == "A" {
                let args = facts
                    .types
                    .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
                let ordinary = tests::predicate(
                    &mut facts,
                    tests::definition(&report, "C")?,
                    TypeId::primitive(Primitive::U8),
                    args,
                )?;
                assert!(state.prove(&mut facts, ordinary, true)?);
            }
        }
        let proof = state.finish();
        assert_eq!(proof.scope_count(), 1);
        assert_eq!(valid(&facts, &proof, 1)?, !static_only);
    }
    Ok(())
}

#[test]
fn shared_openings_preserve_nested_callable_captures() -> TestResult {
    for (capture, expected) in [("'a", true), ("'static", false)] {
        let (report, mut facts) = tests::declarations(&format!(
            "trait A<'a>{{}}trait B<'a,T>{{}}impl<'a>A<'a>for u8{{}}impl<'a>B<'a,for<'b>fn(&'b u8,&{capture} u8)>for u8{{}}"
        ))?;
        let mut references = Vec::new();
        for depth in [0, 1] {
            references.push(facts.types.intern(Kind::Reference {
                mutable: false,
                region: RegionId::bound(depth, 0)?,
                referent: TypeId::primitive(Primitive::U8),
            })?);
        }
        let parameters = facts.types.intern_list(&references)?;
        let function = facts.types.intern(Kind::Function {
            parameters,
            result: TypeId::UNIT,
            bound_regions: 1,
            unsafe_: false,
        })?;
        let args = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Type(function),
        ])?;
        let b = tests::predicate(
            &mut facts,
            tests::definition(&report, "B")?,
            TypeId::primitive(Primitive::U8),
            args,
        )?;
        let b = clause(&mut facts, 1, b)?;
        let a = lifetime_goal(
            &mut facts,
            tests::definition(&report, "A")?,
            1,
            &[RegionId::bound(0, 0)?],
        )?;
        let mut state = State::instance(&mut facts, 0)?;
        assert!(state.prove_clause(&mut facts, a, true)?);
        assert!(state.prove_clause(&mut facts, b, true)?);
        assert_eq!(valid(&facts, &state.finish(), 0)?, expected);
    }
    Ok(())
}
