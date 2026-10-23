//! Parent originals are one bounded transaction, independent of later execution.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture() -> Result<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/inline-query-context/paid-lifetime.rs"),
    )?)
}
#[test]
fn associated_and_component_demand_orders_reuse_the_complete_contract() -> Result {
    let text = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa25/tests/spec/inline-query-context/siblings.rs"),
    )?;
    for associated_first in [false, true] {
        let report = crate::test_source::resolve(&text, true)?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let initializer = facts
            .initializers
            .iter()
            .copied()
            .find(|i| {
                facts.inline_constants.as_deref().is_some_and(|registry| {
                    registry
                        .bodies()
                        .any(|b| b.parent_body == i.semantic_source())
                })
            })
            .ok_or("inline initializer owner")?;
        let source = facts
            .inline_constants
            .as_deref()
            .ok_or("inline registry")?
            .bodies()
            .find(|b| {
                b.parent_body == initializer.semantic_source()
                    && facts
                        .nested_outputs
                        .as_deref()
                        .is_some_and(|r| r.get(&b.source).is_some())
            })
            .ok_or("nested original")?
            .source;
        let mut engine = Engine::default();
        if associated_first {
            let row = engine.associated_template(&report, &mut facts, &initializer)?;
            engine
                .associated
                .get_or_insert_with(Default::default)
                .insert(initializer.definition, row);
        }
        let (template, contract) = engine.build_anonymous(&report, &mut facts, source)?;
        engine.original_component(&report, &mut facts, Original { template, contract })?;
        let key = BodyKey::Declaration(initializer.definition);
        let row = engine
            .component_original(key)
            .ok_or("component initializer")?;
        let counts = (row.contract.obligations.len(), row.contract.proofs.len());
        assert!(counts.0 > 0 || counts.1 > 0);
        let observations = engine.contracts.observations();
        let construction = engine.construction.work();
        for _ in 0..2 {
            let (_, contract) = engine.associated_template(&report, &mut facts, &initializer)?;
            assert_eq!((contract.obligations.len(), contract.proofs.len()), counts);
        }
        assert_eq!(engine.contracts.observations(), observations);
        assert_eq!(engine.construction.work(), construction);
        engine
            .original_components
            .as_mut()
            .ok_or("component")?
            .rows
            .get_mut(&key)
            .ok_or("initializer")?
            .contract
            .source = source.into();
        assert!(
            engine
                .associated_template(&report, &mut facts, &initializer)
                .is_err()
        );
        assert!(engine.roots.is_empty());
        assert_eq!(engine.catalog.bytes, 0);
    }
    Ok(())
}
fn prepare(text: &str) -> Result<(Report, Declarations, crate::resolve::unit::NodeKey)> {
    let report = crate::test_source::resolve(text, true)?;
    let context = super::super::Context::default();
    let facts = crate::sema::declarations::analyze_with_executor(&report, context.executor())?;
    let facts = crate::pipeline::analyze(&report, facts)?.facts;
    let source = facts
        .inline_constants
        .as_deref()
        .ok_or("inline originals")?
        .bodies()
        .find(|i| facts.closures.get(i.parent_body).is_some())
        .ok_or("closure inline original")?
        .source;
    Ok((report, facts, source))
}
#[test]
fn sibling_ports_reuse_one_proved_original_component() -> Result {
    for width in [1, 8, 32, 128] {
        let ports = (1..width)
            .map(|_| "let _=const{S::<'a,Tag>::C};")
            .collect::<String>();
        let text = fixture()?.replace("let f=||{const{", &format!("let f=||{{{ports}const{{"));
        let (report, mut facts, source) = prepare(&text)?;
        let mut engine = Engine::default();
        engine.retain_anonymous(&report, &mut facts, source)?;
        let retained = engine
            .original_components
            .as_ref()
            .ok_or("component retention")?;
        assert_eq!(retained.members.len(), width + 2);
        assert_eq!(retained.rows.len(), 2);
        assert_eq!(engine.anonymous.len(), width);
        let work = (
            facts.body_work.canonical_steps,
            engine.construction.work(),
            engine.contracts.observations(),
            facts.constant_work.template_bytes,
        );
        let sources = engine.anonymous.keys().copied().collect::<Vec<_>>();
        for _ in 0..3 {
            for &source in &sources {
                engine.retain_anonymous(&report, &mut facts, source)?;
            }
        }
        assert_eq!(
            work,
            (
                facts.body_work.canonical_steps,
                engine.construction.work(),
                engine.contracts.observations(),
                facts.constant_work.template_bytes
            )
        );
        assert!(engine.roots.is_empty());
        assert_eq!(engine.catalog.bytes, 0);
        eprintln!(
            "ports={width} constructions={:?} charged-canonical={} retained={}",
            work.1, work.0, work.3
        );
    }
    Ok(())
}
#[test]
fn unrelated_originals_are_indexed_once_and_never_join_the_component() -> Result {
    let mut previous = None;
    for unrelated in [0_usize, 8, 32, 128] {
        let mut text = fixture()?;
        for n in 0..unrelated {
            use std::fmt::Write;
            write!(text, "fn unrelated{n}(x:u8)->u8{{x}}").map_err(|_| "format")?;
        }
        let (report, mut facts, source) = prepare(&text)?;
        let mut engine = Engine::default();
        let before = facts.body_work.canonical_steps;
        engine.retain_anonymous(&report, &mut facts, source)?;
        let work = facts.body_work.canonical_steps - before;
        let retained = engine
            .original_components
            .as_ref()
            .ok_or("retained component")?;
        assert_eq!(retained.members.len(), 3);
        assert_eq!(engine.construction.work().0, 3);
        if let Some((old_width, old_work)) = previous {
            assert_eq!(work - old_work, 2 * (unrelated - old_width));
        }
        previous = Some((unrelated, work));
        eprintln!("unrelated={unrelated} component-rows=3 charged-canonical={work}");
    }
    Ok(())
}
#[test]
fn failed_original_component_publishes_no_parent_child_or_execution() -> Result {
    let text = fixture()?.replace(" where 'a:'b", "");
    let (report, mut facts, source) = prepare(&text)?;
    let mut engine = Engine::default();
    let before = facts.body_work.canonical_steps;
    let error = engine
        .retain_anonymous(&report, &mut facts, source)
        .err()
        .ok_or("unpaid relation")?;
    assert!(
        error
            .to_string()
            .contains("required lifetime does not outlive its target"),
        "{error}"
    );
    let retained = engine
        .original_components
        .as_ref()
        .ok_or("pure index remains")?;
    assert!(retained.rows.is_empty());
    assert!(retained.members.is_empty());
    assert!(engine.anonymous.is_empty());
    assert!(engine.roots.is_empty());
    assert_eq!(engine.catalog.bytes, 0);
    assert!(facts.body_work.canonical_steps > before);
    Ok(())
}
#[test]
fn original_index_rejects_missing_foreign_stale_and_cyclic_owners() -> Result {
    let (_, mut facts, source) = prepare(&fixture()?)?;
    let mut work = 0;
    assert!(Index::new(&facts, 0, &mut work).is_err());
    assert_eq!(work, 0);
    let index = Index::new(&facts, BYTES, &mut work)?;
    let complete = index.component(&facts, source, &mut work)?;
    assert_eq!(complete.len(), 3);
    let link = facts
        .nested_outputs
        .as_deref()
        .and_then(|r| r.get(&source))
        .cloned()
        .ok_or("link")?;
    let main_source = facts
        .functions
        .iter()
        .filter_map(|f| f.body)
        .find(|s| {
            !complete
                .iter()
                .any(|&i| index.node(i).is_ok_and(|n| n.source == *s))
        })
        .ok_or("foreign source")?;
    for parent in [main_source, facts.functions[0].node] {
        let mut registry = crate::sema::nested_outputs::Registry::default();
        for &member in &complete {
            let node = index.node(member)?;
            if let Some(link) = &node.link {
                registry.insert(
                    node.source,
                    copy_link(
                        link,
                        if node.source == source {
                            parent.into()
                        } else {
                            link.parent
                        },
                    ),
                )?;
            }
        }
        facts.nested_outputs = Some(Box::new(registry));
        assert!(index.component(&facts, source, &mut work).is_err());
        let changed_index = Index::new(&facts, BYTES, &mut work)?;
        assert!(changed_index.component(&facts, source, &mut work).is_err());
    }
    let mut cyclic = crate::sema::nested_outputs::Registry::default();
    for &member in &complete {
        let node = index.node(member)?;
        if let Some(original) = &node.link {
            cyclic.insert(
                node.source,
                copy_link(
                    original,
                    if crate::body::ir::BodyOrigin::from(node.source) == link.parent {
                        source.into()
                    } else {
                        original.parent
                    },
                ),
            )?;
        }
    }
    facts.nested_outputs = Some(Box::new(cyclic));
    assert!(
        Index::new(&facts, BYTES, &mut work)?
            .component(&facts, source, &mut work)
            .is_err()
    );
    let mut restored = crate::sema::nested_outputs::Registry::default();
    for &member in &complete {
        let node = index.node(member)?;
        if let Some(link) = &node.link {
            restored.insert(node.source, copy_link(link, link.parent))?;
        }
    }
    facts.nested_outputs = Some(Box::new(restored));
    let index = Index::new(&facts, BYTES, &mut work)?;
    let function = complete
        .iter()
        .filter_map(|&i| index.node(i).ok())
        .find_map(|n| match n.owner {
            Owner::Function(id) => Some(id),
            _ => None,
        })
        .ok_or("parent function")?;
    let slot = *facts
        .function_indices
        .get(&function)
        .ok_or("function slot")?;
    facts.functions[slot].signature = facts.functions[0].signature;
    assert!(index.component(&facts, source, &mut work).is_err());
    Ok(())
}
fn copy_link(
    link: &crate::sema::nested_outputs::Link,
    parent: impl Into<crate::body::ir::BodyOrigin>,
) -> crate::sema::nested_outputs::Link {
    crate::sema::nested_outputs::Link {
        source: link.source,
        parent: parent.into(),
        revision: link.revision,
        universals: link.universals,
        bridges: link.bridges.clone(),
        formation: link
            .formation
            .as_ref()
            .map(|f| crate::sema::nested_outputs::Formation {
                identity: f.identity,
                inputs: f.inputs.clone(),
            }),
    }
}

