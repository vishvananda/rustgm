//! Transport metadata rejects foreign identity without becoming proof authority.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Fixture {
    facts: Declarations,
    body: Body,
    input: Contract,
    sources: Sources,
    foreign_source: NodeKey,
}
fn fixture(width: usize) -> Result<Fixture> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/inline-query-context/paid-lifetime.rs"),
    )?;
    let extra = "let _=const{S::<'a,Tag>::C};".repeat(width - 1);
    let source = source.replace("let f=||{const{", &format!("let f=||{{{extra}const{{"));
    let report = crate::test_source::resolve(&source, true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let facts = analysis.facts;
    let mut bodies = crate::body::build_concrete(&report, &facts)?.bodies;
    let position = bodies
        .iter()
        .position(|body| matches!(body.owner, Owner::Closure { .. }))
        .ok_or("parent")?;
    let body = bodies.swap_remove(position);
    let input = crate::body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|c| c.key == body.key())
        .ok_or("parent contract")?;
    let mut sources = Sources::default();
    for value in &body.values {
        if let Rvalue::InlineInstance(instance) = value.kind {
            let link = facts
                .nested_outputs
                .as_deref()
                .and_then(|r| r.get(&instance.source()))
                .ok_or("link")?;
            sources.insert(instance.source(), body.source, link, &[], None)?;
        }
    }
    let unit = facts.body_regions.unit_of(body.source);
    let foreign_source = bodies
        .iter()
        .find(|other| facts.body_regions.unit_of(other.source) != unit)
        .map(|other| other.source)
        .ok_or("unrelated numbering unit")?;
    Ok(Fixture {
        facts,
        body,
        input,
        sources,
        foreign_source,
    })
}
#[test]
fn exact_ports_reject_foreign_epoch_body_instance_result_and_stale_operands() -> Result {
    let Fixture {
        facts,
        mut body,
        input: mut contract,
        mut sources,
        foreign_source,
    } = fixture(1)?;
    let pending = sources
        .prepare(&facts, &body, &mut contract)?
        .ok_or("ports")?;
    let port = &pending.ports[0];
    let mut owner_work = 0;
    assert!(
        pending
            .request(
                &facts,
                &body,
                &contract,
                port.instance,
                port.result,
                &mut owner_work
            )
            .is_ok()
    );
    assert!(
        pending
            .request(
                &facts,
                &body,
                &contract,
                port.instance,
                TypeId(u32::MAX),
                &mut owner_work
            )
            .is_err()
    );
    let foreign = fixture(1)?.facts;
    assert!(
        pending
            .validate(&foreign, &body, &contract, &mut owner_work)
            .is_err()
    );
    let original_source = body.source;
    body.source = foreign_source;
    assert!(
        pending
            .validate(&facts, &body, &contract, &mut owner_work)
            .is_err()
    );
    body.source = original_source;
    body.instance =
        Some(crate::body::instantiate::original_inline_context(&facts, &body)?.ok_or("context")?);
    assert!(
        pending
            .validate(&facts, &body, &contract, &mut owner_work)
            .is_err()
    );
    body.instance = None;
    let original_ty = body.values[port.operand.0 as usize].ty;
    body.values[port.operand.0 as usize].ty = TypeId(u32::MAX);
    assert!(
        pending
            .validate(&facts, &body, &contract, &mut owner_work)
            .is_err()
    );
    body.values[port.operand.0 as usize].ty = original_ty;
    let parent = contract.source;
    contract.source = foreign_source;
    assert!(
        pending
            .validate(&facts, &body, &contract, &mut owner_work)
            .is_err()
    );
    contract.source = parent;
    assert!(
        pending
            .validate(&facts, &body, &contract, &mut owner_work)
            .is_ok()
    );
    eprintln!(
        "ports={} prepare-work={} retained={}",
        pending.ports.len(),
        sources.steps,
        pending.retained_bytes()
    );
    Ok(())
}

#[test]
fn preparation_visits_each_port_and_bridge_once() -> Result {
    for width in [1, 8, 32, 128] {
        let Fixture {
            facts,
            body,
            mut input,
            mut sources,
            ..
        } = fixture(width)?;
        let before = sources.steps;
        let bridges: usize = sources
            .children
            .values()
            .map(|child| child.link.bridges.len())
            .sum();
        let pending = sources.prepare(&facts, &body, &mut input)?.ok_or("ports")?;
        assert_eq!(pending.ports().len(), width);
        assert_eq!(
            sources.steps - before,
            body.values.len() + bridges + 2 * width + 1
        );
        eprintln!(
            "width={width} preparation-visits={} retained={}",
            sources.steps - before,
            pending.retained_bytes()
        );
    }
    Ok(())
}

