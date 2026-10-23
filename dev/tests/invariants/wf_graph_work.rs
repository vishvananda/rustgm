//! Source nominal formation work distinguishes depth from unrelated proof nodes.
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, implementations::Availability},
    types::{GenericArgs, ItemId, Kind, arguments::Argument},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn nested_nominal_formation_records_actual_work_and_retained_edges() -> Result {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-freeze/nominal-no-override.rs")?;
    for axis in ["depth", "width"] {
        let mut previous: Option<(usize, usize, usize)> = None;
        for extent in [8, 32, 128] {
            let depth = if axis == "depth" { extent } else { 1 };
            let mut measured = None;
            for noise in [0, 128] {
                let report = crate::test_source::source_provider(&source, "formation_work")?;
                let mut facts = crate::sema::declarations::analyze(&report)?;
                let mut ty = facts.types.intern(Kind::Adt(
                    ItemId(tests::definition(&report, "N")?.0),
                    GenericArgs::EMPTY,
                ))?;
                let item = ItemId(tests::definition(&report, "W")?.0);
                for _ in 0..depth {
                    let arguments = facts.types.intern_arguments(&[Argument::Type(ty)])?;
                    ty = facts.types.intern(Kind::Adt(item, arguments))?;
                }
                if axis == "width" {
                    let fields = facts.types.intern_list(&vec![ty; extent])?;
                    ty = facts.types.intern(Kind::Tuple(fields))?;
                }
                let bound = freezing::bound(&mut facts)?;
                let (mut context, goal) = tests::instantiate(&mut facts, Predicate { ty, bound })?;
                let assumptions = Assumptions::default();
                let mut search =
                    Search::new(tests::caller(&assumptions, None), Availability::Validated);
                for _ in 0..noise {
                    search.query.push(
                        Node {
                            debt_free: false,
                            cycle_dependent: false,
                            provisional_size: false,
                            goal: Predicate {
                                ty: crate::types::TypeId::UNIT,
                                bound: crate::types::parameters::predicates::Bound::Sized,
                            },
                            source: Some(Source::Builtin),
                            children: Vec::new(),
                            first: 1,
                            end: 1,
                            equalities: Vec::new(),
                            outlives: Vec::new(),
                            type_outlives: Vec::new(),
                            sized: Vec::new(),
                        },
                        crate::types::lifetime_proof::ScopeId::ROOT,
                        1..1,
                    );
                }
                let first = search.query.steps;
                let frame = Frame::new(&facts, &mut context, &goal, 1)?;
                let Decision::Proven(_) = search.run(&mut facts, frame)? else {
                    return Err("nested source Freeze formation failed".into());
                };
                let work = search.query.steps - first;
                let nodes = search.query.nodes.len() - noise;
                let edges: usize = search.query.nodes[noise..]
                    .iter()
                    .map(|n| n.children.len())
                    .sum();
                println!(
                    "axis={axis} extent={extent} noise={noise} work={work} nodes={nodes} edges={edges}"
                );
                let current = (work, nodes, edges);
                if let Some(previous) = measured {
                    assert_eq!(previous, current);
                } else {
                    measured = Some(current);
                }
                assert!(
                    work <= 1024 * extent + 1024,
                    "formation work must stay linear: {axis} {extent} {work}"
                );
                assert!(
                    edges <= 128 * extent + 128,
                    "formation edges must stay linear: {axis} {extent} {edges}"
                );
            }
            let (work, _, edges) = measured.ok_or("missing formation observation")?;
            if let Some((old_depth, old_work, old_edges)) = previous {
                let factor = extent / old_depth;
                assert!(work <= factor * old_work + 1024);
                assert!(edges <= factor * old_edges + 128);
            }
            previous = Some((extent, work, edges));
        }
    }
    Ok(())
}