#[test]
fn ordinary_execution_keeps_the_component_catalog_absent() {
    let engine = Engine::default();
    assert!(engine.original_components.is_none());
    assert_eq!(engine.anonymous.capacity(), 0);
    assert_eq!(engine.originals.capacity(), 0);
    let prepared = crate::pipeline::Prepared::default();
    assert!(prepared.original_contexts.is_none());
    eprintln!(
        "layout Engine={} pipelinePrepared={} bodyPrepared={} Checked={} Target={} Request={} Code={} Body={} Value={}",
        size_of::<Engine>(),
        size_of::<crate::pipeline::Prepared>(),
        size_of::<body::templates::Prepared>(),
        size_of::<crate::borrow::templates::Checked<'_>>(),
        size_of::<crate::sema::values::execution::Target>(),
        size_of::<crate::sema::values::execution::Request>(),
        size_of::<crate::const_eval::code::Code>(),
        size_of::<body::ir::Body>(),
        size_of::<body::ir::Value>()
    );
    assert_eq!(size_of::<crate::sema::values::execution::Target>(), 28);
    assert_eq!(size_of::<crate::sema::values::execution::Request>(), 56);
    // The two Fx tables omit the former randomized 16-byte hash states.
    assert_eq!(size_of::<crate::const_eval::code::Code>(), 304);
    assert_eq!(size_of::<body::ir::Value>(), 24);
}
