//! Repeated complete closures share work; incomplete diamonds and caller proofs do not.
use super::Dependencies;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
        model::DefId,
    },
    sema::{
        dispatch::Dispatcher, model::Declarations, traits::implementations::Availability,
        values::Instance,
    },
    support::source::SourceFile,
    types::{GenericArgs, arguments::Argument},
};
use std::{fmt::Write as _, path::Path};

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn facts(
    source: &str,
) -> Result<(resolve::expansion::Report, Declarations), Box<dyn std::error::Error>> {
    let report = resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "dispatch.rs".into(),
            path: "dispatch.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut facts = crate::sema::bodies::analyze(&report, facts)?;
    facts.dispatch_dependencies = Dependencies::default();
    for initializer in &facts.initializers {
        assert!(initializer.type_checked());
        facts.dispatch_dependencies.complete(initializer.definition);
    }
    Ok((report, facts))
}
#[test]
fn completed_closures_share_across_callers_but_not_availability_modes()
-> Result<(), Box<dyn std::error::Error>> {
    for depth in [32, 96] {
        let mut source = String::from("trait A{const C:usize;}fn main(){let _=<S0 as A>::C;}");
        for i in 0..depth {
            write!(source, "struct S{i};impl A for S{i}{{const C:usize=")?;
            if i + 1 == depth {
                source.push_str("3;}");
            } else {
                write!(source, "<S{} as A>::C;}}", i + 1)?;
            }
        }
        let (report, mut facts) = facts(&source)?;
        let subject = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == "S0")
            .ok_or("missing subject")?;
        let subject = facts.definitions[subject].ok_or("missing subject type")?;
        let source = facts
            .constant_uses
            .values()
            .find(|instance| {
                facts
                    .types
                    .arguments(instance.arguments)
                    .is_some_and(|mut args| args.any(|arg| arg == Argument::Type(subject)))
            })
            .copied()
            .ok_or("missing constant use")?;
        let mut first = Dispatcher::declaration(1)?;
        let selected = first.constant(&mut facts, source)?;
        assert!(first.obligations.is_empty());
        let initial_work = first.context.steps;
        assert!(facts.dispatch_dependencies.declarations.len() >= depth);
        let mut repeated = Dispatcher::declaration(1)?;
        assert_eq!(repeated.constant(&mut facts, source)?, selected);
        assert!(repeated.context.steps * depth < initial_work * 2);
        let mut validated = Dispatcher::new(0, 1)?;
        assert_eq!(validated.constant(&mut facts, source)?, selected);
        assert!(validated.context.steps >= initial_work / 2);
        assert!(facts.dispatch_dependencies.validated.len() >= depth);
        let mut caller = Dispatcher::new(1, 1)?;
        assert_eq!(caller.constant(&mut facts, source)?, selected);
        assert!(caller.context.steps >= initial_work / 2);
    }
    Ok(())
}
#[test]
fn incomplete_diamond_never_publishes_a_completed_parent() -> Result<(), Box<dyn std::error::Error>>
{
    let (report, mut facts) = facts(
        "struct S;impl S{const C0:usize=Self::C1+Self::C2;const C1:usize=Self::C3;const C2:usize=Self::C3;const C3:usize=3;}fn main(){let _=S::C0;}",
    )?;
    let mut constants = Vec::new();
    for name in ["C0", "C1", "C2", "C3"] {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == name)
            .ok_or("missing constant")?;
        constants.push(Instance {
            definition: DefId(u32::try_from(index)?),
            arguments: GenericArgs::EMPTY,
        });
    }
    facts
        .dispatch_dependencies
        .ready
        .remove(&constants[3].definition);
    Dispatcher::new(0, 1)?.constant(&mut facts, constants[0])?;
    assert!(constants.iter().all(|&instance| {
        !facts
            .dispatch_dependencies
            .contains(Availability::Validated, instance)
    }));
    facts
        .dispatch_dependencies
        .complete(constants[3].definition);
    Dispatcher::new(0, 1)?.constant(&mut facts, constants[0])?;
    assert!(constants.iter().all(|&instance| {
        facts
            .dispatch_dependencies
            .contains(Availability::Validated, instance)
    }));
    Ok(())
}

