//! Revision-zero input bridges are sealed by immutable identity, not just a count.
use super::*;
use crate::sema::nested_outputs::{Bridge, Formation, Link};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn copy_link(link: &Link) -> Link {
    Link {
        source: link.source,
        parent: link.parent,
        revision: link.revision,
        universals: link.universals,
        bridges: link.bridges.clone(),
        formation: link.formation.as_ref().map(|f| Formation {
            identity: f.identity,
            inputs: f.inputs.clone(),
        }),
    }
}
#[test]
fn input_only_summary_rejects_wrong_parent_and_same_revision_image() -> Result {
    let source = "fn make(x:&'static u8)->impl Fn()->&'static u8+'static{||&*x}fn main(){}";
    let report = crate::test_source::resolve(source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut templates = crate::body::templates::build(&report, &facts)?;
    crate::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let inputs = crate::body::regions::collect(&facts)?.bodies;
    for input in &inputs {
        input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    let proof = crate::borrow::regions::check_templates(
        &facts.types,
        &facts.lifetimes.variances,
        &Contracts {
            bodies: inputs.clone(),
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        },
        &templates,
    )?;
    let input = inputs
        .iter()
        .find(|input| {
            input
                .output
                .as_ref()
                .is_some_and(|link| !link.bridges.is_empty())
        })
        .ok_or("input-only contract")?;
    let link = input.output.as_ref().ok_or("link")?;
    assert_eq!(link.revision, 0);
    assert!(link.formation.is_none());
    assert!(link.bridges.iter().all(|bridge| !bridge.output));
    let summary = proof
        .bodies
        .iter()
        .find(|body| body.key == input.key)
        .and_then(|body| body.output.as_ref())
        .ok_or("sealed summary")?;
    let conditions = summary
        .execution_conditions(input)?
        .ok_or("conditional debt")?;
    assert_eq!(
        conditions.as_ref(),
        &[(RegionId::bound(0, 0)?, RegionId::STATIC)]
    );
    let mut shared = Arc::clone(link);
    assert!(
        Arc::get_mut(&mut shared).is_none(),
        "checked map must remain immutable"
    );
    for wrong_image in [false, true] {
        let mut forged = input.clone();
        let mut changed = copy_link(link);
        if wrong_image {
            let bridge = changed.bridges.first_mut().ok_or("bridge")?;
            *bridge = Bridge {
                parent: RegionId::STATIC,
                ..*bridge
            };
        } else {
            changed.parent = inputs
                .iter()
                .find(|candidate| {
                    candidate.source != link.parent && candidate.source != link.source
                })
                .ok_or("different parent")?
                .source;
        }
        assert_eq!(changed.revision, link.revision);
        assert_eq!(changed.bridges.len(), link.bridges.len());
        forged.output = Some(Arc::new(changed));
        assert!(
            summary.execution_conditions(&forged).is_err(),
            "same revision authorized a changed map"
        );
    }
    Ok(())
}
