//! Genuine moved-report/catalog transactions preserve identity and all public guards.
use super::*;
use crate::{resolve::model::DefKind, support::crates::Namespace as WireNamespace};
type Result = std::result::Result<(), Box<dyn std::error::Error>>;
fn client() -> std::result::Result<Report, String> {
    crate::test_source::resolve("fn current(){}", true)
}

#[test]
fn owned_module_catalog_authenticates_coordinates_warm_cells_and_failed_transactions() -> Result {
    let snapshot = crate::lower::instances::namespace_fixture("nested-aliases")?;
    let mut work = Work::default();
    let mut owned = PendingNamespace::prepare(client()?, Arc::clone(&snapshot), &mut work)?
        .freeze(&mut work)?;
    assert_eq!(owned.report.package.modules.len(), 1);
    assert!(owned.report.package.source_client().is_none());
    assert!(owned.report.resolution.crates.is_none());
    let root = owned.root()?;
    let mut queries = QueryWork::default();
    let outer = owned
        .lookup(root, "outer", WireNamespace::Type, &mut queries)?
        .ok_or("outer")?;
    let visible = owned
        .lookup(outer, "Visible", WireNamespace::Type, &mut queries)?
        .ok_or("visible")?;
    let row = owned.coordinate(visible)?;
    let parent = snapshot.columns().declarations[row.0 as usize]
        .definition
        .parent
        .ok_or("canonical module")?;
    assert_eq!(
        snapshot.columns().declarations[parent.0 as usize]
            .name
            .as_deref(),
        Some("hidden")
    );
    assert!(
        owned
            .lookup(outer, "hidden", WireNamespace::Type, &mut queries)?
            .is_none()
    );
    let private_module = owned.declaration(parent, DefKind::Module)?;
    assert_eq!(owned.coordinate(private_module)?, parent);
    let root_definition = owned.catalog.declaration_identity(root, DefKind::Module)?;
    assert!(
        crate::resolve::impl_restrictions::module(
            &owned.report.package,
            &owned.report.resolution,
            root_definition
        )
        .is_err()
    );
    let foreign =
        PendingNamespace::prepare(client()?, Arc::clone(&snapshot), &mut Work::default())?
            .freeze(&mut Work::default())?;
    assert!(
        owned
            .lookup(foreign.root()?, "outer", WireNamespace::Type, &mut queries)
            .is_err()
    );
    let other = crate::lower::instances::namespace_fixture("nested-aliases")?;
    let before = queries.steps;
    assert!(owned.require(&other, &mut queries).is_err());
    assert_eq!(queries.steps, before + 1);
    assert!(owned.declaration(row, DefKind::Function).is_err());
    for _ in 0..16 {
        assert_eq!(
            owned.lookup(outer, "Visible", WireNamespace::Type, &mut queries)?,
            Some(visible)
        );
    }
    let actual = owned
        .catalog
        .declaration_identity(visible, DefKind::Trait)?;
    let old = owned.report.resolution.definitions[actual.0 as usize].members;
    owned.report.resolution.definitions[actual.0 as usize].members = None;
    assert!(
        owned
            .lookup(outer, "Visible", WireNamespace::Type, &mut queries)
            .is_err()
    );
    owned.report.resolution.definitions[actual.0 as usize].members = old;
    assert!(
        owned
            .lookup(outer, "Visible", WireNamespace::Type, &mut queries)?
            .is_some()
    );
    let mut exhausted = Work {
        steps: MAX_COMPONENTS,
        ..Work::default()
    };
    assert!(PendingNamespace::prepare(client()?, Arc::clone(&snapshot), &mut exhausted).is_err());
    assert_eq!(exhausted.steps, MAX_COMPONENTS + 1);
    check_homonyms()?;
    check_closed_profiles()?;
    check_nonzero_origin()?;
    println!(
        "namespace catalog pending={} owned={} localModules={} steps={} retained={} querySteps={}",
        size_of::<PendingNamespace>(),
        size_of::<OwnedNamespaceCatalog>(),
        owned.report.package.modules.len(),
        work.steps,
        work.retained,
        queries.steps
    );
    Ok(())
}

fn check_homonyms() -> Result {
    let same = crate::lower::instances::namespace_fixture("homonyms")?;
    let mut w = Work::default();
    let names = PendingNamespace::prepare(client()?, same, &mut w)?.freeze(&mut w)?;
    let owner = names
        .lookup(
            names.root()?,
            "owner",
            WireNamespace::Type,
            &mut QueryWork::default(),
        )?
        .ok_or("owner")?;
    assert_ne!(
        names.lookup(
            owner,
            "Same",
            WireNamespace::Type,
            &mut QueryWork::default()
        )?,
        names.lookup(
            owner,
            "Same",
            WireNamespace::Value,
            &mut QueryWork::default()
        )?
    );
    Ok(())
}

fn check_closed_profiles() -> Result {
    assert!(crate::lower::instances::namespace_fixture("restricted-stays-closed").is_err());
    for fixture in ["root-trait", "nested-aliases", "restricted-stays-closed"] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/private-module-namespace/{fixture}.rs"
        ))?;
        let report = crate::test_source::source_provider(&source, "closed_namespace")?;
        let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
        let identity = report
            .package
            .source_core()
            .ok_or("provider")?
            .provider
            .clone();
        assert!(
            crate::sema::crates::authored_export::Stage::begin(&report, &mut facts, &identity)
                .is_err()
        );
    }
    Ok(())
}

fn check_nonzero_origin() -> Result {
    let data = crate::lower::instances::namespace_fixture("nonzero-names")?;
    let mut work = Work::default();
    let owned =
        PendingNamespace::prepare(client()?, Arc::clone(&data), &mut work)?.freeze(&mut work)?;
    let num = owned
        .lookup(
            owned.root()?,
            "num",
            WireNamespace::Type,
            &mut QueryWork::default(),
        )?
        .ok_or("num")?;
    let item = owned
        .lookup(
            num,
            "ZeroablePrimitive",
            WireNamespace::Type,
            &mut QueryWork::default(),
        )?
        .ok_or("alias")?;
    let row = owned.coordinate(item)?;
    let parent = data.columns().declarations[row.0 as usize]
        .definition
        .parent
        .ok_or("actual private defining module")?;
    assert_eq!(
        data.columns().declarations[parent.0 as usize]
            .name
            .as_deref(),
        Some("nonzero")
    );
    assert!(
        owned
            .lookup(
                num,
                "nonzero",
                WireNamespace::Type,
                &mut QueryWork::default()
            )?
            .is_none()
    );
    assert_eq!(
        owned.coordinate(owned.declaration(parent, DefKind::Module)?)?,
        parent
    );
    Ok(())
}