#[test]
fn closed_lifetime_proofs_share_without_leaking_private_arguments()
-> Result<(), Box<dyn std::error::Error>> {
    let (_, mut facts) = facts(
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}fn main(){let _=<S as A<'static,'static>>::C;}",
    )?;
    let instance = *facts
        .constant_uses
        .values()
        .next()
        .ok_or("missing constant use")?;
    let mut target = None;
    for first in [1, 10, 100] {
        let mut dispatch = Dispatcher::new(0, first)?;
        let selected = dispatch.constant(&mut facts, instance)?;
        assert!(dispatch.finish().is_empty());
        assert!(facts.types.arguments_are_concrete(selected.arguments)?);
        assert_eq!(*target.get_or_insert(selected), selected);
        assert!(
            facts
                .dispatch_dependencies
                .contains(Availability::Validated, selected)
        );
        assert_eq!(facts.dispatch_work().selections, 1);
        assert_eq!(facts.dispatch_work().closed_proofs, 1);
        assert_eq!(facts.dispatch_work().dependency_expansions, 1);
    }
    assert_eq!(facts.dispatch_work().shared_selections, 2);
    let mut pattern = Dispatcher::pattern(&mut facts, 0, 10, None)?;
    pattern.constant(&mut facts, instance)?;
    assert!(
        !pattern.finish().is_empty(),
        "caller scopes retain their proof"
    );
    Ok(())
}

#[test]
fn lifetime_debts_are_retained_for_each_requesting_body() -> Result<(), Box<dyn std::error::Error>>
{
    use crate::types::{RegionId, outlives::Queries, relations::Constraints};
    let (_, mut facts) = facts(
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}fn main(){let _=<S as A<'static,'static>>::C;}",
    )?;
    let instance = *facts
        .constant_uses
        .values()
        .next()
        .ok_or("missing constant use")?;
    Dispatcher::new(0, 1)?.constant(&mut facts, instance)?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    for (first, equal, paid) in [(1, false, true), (10, false, false), (100, true, false)] {
        let mut values = facts
            .types
            .arguments(instance.arguments)
            .ok_or("missing arguments")?
            .collect::<Vec<_>>();
        values[0] = Argument::Lifetime(a);
        values[1] = Argument::Lifetime(if equal { a } else { b });
        let instance = Instance {
            arguments: facts.types.intern_arguments(&values)?,
            ..instance
        };
        let mut dispatch = Dispatcher::new(2, first)?;
        let selected = dispatch.constant(&mut facts, instance)?;
        let proof = dispatch.finish();
        assert!(!proof.is_empty());
        assert_eq!(proof.regions.start, first);
        assert!(
            !facts
                .dispatch_dependencies
                .contains(Availability::Validated, selected)
        );
        let mut constraints =
            Constraints::new(&facts.types, &facts.lifetimes.variances, 2, 1_000_000)?;
        if paid {
            constraints.assume(a, b)?;
        }
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        proof.require(&mut constraints, &mut queries)?;
        assert_eq!(constraints.solve().is_ok(), paid || equal);
    }
    assert_eq!(facts.dispatch_work().closed_proofs, 1);
    assert_eq!(facts.dispatch_work().shared_selections, 0);
    Ok(())
}

#[test]
fn unanchored_private_arguments_keep_the_original_proof() -> Result<(), Box<dyn std::error::Error>>
{
    let (_, mut facts) = facts(
        "trait A{const C:usize;}struct S;impl<'a>A for S{const C:usize=3;}fn main(){let _=<S as A>::C;}",
    )?;
    let instance = *facts
        .constant_uses
        .values()
        .next()
        .ok_or("missing constant use")?;
    for first in [1, 10] {
        let mut dispatch = Dispatcher::new(0, first)?;
        let selected = dispatch.constant(&mut facts, instance)?;
        let proof = dispatch.finish();
        assert!(!proof.is_empty());
        assert_eq!(proof.regions.start, first);
        assert!(!facts.types.arguments_are_concrete(selected.arguments)?);
        assert!(
            !facts
                .dispatch_dependencies
                .contains(Availability::Validated, selected)
        );
    }
    assert_eq!(
        facts.dispatch_work().selections,
        2,
        "fallback never reselects"
    );
    assert_eq!(facts.dispatch_work().closed_proofs, 0);
    Ok(())
}

#[test]
fn inherent_free_arguments_do_not_publish_an_independent_closure()
-> Result<(), Box<dyn std::error::Error>> {
    let (_, mut facts) = facts(
        "struct R<'a>(&'a u8);impl<'a>R<'a>{const C:usize=3;}fn main(){let _=<R<'static>>::C;}",
    )?;
    let instance = *facts
        .constant_uses
        .values()
        .next()
        .ok_or("missing constant use")?;
    let instance = Instance {
        arguments: facts
            .types
            .intern_arguments(&[Argument::Lifetime(crate::types::RegionId(1))])?,
        ..instance
    };
    let mut dispatch = Dispatcher::new(0, 10)?;
    let target = dispatch.constant(&mut facts, instance)?;
    assert!(dispatch.finish().is_empty());
    assert!(
        !facts
            .dispatch_dependencies
            .contains(Availability::Validated, target)
    );
    Ok(())
}

