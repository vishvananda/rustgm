//! Real module/header snapshots are structural data, not a published source profile.
use super::*;
type Result = std::result::Result<(), Box<dyn std::error::Error>>;

#[test]
fn actual_nonrole_traits_modules_aliases_and_ordered_names_remain_unpublished() -> Result {
    let root = crate::lower::instances::namespace_fixture("root-trait")?;
    assert_eq!(root.modules().len(), 1);
    assert_eq!(root.columns().roles.len(), 7);
    let open = root
        .columns()
        .declarations
        .iter()
        .enumerate()
        .find(|(_, r)| r.name.as_deref() == Some("Open"))
        .ok_or("actual Open")?
        .0;
    let open = DeclarationRef(u32::try_from(open)?);
    assert!(matches!(root.header(open)?, Some(s::HeaderRef::Trait(_))));
    assert!(
        root.columns()
            .roles
            .iter()
            .all(|role| role.declaration.declaration != open)
    );
    let names = crate::lower::instances::namespace_fixture("nested-aliases")?;
    assert_eq!(names.modules().len(), 4);
    assert_eq!(names.columns().traits.len(), 9);
    assert_eq!(names.columns().roles.len(), 7);
    let mut work = Work::default();
    let outer = names
        .find(names.root(), "outer", Namespace::Type, &mut work)?
        .ok_or("outer")?;
    let outer = names.bindings()[outer].target.declaration;
    let visible = names
        .find(outer, "Visible", Namespace::Type, &mut work)?
        .ok_or("public alias")?;
    let visible = names.bindings()[visible].target.declaration;
    let parent = names.columns().declarations[visible.0 as usize]
        .definition
        .parent
        .ok_or("defining module")?;
    assert_eq!(
        names.columns().declarations[parent.0 as usize]
            .name
            .as_deref(),
        Some("hidden")
    );
    assert!(
        names
            .find(outer, "hidden", Namespace::Type, &mut work)?
            .is_none()
    );
    check_invalid_columns(&names, visible);
    let mut failed = Work {
        steps: crates::MAX_COMPONENTS,
        ..Work::default()
    };
    assert!(
        names
            .find(outer, "Visible", Namespace::Type, &mut failed)
            .is_err()
    );
    assert_eq!(failed.steps, crates::MAX_COMPONENTS + 1);
    assert!(
        names
            .find(outer, "Visible", Namespace::Type, &mut Work::default())?
            .is_some()
    );
    check_growth()?;
    println!(
        "namespace layouts raw={} binding={} frozen={} stats={} Columns={} SourceRows={} Package={} Report={}",
        size_of::<NamespaceColumns>(),
        size_of::<ExportBinding>(),
        size_of::<UnpublishedNamespace>(),
        size_of::<NamespaceWork>(),
        size_of::<Columns>(),
        size_of::<SourceRows>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<crate::resolve::expansion::Report>()
    );
    println!(
        "namespace owned layout pending={} frozen={}",
        size_of::<crate::resolve::crates::PendingNamespace>(),
        size_of::<crate::resolve::crates::OwnedNamespaceCatalog>()
    );
    Ok(())
}

fn check_invalid_columns(names: &UnpublishedNamespace, visible: DeclarationRef) {
    let declarations = &names.columns().declarations;
    for changed in [
        {
            let mut n = names.names.clone();
            n.modules.pop();
            n
        },
        {
            let mut n = names.names.clone();
            n.bindings[0].target.artifact = s::ArtifactRef(1);
            n
        },
        {
            let mut n = names.names.clone();
            n.bindings[0].target.declaration = DeclarationRef(u32::MAX);
            n
        },
        {
            let mut n = names.names.clone();
            n.modules[0] = visible;
            n
        },
    ] {
        assert!(
            changed
                .validate(declarations, &mut Work::default())
                .is_err()
        );
    }
    let mut duplicate = names.names.clone();
    duplicate.bindings.push(duplicate.bindings[0].clone());
    assert!(
        duplicate
            .sort(&mut Work::default(), &mut NamespaceWork::default())
            .is_err()
    );
}

fn check_growth() -> Result {
    let mut baseline = None;
    for width in [8_usize, 32, 128] {
        let mut source = crate::test_source::fixture(
            "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
        )?;
        source.push_str("pub mod owner {pub trait T{}}\n");
        for i in 0..width {
            use std::fmt::Write;
            writeln!(&mut source, "pub use owner::T as Alias_{i};")?;
        }
        let report = crate::test_source::source_provider(&source, "namespace_growth")?;
        let identity = report
            .package
            .source_core()
            .ok_or("provider")?
            .provider
            .clone();
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(&report)?;
        let snapshot = crate::lower::instances::inspect_source_namespace(
            &report, &mut facts, &identity, prepared,
        )?;
        assert_eq!(snapshot.modules().len(), 2);
        assert_eq!(snapshot.bindings().len(), 9 + width);
        if let Some((declarations, retained)) = baseline {
            assert_eq!(snapshot.columns().declarations.len(), declarations);
            assert!(snapshot.work().retained_bytes <= retained + (width - 8) * 256 + 2048);
        } else {
            baseline = Some((
                snapshot.columns().declarations.len(),
                snapshot.work().retained_bytes,
            ));
        }
        assert!(snapshot.work().comparisons < (width + 16) * 16);
        assert!(snapshot.work().compared_bytes < (width + 16) * 256);
        println!("namespace width={width} {:?}", snapshot.work());
    }
    Ok(())
}
