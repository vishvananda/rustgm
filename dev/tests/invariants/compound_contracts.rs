//! Real typed demands activate one canonical owner without changing lexical visibility.
use super::*;
use crate::{
    pipeline,
    types::{Kind, TypeId},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn analyze(source: &str) -> TestResult<(crate::resolve::expansion::Report, pipeline::Analysis)> {
    let report = crate::test_source::resolve(source, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
#[test]
fn actual_reference_forms_and_shadows_lower_through_selected_assignment_calls() -> TestResult {
    for name in [
        "bound-reference",
        "returned-reference",
        "lexical-shadow",
        "module-shadow",
        "aliased-scalars",
        "call-order",
        "boolean-forward",
        "all-families",
        "imported-dot",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa26/tests/spec/implicit-assignment-demand/{name}.rs"
        ))?;
        let (
            mut report,
            pipeline::Analysis {
                mut facts,
                prepared,
            },
        ) = analyze(&source)?;
        let entry = crate::sema::entry::select(&mut report, &facts)?;
        let program =
            crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        assert!(!program.symbols.is_empty(), "{name}");
        assert!(
            facts
                .primitive_methods
                .as_ref()
                .is_some_and(|methods| !methods.is_empty()),
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn implicit_operator_demand_never_imports_a_dot_method_or_weakens_operand_proof() -> TestResult {
    for name in [
        "dot-before-demand",
        "dot-after-demand",
        "mutable-rhs",
        "mixed-width",
        "boolean-add",
    ] {
        let source = crate::test_source::fixture(&format!(
            "pa26/tests/spec/implicit-assignment-demand/{name}.rs"
        ))?;
        let report = crate::test_source::resolve(&source, true)?;
        assert!(pipeline::analyze_source(&report).is_err(), "{name}");
    }
    Ok(())
}
#[test]
fn scalar_and_disabled_paths_leave_semantic_headers_and_factories_cold() -> TestResult {
    for source in [
        "fn main(){}",
        "#[cfg(any())]fn absent(){let mut x=1u32;x+=&2u32;}fn main(){}",
    ] {
        let (report, facts) = analyze(source)?;
        assert!(report.resolution.builtin_traits.language.is_none());
        assert!(facts.facts.primitive_methods.is_none());
    }
    let (report, analysis) = analyze("fn main(){let mut x=1u32;x+=2u32;}")?;
    let declaration = report
        .resolution
        .builtin_traits
        .language(Hook::AddAssign)
        .ok_or("reserved identity")?;
    assert!(
        report
            .resolution
            .builtin_traits
            .language_latent(Hook::AddAssign)
    );
    assert!(analysis.facts.definitions[declaration.method.0 as usize].is_none());
    assert!(
        !analysis
            .facts
            .generics
            .parameters
            .contains_key(&declaration.definition)
    );
    assert_eq!(analysis.facts.associated.language_functions().count(), 0);
    assert!(analysis.facts.primitive_methods.is_none());
    Ok(())
}
#[test]
fn repeated_uses_retain_one_family_and_one_semantic_header_with_fixed_global_cost() -> TestResult {
    let mut declarations = None;
    for width in [1, 8, 32] {
        let source = format!(
            "const Y:u32=4;fn main(){{let mut x=3u32;{}}}",
            "x+=&Y;".repeat(width)
        );
        let (report, analysis) = analyze(&source)?;
        let registry = report
            .resolution
            .builtin_traits
            .language
            .as_deref()
            .ok_or("cold registry")?;
        assert_eq!(registry.entries.len(), 1);
        let count = report.resolution.definitions.len();
        assert_eq!(*declarations.get_or_insert(count), count);
        let declaration = registry
            .entries
            .get(&Hook::AddAssign)
            .ok_or("actual family")?;
        let ty =
            analysis.facts.definitions[declaration.method.0 as usize].ok_or("actual callable")?;
        let Some(Kind::FunctionItem { signature, .. }) = analysis.facts.types.kind(ty) else {
            return Err("actual function item".into());
        };
        assert!(matches!(
            analysis.facts.types.kind(signature),
            Some(Kind::Function {
                result: TypeId::UNIT,
                bound_regions: 1,
                ..
            })
        ));
        assert_eq!(analysis.facts.associated.language_functions().count(), 1);
        println!(
            "assignment width={width} definitions={count} registry={} capacity={} Traits={} Language={} Resolution={} Body={}",
            size_of::<super::super::Registry>(),
            registry.entries.capacity(),
            size_of::<Traits>(),
            size_of::<Language>(),
            size_of::<crate::resolve::model::Resolution>(),
            size_of::<crate::body::ir::Body>()
        );
        assert_eq!(size_of::<Traits>(), 168);
        assert_eq!(size_of::<Language>(), 44);
        assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2032);
        assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    }
    Ok(())
}
