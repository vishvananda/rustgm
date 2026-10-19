//! Partial exported prefixes share traversal and validation work across input roots.
use rustgm::types::{
    Store,
    infer::{Constructor, Context, Domain, arguments::Argument, refinement::Refinement},
    substitute::Bound,
};
use std::collections::HashMap;
#[test]
fn shared_partial_refinements_do_not_rescan_every_prefix() -> Result<(), Box<dyn std::error::Error>>
{
    let mut observations = Vec::new();
    for count in [16, 64, 256, 512, 2048] {
        let mut store = Store::default();
        let mut source = Context::default();
        let roots = (0..count)
            .map(|_| source.variable(Domain::Any).map(Argument::Type))
            .collect::<Result<Vec<_>, _>>()?;
        let (mut query, copied) = Refinement::new(&store, &mut source, &roots)?;
        let mut prefix = query.context().variable(Domain::Any)?;
        let mut pairs = Vec::new();
        for argument in copied {
            prefix = query.context().construct(Constructor::Tuple, &[prefix])?;
            let Argument::Type(root) = argument else {
                return Err("missing input".into());
            };
            pairs.push((Argument::Type(root), Argument::Type(prefix)));
        }
        query.context().equate_input_pairs(&store, &pairs)?;
        let setup = query.context().steps;
        let before = source.steps;
        let committed = query.commit(
            &mut store,
            &mut source,
            &mut Bound::restore_regions(HashMap::new()),
        )?;
        assert!(committed.changed);
        let work = source.steps - before;
        println!("roots={count} query_setup={setup} commit_work={work}");
        observations.push((count, work));
    }
    assert!(
        observations
            .iter()
            .all(|&(count, work)| work < 128 * count + 32),
        "shared-prefix work must remain linear: {observations:?}"
    );
    Ok(())
}
#[test]
fn shared_partial_parent_changes_are_reconciled_in_one_graph()
-> Result<(), Box<dyn std::error::Error>> {
    for refresh in [false, true] {
        for count in [16, 64, 256, 2048] {
            let mut store = Store::default();
            let mut source = Context::default();
            let roots = (0..count)
                .map(|_| source.variable(Domain::Any).map(Argument::Type))
                .collect::<Result<Vec<_>, _>>()?;
            let (mut query, copied) = Refinement::new(&store, &mut source, &roots)?;
            let source_leaf = source.variable(Domain::Any)?;
            let mut prefix = source_leaf;
            let mut pairs = Vec::new();
            for &root in &roots {
                prefix = source.construct(Constructor::Tuple, &[prefix])?;
                pairs.push((root, Argument::Type(prefix)));
            }
            source.equate_input_pairs(&store, &pairs)?;
            let query_leaf = query.context().variable(Domain::Integer)?;
            let mut prefix = query_leaf;
            let mut pairs = Vec::new();
            for root in copied {
                prefix = query.context().construct(Constructor::Tuple, &[prefix])?;
                pairs.push((root, Argument::Type(prefix)));
            }
            query.context().equate_input_pairs(&store, &pairs)?;
            if refresh {
                query.refresh(&store, &mut source)?;
                assert_eq!(source.unresolved_domain(source_leaf)?, Some(Domain::Any));
            }
            let before = source.steps;
            let committed = query.commit(
                &mut store,
                &mut source,
                &mut Bound::restore_regions(HashMap::new()),
            )?;
            assert!(committed.changed);
            assert_eq!(
                source.unresolved_domain(source_leaf)?,
                Some(Domain::Integer)
            );
            let work = source.steps - before;
            println!("parent_changes roots={count} refresh={refresh} commit_work={work}");
            assert!(work < 128 * count + 32);
        }
    }
    Ok(())
}