fn annotation(body: NodeKey) -> crate::sema::model::BodyRegion {
    crate::sema::model::BodyRegion {
        body,
        source: body,
        origin: crate::sema::model::BodyRegionOrigin::Annotation,
    }
}
#[test]
fn coincident_foreign_unit_ids_cannot_replace_a_prepared_child() -> Result {
    let Fixture {
        mut facts,
        body,
        mut input,
        mut sources,
        foreign_source,
    } = fixture(1)?;
    let pending = sources.prepare(&facts, &body, &mut input)?.ok_or("ports")?;
    let port = &pending.ports[0];
    let original = std::mem::take(&mut facts.body_regions);
    facts.body_regions.nest(body.source, pending.unit)?;
    let parent = facts.body_regions.allocate(annotation(body.source))?;
    let child = facts
        .body_regions
        .allocate(annotation(port.instance.source()))?;
    assert_eq!(
        parent, child,
        "unrelated units deliberately reuse the same number"
    );
    assert_ne!(
        facts.body_regions.unit_of(body.source),
        facts.body_regions.unit_of(port.instance.source())
    );
    let mut work = 0;
    assert!(
        pending
            .request(&facts, &body, &input, port.instance, port.result, &mut work)
            .is_err()
    );
    assert_eq!(
        work, 2,
        "parent and child owner queries are charged on failure"
    );
    assert!(pending.validate(&facts, &body, &input, &mut work).is_err());
    let mut copied_input = input.clone();
    assert!(sources.prepare(&facts, &body, &mut copied_input).is_err());
    // A changed shared unit is foreign even when parent and child still agree.
    assert_ne!(foreign_source, pending.unit);
    facts.body_regions = crate::sema::numbered_regions::BodyRegions::default();
    facts.body_regions.nest(body.source, foreign_source)?;
    facts
        .body_regions
        .nest(port.instance.source(), foreign_source)?;
    let before = work;
    assert!(
        pending
            .request(&facts, &body, &input, port.instance, port.result, &mut work)
            .is_err()
    );
    assert_eq!(
        work - before,
        1,
        "changed parent unit fails before child lookup"
    );
    // Neither same-shaped source facts nor a warm transport row can grant a unit.
    facts.body_regions = original;
    let retained = pending.retained_bytes();
    for _ in 0..3 {
        let before = work;
        pending.request(&facts, &body, &input, port.instance, port.result, &mut work)?;
        assert_eq!(work - before, 2);
        assert_eq!(pending.retained_bytes(), retained);
    }
    Ok(())
}
#[test]
fn nested_shared_units_survive_foreign_allocation_interleaving() -> Result {
    let Fixture {
        mut facts,
        body,
        mut input,
        mut sources,
        foreign_source,
    } = fixture(1)?;
    let pending = sources.prepare(&facts, &body, &mut input)?.ok_or("ports")?;
    let port = &pending.ports[0];
    assert_ne!(foreign_source, pending.unit);
    let original = std::mem::take(&mut facts.body_regions);
    facts.body_regions.nest(body.source, pending.unit)?;
    facts
        .body_regions
        .nest(port.instance.source(), body.source)?;
    let parent = facts.body_regions.allocate(annotation(body.source))?;
    let unrelated = facts.body_regions.allocate(annotation(foreign_source))?;
    let child = facts
        .body_regions
        .allocate(annotation(port.instance.source()))?;
    assert_eq!(parent, unrelated);
    assert_ne!(parent, child);
    assert_eq!(
        facts.body_regions.unit_of(port.instance.source()),
        pending.unit
    );
    assert_eq!(
        facts.body_regions.get(body.source, child).map(|r| r.body),
        Some(port.instance.source())
    );
    let mut work = 0;
    pending.request(&facts, &body, &input, port.instance, port.result, &mut work)?;
    assert_eq!(work, 2);
    pending.validate(&facts, &body, &input, &mut work)?;
    // The transport check above is not a body proof: original slot obligations
    // and the paired parent region proof are still independently mandatory.
    facts.body_regions = original;
    Ok(())
}
#[test]
fn preparation_never_scans_unrelated_unit_suffixes() -> Result {
    for width in [1, 8, 32, 128] {
        for unrelated in [0, 128, 4096] {
            let Fixture {
                mut facts,
                body,
                mut input,
                mut sources,
                foreign_source,
            } = fixture(width)?;
            let expected = body.values.len()
                + sources
                    .children
                    .values()
                    .map(|child| child.link.bridges.len())
                    .sum::<usize>()
                + 2 * width
                + 1;
            for _ in 0..unrelated {
                facts.body_regions.allocate(annotation(foreign_source))?;
            }
            let before = sources.steps;
            let pending = sources.prepare(&facts, &body, &mut input)?.ok_or("ports")?;
            assert_eq!(pending.ports().len(), width);
            assert_eq!(sources.steps - before, expected);
            eprintln!(
                "width={width} unrelated-regions={unrelated} exact-preparation-visits={expected}"
            );
        }
    }
    Ok(())
}
