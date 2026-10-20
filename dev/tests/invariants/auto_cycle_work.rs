//! Shared cyclic conditions must not be replayed along every path through a diamond.
use super::{Decision, Frame, Predicate, Search, tests};
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{GenericArgs, ItemId, Kind, auto_traits::AutoTrait, parameters::predicates::Bound},
};
use std::fmt::Write;

#[derive(Clone, Copy, Debug)]
enum Graph {
    Plain,
    Captured,
    Quantified,
}
impl Graph {
    fn source(self, width: usize) -> Result<String, std::fmt::Error> {
        let mut source = String::new();
        for index in 0..width {
            match self {
                Self::Plain => write!(source, "struct N{index};impl Unpin for N{index} where ")?,
                Self::Captured => write!(
                    source,
                    "struct N{index}<'a>(&'a u8);impl<'a> Unpin for N{index}<'a> where "
                )?,
                Self::Quantified => write!(
                    source,
                    "struct N{index};unsafe impl Sync for N{index} where "
                )?,
            }
            for next in [(index + 1) % width]
                .into_iter()
                .chain((index + 2..width).take(1))
            {
                match self {
                    Self::Plain => write!(source, "N{next}:Unpin,")?,
                    Self::Captured => write!(source, "N{next}<'static>:Unpin,")?,
                    Self::Quantified => write!(source, "for<'a> &'a N{next}:Sync,")?,
                }
            }
            source.push_str("{}");
        }
        Ok(source)
    }
}

#[test]
fn explicit_auto_diamonds_share_cyclic_subgoals() -> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{RegionId, arguments::Argument};
    for graph in [Graph::Plain, Graph::Captured, Graph::Quantified] {
        for width in [8, 16, 32, 64] {
            let (report, mut facts) = tests::declarations(&graph.source(width)?)?;
            let arguments = if matches!(graph, Graph::Captured) {
                facts
                    .types
                    .intern_arguments(&[Argument::Lifetime(RegionId::STATIC)])?
            } else {
                GenericArgs::EMPTY
            };
            let ty = facts.types.intern(Kind::Adt(
                ItemId(tests::definition(&report, "N0")?.0),
                arguments,
            ))?;
            let marker = if matches!(graph, Graph::Quantified) {
                AutoTrait::Sync
            } else {
                AutoTrait::Unpin
            };
            let (mut context, goal) = tests::instantiate(
                &mut facts,
                Predicate {
                    ty,
                    bound: Bound::Auto(marker),
                },
            )?;
            let assumptions = Assumptions::default();
            let mut caller = tests::caller(&assumptions, None);
            caller.first_region = 100;
            let mut search = Search::new(caller, Availability::Validated);
            let frame = Frame::new(&facts, &mut context, &goal, 100)?;
            let Decision::Proven(root) = search.run(&mut facts, frame)? else {
                return Err(format!("missing {graph:?} proof at {width}").into());
            };
            assert!(search.query.active.is_empty());
            assert!(search.query.auto_active.is_empty());
            assert!(
                search
                    .query
                    .auto_provisional
                    .values()
                    .flat_map(|values| values.values())
                    .all(|answer| {
                        let Decision::Proven(index) = answer else {
                            return false;
                        };
                        let node = &search.query.nodes[*index];
                        !node.cycle_dependent || !node.children.is_empty()
                    }),
                "a cycle assumption must never replace a completed proof"
            );
            println!(
                "auto-cycle {graph:?} width={width} steps={}",
                search.query.steps
            );
            assert!(
                search.query.steps < width * 1000 + 1000,
                "{graph:?} {width}: {}",
                search.query.steps
            );
            let evidence = search.evidence(root)?;
            assert!(super::size_evidence_tests::valid(&facts, &evidence.proof)?);
            if matches!(graph, Graph::Captured) {
                assert!(evidence.proof.equalities.len() >= width);
            }
        }
    }
    Ok(())
}

#[test]
fn failed_auto_components_invalidate_all_provisional_answers()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{Primitive, TypeId, arguments::Argument};
    let source = "struct A<T>(fn(T));struct B<T>(fn(T));struct C<T>(A<T>,B<T>);
        impl<T> Unpin for A<T> where B<T>:Unpin,T:Sync{}
        impl<T> Unpin for B<T> where A<T>:Unpin{}";
    let (report, mut facts) = tests::declarations(source)?;
    let byte = TypeId::primitive(Primitive::U8);
    let pointer = facts.types.intern(Kind::Pointer {
        mutable: false,
        pointee: byte,
    })?;
    for (input, expected) in [(pointer, false), (byte, true), (pointer, false)] {
        let arguments = facts.types.intern_arguments(&[Argument::Type(input)])?;
        let ty = facts.types.intern(Kind::Adt(
            ItemId(tests::definition(&report, "C")?.0),
            arguments,
        ))?;
        let (mut context, goal) = tests::instantiate(
            &mut facts,
            Predicate {
                ty,
                bound: Bound::Auto(AutoTrait::Unpin),
            },
        )?;
        let assumptions = Assumptions::default();
        let mut caller = tests::caller(&assumptions, None);
        caller.first_region = 100;
        let mut search = Search::new(caller, Availability::Validated);
        let frame = Frame::new(&facts, &mut context, &goal, 100)?;
        let result = search.run(&mut facts, frame)?;
        assert_eq!(matches!(result, Decision::Proven(_)), expected);
        assert!(
            search.query.cycles > 0,
            "the failure must follow a provisional cycle"
        );
        assert!(search.query.active.is_empty());
        assert!(search.query.auto_active.is_empty());
        assert!(
            search
                .query
                .completed
                .values()
                .all(|decision| match decision {
                    Decision::Proven(index) => !search.query.nodes[*index].cycle_dependent,
                    _ => true,
                })
        );
        if !expected {
            assert!(search.query.auto_provisional.is_empty());
        }
    }
    Ok(())
}
