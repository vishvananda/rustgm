//! Parent request aliases require exact equality and lifetime ownership.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn reference(store: &mut crate::types::Store, region: RegionId) -> Result<TypeId> {
    Ok(store.intern(crate::types::Kind::Reference {
        mutable: false,
        region,
        referent: TypeId::primitive(crate::types::Primitive::U8),
    })?)
}
#[test]
fn selected_arguments_are_inserted_only_after_owned_equality_restoration() -> Result {
    let local = RegionId(7);
    let external = RegionId(41);
    let owned = HashSet::from([local]);
    let actuals = HashSet::from([external]);
    let mut equations = RegionEquations::default();
    let mut work = 0;
    assert_eq!(
        replacement(
            RegionId::bound(0, 0)?,
            &actuals,
            &owned,
            &[external],
            &mut equations,
            &mut work
        )?,
        external
    );
    assert!(
        replacement(
            RegionId::bound(0, 1)?,
            &actuals,
            &owned,
            &[external],
            &mut equations,
            &mut work
        )
        .is_err()
    );
    equations.extend_counted([(local, RegionId::bound(0, 0)?)], &mut work)?;
    assert_eq!(
        replacement(
            local,
            &actuals,
            &owned,
            &[external],
            &mut equations,
            &mut work
        )?,
        external
    );
    assert!(
        replacement(
            local,
            &actuals,
            &HashSet::new(),
            &[external],
            &mut equations,
            &mut work
        )
        .is_err()
    );
    assert!(
        replacement(
            RegionId(8),
            &actuals,
            &owned,
            &[external],
            &mut equations,
            &mut work
        )
        .is_err()
    );
    assert!(replacement(local, &actuals, &owned, &[], &mut equations, &mut work).is_err());
    // A root with the same numeric private and actual identity has no authority
    // to choose between those namespaces. A different source alias can map TO
    // that actual identity because replacement is simultaneous.
    assert!(
        replacement(
            local,
            &HashSet::from([local]),
            &owned,
            &[local],
            &mut equations,
            &mut work
        )
        .is_err()
    );
    assert_eq!(
        replacement(
            local,
            &HashSet::from([external]),
            &owned,
            &[external],
            &mut equations,
            &mut work
        )?,
        external
    );
    Ok(())
}
#[test]
fn matching_shape_or_outlives_cannot_replace_a_missing_parent_equality() -> Result {
    let owned = HashSet::from([RegionId(7), RegionId(8)]);
    let external = HashSet::from([RegionId(41)]);
    let mut equations = RegionEquations::default();
    let mut work = 0;
    equations.extend_counted([(RegionId(7), RegionId(8))], &mut work)?;
    assert!(
        replacement(
            RegionId(7),
            &external,
            &owned,
            &[RegionId(41)],
            &mut equations,
            &mut work
        )
        .is_err()
    );
    assert_eq!(
        replacement(
            RegionId(41),
            &external,
            &owned,
            &[RegionId(41)],
            &mut equations,
            &mut work
        )?,
        RegionId(41)
    );
    assert!(work > 0);
    Ok(())
}
#[test]
fn request_restoration_preserves_nested_public_binders_and_rejects_unknown_roots() -> Result {
    use crate::types::{Kind, Store};
    let mut store = Store::default();
    let local = reference(&mut store, RegionId::bound(0, 0)?)?;
    let result = reference(&mut store, RegionId(7))?;
    let parameters = store.intern_list(&[local])?;
    let function = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let mut fold =
        Bound::restore_budgeted_regions(HashMap::from([(RegionId(7), RegionId(41))]), WORK, BYTES)?;
    let mapped = fold.apply(&mut store, function)?;
    let Some(Kind::Function {
        parameters: input,
        result: output,
        bound_regions: 1,
        ..
    }) = store.kind(mapped)
    else {
        return Err("missing mapped callable".into());
    };
    assert_eq!(input, parameters);
    assert_eq!(Some(output), Some(reference(&mut store, RegionId(41))?));
    let before = fold.steps;
    assert_eq!(fold.apply(&mut store, function)?, mapped);
    assert!(fold.steps - before <= 2);
    let unknown = reference(&mut store, RegionId(8))?;
    assert!(fold.apply(&mut store, unknown).is_err());
    assert!(fold.steps > before);
    Ok(())
}

#[test]
fn sibling_inline_values_cannot_borrow_anonymous_parent_equations() -> Result {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa25/tests/spec/inline-query-context/siblings.rs");
    let mut report = crate::test_source::resolve(&std::fs::read_to_string(path)?, true)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    crate::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)?;
    let instances = facts
        .inline_constants
        .as_deref()
        .ok_or("inline registry")?
        .instance_values()
        .map(|(instance, _, _)| instance)
        .filter(|instance| matches!(instance.enclosing(), Context::SourceInitializer(_)))
        .collect::<Vec<_>>();
    let parent = *instances.first().ok_or("source inline parent")?;
    let sibling = *instances
        .iter()
        .find(|instance| {
            instance.source() != parent.source() && instance.enclosing() == parent.enclosing()
        })
        .ok_or("same-context sibling")?;
    let Context::SourceInitializer(witness) = parent.enclosing() else {
        return Err("source witness".into());
    };
    let mut body = crate::body::templates::Prepared::default()
        .anonymous(&report, &facts, parent.source())?
        .body;
    body.owner = Owner::AnonymousInstance(parent);
    let id = ValueId(u32::try_from(body.values.len())?);
    body.values.push(crate::body::ir::Value {
        ty: body.result,
        kind: Rvalue::InlineInstance(sibling),
    });
    body.statements.push(crate::body::ir::Statement {
        source: body.source.node,
        action: Action::Evaluate(id),
    });
    let mut work = 0;
    assert!(roots(&facts, &body, witness, &mut work).is_err());
    assert!(work > 0);
    Ok(())
}
