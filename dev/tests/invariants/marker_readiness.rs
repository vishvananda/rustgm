//! An unresolved impl alias cannot silently authorize a structural marker default.
use super::{Decision, Dependency, Frame, Predicate, Search, continuation::Progress, tests};
use crate::{
    sema::traits::{
        assumptions::Assumptions,
        implementations::{Availability, Identity, Implementation, Implementations, PendingHeader},
    },
    types::{GenericArgs, ItemId, Kind, auto_traits::AutoTrait, parameters::predicates::Bound},
};

#[test]
fn pending_marker_aliases_resume_the_correct_nominal_rule() -> Result<(), Box<dyn std::error::Error>>
{
    for (source, marker, expected) in [
        (
            "struct W;struct V;type Alias=V;impl Unpin for Alias{}",
            AutoTrait::Unpin,
            true,
        ),
        (
            "struct W(*const u8);type Alias=W;unsafe impl Send for Alias{}",
            AutoTrait::Send,
            true,
        ),
        (
            "struct W(*const u8);struct V(*const u8);type Alias=V;unsafe impl Send for Alias{}",
            AutoTrait::Send,
            false,
        ),
    ] {
        let (report, mut facts) = tests::declarations(source)?;
        let header = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|registry| registry.headers().next())
            .ok_or("missing marker impl")?;
        let header = Implementation {
            owner: header.owner,
            source: header.source,
            predicate: header.predicate,
            methods: Vec::new(),
            types: Vec::new(),
            constants: Vec::new(),
        };
        let owner = header.owner;
        let mut registry = Implementations::default();
        registry.reserve(vec![PendingHeader {
            owner,
            item: Identity::Auto(marker),
            head: None,
        }])?;
        facts.generics.implementations = Some(Box::new(registry));
        let ty = facts.types.intern(Kind::Adt(
            ItemId(tests::definition(&report, "W")?.0),
            GenericArgs::EMPTY,
        ))?;
        let (mut context, goal) = tests::instantiate(
            &mut facts,
            Predicate {
                ty,
                bound: Bound::Auto(marker),
            },
        )?;
        let assumptions = Assumptions::default();
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 100)?;
        assert_eq!(
            search.begin(&mut facts, frame)?,
            Progress::Pending(Dependency::Header(owner))
        );
        let steps = search.query.steps;
        assert_eq!(
            search.poll(&mut facts)?,
            Progress::Pending(Dependency::Header(owner))
        );
        assert_eq!(
            search.query.steps, steps,
            "waiting must not repeat field or header work"
        );
        let registry = facts
            .generics
            .implementations
            .as_mut()
            .ok_or("missing pending registry")?;
        registry.prepare(&mut facts.types, Identity::Auto(marker), header)?;
        registry.complete(owner, Vec::new(), Vec::new(), Vec::new())?;
        let Progress::Ready(decision) = search.poll(&mut facts)? else {
            return Err("completed marker header did not release its query".into());
        };
        assert_eq!(matches!(decision, Decision::Proven(_)), expected);
        assert!(search.query.auto_active.is_empty());
        assert!(search.query.active.is_empty());
    }
    Ok(())
}

#[test]
fn hidden_marker_queries_wait_without_replaying_and_keep_private_traits_private()
-> Result<(), Box<dyn std::error::Error>> {
    for (source, expected) in [
        ("trait A{}impl A for u8{}fn make()->impl A{7u8}", true),
        (
            "trait A{}impl A for *const u8{}fn make(x:*const u8)->impl A{x}",
            false,
        ),
        (
            "trait A{}impl A for u8{}fn make()->impl A{other()}fn other()->impl A{7u8}",
            true,
        ),
    ] {
        let (report, mut facts) = tests::declarations(source)?;
        let owner = tests::definition(&report, "make")?;
        let (&id, declaration) = facts
            .opaques
            .declarations
            .iter()
            .find(|(_, declaration)| declaration.owner == owner)
            .ok_or("missing defining opaque result")?;
        let ty = declaration.application;
        let assumptions = Assumptions::default();
        let mut caller = tests::caller(&assumptions, None);
        caller.first_region = 100;
        let (mut context, goal) = tests::instantiate(
            &mut facts,
            Predicate {
                ty,
                bound: Bound::Auto(AutoTrait::Send),
            },
        )?;
        let mut search = Search::new(caller, Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 100)?;
        assert_eq!(
            search.begin(&mut facts, frame)?,
            Progress::Pending(Dependency::Opaque(id))
        );
        let steps = search.query.steps;
        assert_eq!(
            search.poll(&mut facts)?,
            Progress::Pending(Dependency::Opaque(id))
        );
        assert_eq!(steps, search.query.steps);
        facts = crate::sema::bodies::analyze(&report, facts)?;
        let Progress::Ready(result) = search.poll(&mut facts)? else {
            return Err("completed defining body did not release auto query".into());
        };
        assert_eq!(matches!(result, Decision::Proven(_)), expected);
        assert!(search.query.auto_active.is_empty());
        assert!(search.query.active.is_empty());
        if let Decision::Proven(root) = result {
            assert!(matches!(
                search.evidence(&facts, root)?.source,
                super::Source::Builtin
            ));
        }
        let (mut context, goal) = tests::instantiate(
            &mut facts,
            Predicate {
                ty,
                bound: Bound::Copy,
            },
        )?;
        let mut search = Search::new(tests::caller(&assumptions, None), Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 100)?;
        assert_eq!(search.run(&mut facts, frame)?, Decision::Absent);
    }
    Ok(())
}

