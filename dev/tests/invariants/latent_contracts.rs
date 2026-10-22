//! Latency adds one bounded registry payload and no per-expression contract work.
use super::*;
#[test]
fn comparison_registry_keeps_ordinary_owner_size_and_bounds_reserved_storage() {
    let old = std::mem::size_of::<[Option<Equality>; 4]>();
    let current = std::mem::size_of::<Comparisons>();
    assert_eq!(std::mem::size_of::<Traits>(), 192);
    assert!(current <= old + 24);
    eprintln!(
        "comparison storage: Traits={} old_registry={old} registry={current}",
        std::mem::size_of::<Traits>()
    );
}
#[test]
fn repeated_scalar_operators_reserve_fixed_identity_without_semantic_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    let mut count = None;
    for width in [8, 32, 128] {
        let mut source = String::from("fn main(){");
        for _ in 0..width {
            source.push_str("let _=1u8==2u8;");
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let definitions = report.resolution.definitions.len();
        if let Some(count) = count {
            assert_eq!(definitions, count);
        }
        count = Some(definitions);
        let declaration = report
            .resolution
            .builtin_traits
            .comparison(Comparison::PartialEq)
            .ok_or("identity")?;
        let analysis = crate::pipeline::analyze_source(&report)?;
        assert!(
            analysis
                .facts
                .associated
                .comparison(declaration.definition)
                .is_none()
        );
        assert_eq!(analysis.facts.associated.language_functions().count(), 0);
        assert_eq!(analysis.facts.associated.library_defaults().count(), 0);
        assert!(analysis.facts.primitive_methods.is_none());
    }
    Ok(())
}
