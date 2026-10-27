//! Failed module paths use canonical primitive prefixes, not new call permission.
use crate::{
    pipeline,
    resolve::{
        expansion::Report,
        model::{DefKind, Namespace, Target},
    },
    test_source,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn input(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!(
        "pa6/tests/spec/primitive-members/{name}.rs"
    ))?)
}
fn member_target(report: &Report) -> Result<Target> {
    report
        .resolution
        .references
        .iter()
        .filter(|reference| reference.namespace == Namespace::Value)
        .find_map(|reference| {
            let unit = report.package.unit(reference.node.file);
            let span = unit.configured.ast.node(reference.node.node).span;
            (unit
                .source
                .text()
                .get(span.start as usize..span.end as usize)
                == Some("u8::build"))
            .then_some(reference.target)
        })
        .ok_or_else(|| "actual member path target".into())
}
#[test]
fn module_fallback_keeps_canonical_trait_dispatch_and_real_module_members() -> Result {
    for name in [
        "module-fallback",
        "imported-module-fallback",
        "module-member-wins",
        "nominal-alias-shadow",
        "type-alias-shadow",
    ] {
        let mut report = test_source::resolve(&input(name)?, true)?;
        let target = member_target(&report)?;
        match name {
            "module-fallback" | "imported-module-fallback" => {
                let Target::Associated(owner) = target else {
                    return Err(format!("{name}: not an associated primitive").into());
                };
                let definition = &report.resolution.definitions[owner.0 as usize];
                assert_eq!(definition.kind, DefKind::Primitive);
                assert_eq!(report.package.names.get(definition.name), "u8");
            }
            "module-member-wins" => {
                let Target::Definition(owner) = target else {
                    return Err("actual module member lost".into());
                };
                assert_eq!(
                    report.resolution.definitions[owner.0 as usize].kind,
                    DefKind::Function
                );
            }
            _ => {
                let Target::Associated(owner) = target else {
                    return Err("nominal owner lost".into());
                };
                assert!(matches!(
                    report.resolution.definitions[owner.0 as usize].kind,
                    DefKind::Struct | DefKind::TypeAlias
                ));
            }
        }
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let entry = crate::sema::entry::select(&mut report, &facts)?;
        let program =
            crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        assert!(
            program
                .symbols
                .iter()
                .filter(|symbol| symbol.body.is_some())
                .count()
                >= 2
        );
    }
    Ok(())
}
#[test]
fn private_ambiguous_qualified_and_import_failures_keep_their_actual_boundaries() -> Result {
    for (name, needle) in [
        ("private-member", "private"),
        ("ambiguous-member", "ambiguous"),
        ("qualified-module-missing", "unresolved value"),
        ("nonprimitive-module-missing", "unresolved value"),
        ("import-member-missing", "unresolved import"),
    ] {
        let error = test_source::resolve(&input(name)?, true)
            .err()
            .ok_or_else(|| format!("{name}: invalid member accepted"))?;
        assert!(error.contains(needle), "{name}: {error}");
    }
    for name in [
        "module-primitive-fallback",
        "imported-module-primitive-fallback",
    ] {
        let source = test_source::fixture(&format!("pa6/tests/spec/names/{name}.rs"))?;
        let report = test_source::resolve(&source, true)?;
        pipeline::analyze_source(&report)?;
    }
    Ok(())
}
#[test]
fn repeated_primitive_prefixes_keep_fixed_definitions_and_no_retained_fallback_state() -> Result {
    let mut base = None;
    for width in [8, 32, 128] {
        let source = format!(
            "mod u8{{}}trait Build{{fn build()->u8;}}impl Build for u8{{fn build()->u8{{7}}}}fn main(){{{}}}",
            "let _=u8::build();".repeat(width)
        );
        let report = test_source::resolve(&source, true)?;
        let rows = &report.resolution;
        let fixed = (
            rows.definitions.len(),
            rows.scopes.len(),
            rows.primitives.len(),
        );
        assert_eq!(*base.get_or_insert(fixed), fixed);
        assert_eq!(rows.associated_paths.len(), width);
        assert_eq!(
            rows.references
                .iter()
                .filter(|r| matches!(r.target, Target::Associated(_)))
                .count(),
            width
        );
        let analysis = pipeline::analyze_source(&report)?;
        eprintln!(
            "primitive member width={width} fixed={fixed:?} refs={} inference={}",
            rows.references.len(),
            analysis.facts.body_work.inference_steps
        );
    }
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2056);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}