#[test]
fn defining_body_dependencies_keep_linear_work_and_sparse_suspended_storage()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    for (nested, borrow) in [(false, false), (true, false), (true, true)] {
        for width in [8, 32, 128] {
            let mut source = String::from(
                "fn main(){need(make0());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}",
            );
            for index in 0..width {
                write!(source, "fn make{index}()->impl A{{")?;
                let value = if borrow {
                    source.push_str("let n=7u8;let p=&n;");
                    "*p"
                } else {
                    "7u8"
                };
                if index + 1 == width {
                    source.push_str(value);
                } else if nested {
                    write!(source, "need(make{}());{value}", index + 1)?;
                } else {
                    write!(source, "make{}()", index + 1)?;
                }
                source.push('}');
            }
            let (report, mut facts) = tests::declarations(&source)?;
            // Declaration preparation order cannot become a body-proof prerequisite.
            facts
                .functions
                .sort_unstable_by_key(|function| function.definition.0);
            let facts = crate::sema::bodies::analyze(&report, facts)?;
            let work = &facts.body_work;
            println!(
                "opaque bodies={width} nested={nested} borrow={borrow} steps={} inference={} suspensions={} peak_slots={}",
                work.steps, work.inference_steps, work.suspensions, work.peak_suspended_slots
            );
            assert_eq!(work.suspensions, width);
            assert_eq!(work.suspended_slots, 0);
            assert!(work.steps < width * 80 + 100);
            assert!(work.inference_steps < width * 500 + 100);
            assert!(work.peak_suspended_slots < width * 100);
            assert_eq!(facts.opaques.body_results.len(), width);
        }
    }
    Ok(())
}

#[test]
fn final_nominal_dependencies_keep_solved_bodies_and_bounded_proof_work()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut prior = None;
    for width in [8, 32, 128] {
        let mut source = String::from(
            "fn main(){need(make0());}fn need<T:Send>(_:T){}trait A{}impl A for u8{}struct W<T:Send>(T);",
        );
        for index in 0..width {
            write!(source, "fn make{index}()->impl A{{")?;
            if index + 1 < width {
                write!(source, "let _=W(make{}());", index + 1)?;
            }
            source.push_str("7u8}");
        }
        let (report, mut facts) = tests::declarations(&source)?;
        facts
            .functions
            .sort_unstable_by_key(|function| function.definition.0);
        let facts = crate::sema::bodies::analyze(&report, facts)?;
        let work = &facts.body_work;
        println!(
            "final bodies={width} steps={} inference={} canonical={} suspensions={} peak_slots={}",
            work.steps,
            work.inference_steps,
            work.canonical_steps,
            work.suspensions,
            work.peak_suspended_slots
        );
        assert_eq!(work.suspensions, width);
        assert_eq!(work.suspended_slots, 0);
        assert!(work.canonical_steps < width * 200);
        assert!(work.peak_suspended_slots < width * 100);
        if let Some(previous) = prior {
            assert!(work.canonical_steps < previous * 5);
        }
        prior = Some(work.canonical_steps);
        assert_eq!(facts.opaques.body_results.len(), width);
    }
    Ok(())
}