#[test]
fn closed_dispatch_cache_stays_usable_with_unrelated_latent_collection_headers()
-> Result<(), Box<dyn std::error::Error>> {
    let (_, mut facts) = facts(
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}struct I;impl Iterator for I{type Item=u8;fn next(&mut self)->Option<u8>{None}}fn main(){let _=<S as A<'static,'static>>::C;for _ in I{}}",
    )?;
    assert!(
        facts
            .iteration
            .as_deref()
            .is_some_and(|r| r.collections.is_none())
    );
    let instance = *facts.constant_uses.values().next().ok_or("constant")?;
    for first in [1, 10, 100] {
        let mut dispatch = Dispatcher::new(0, first)?;
        dispatch.constant(&mut facts, instance)?;
        assert!(dispatch.finish().is_empty());
    }
    assert_eq!(facts.dispatch_work().selections, 1);
    assert_eq!(facts.dispatch_work().closed_proofs, 1);
    assert_eq!(facts.dispatch_work().shared_selections, 2);
    assert!(
        facts
            .iteration
            .as_deref()
            .is_some_and(|r| r.collections.is_none())
    );
    Ok(())
}

#[test]
fn source_observer_borrows_only_each_selection_and_never_the_growing_prefix()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{RegionId, outlives::Queries, relations::Constraints};
    let (_, mut facts) = facts(
        "trait A<'a,'b>{const C:usize;}struct S;impl<'a:'b,'b>A<'a,'b>for S{const C:usize=3;}fn main(){let _=<S as A<'static,'static>>::C;}",
    )?;
    let instance = *facts.constant_uses.values().next().ok_or("use")?;
    let a = RegionId::bound(0, 0)?;
    let b = RegionId::bound(0, 1)?;
    let mut arguments = facts
        .types
        .arguments(instance.arguments)
        .ok_or("arguments")?
        .collect::<Vec<_>>();
    arguments[0] = Argument::Lifetime(a);
    arguments[1] = Argument::Lifetime(b);
    let input = Instance {
        arguments: facts.types.intern_arguments(&arguments)?,
        ..instance
    };
    for (first, paid) in [(100, true), (300, false), (500, true)] {
        let mut dispatch = Dispatcher::new(2, first)?;
        let mut opening = crate::types::lifetime_proof::Obligations::new(first);
        opening.regions.end = first + 2;
        opening
            .equalities
            .extend([(RegionId(first), a), (RegionId(first + 1), b)]);
        let mut fresh = 0;
        let mut warm = 0;
        let mut visits = 0;
        let mut selected = None;
        let mut observe =
            |facts: &mut Declarations, observed: crate::sema::dispatch::ConstantProof<'_>| {
                assert_eq!(observed.input, input);
                assert!(matches!(observed.availability, Availability::Validated));
                if let Some(proof) = observed.proof {
                    fresh += 1;
                    assert_eq!(proof.regions.start, first + 2);
                    assert_eq!(observed.first, proof.regions.end);
                    visits += proof.retained_allocation().visits;
                    let mut constraints =
                        Constraints::new(&facts.types, &facts.lifetimes.variances, 2, 1_000_000)
                            .map_err(|e| e.to_string())?;
                    if paid {
                        constraints.assume(a, b).map_err(|e| e.to_string())?;
                    }
                    let mut queries = Queries::new(
                        &facts.types,
                        &facts.lifetimes.nominal_requirements,
                        &facts.lifetimes.callables,
                    );
                    opening.require(&mut constraints, &mut queries)?;
                    proof.require(&mut constraints, &mut queries)?;
                    constraints.solve().map_err(|e| e.to_string())?;
                    selected = Some(observed.selected);
                } else {
                    warm += 1;
                    assert_eq!(
                        selected,
                        Some(observed.selected),
                        "warm identity alone cannot mint authority"
                    );
                }
                Ok(crate::sema::dispatch::ConstantDecision::Checked(None))
            };
        let result = dispatch.source_constant(&mut facts, input, &mut observe);
        assert_eq!(result.is_ok(), paid);
        if paid {
            let expected = result?;
            for _ in 0..128 {
                assert_eq!(
                    dispatch.source_constant(&mut facts, input, &mut observe)?,
                    expected
                );
            }
            assert_eq!((fresh, warm), (1, 128));
            assert!(visits < 32);
            assert!(
                !dispatch.finish().is_empty(),
                "observer must not consume caller debts"
            );
        } else {
            assert_eq!(fresh, 1);
            assert_eq!(warm, 0);
            assert!(
                !dispatch.constants.contains_key(&input),
                "failed callback cannot publish selection"
            );
        }
    }
    Ok(())
}
