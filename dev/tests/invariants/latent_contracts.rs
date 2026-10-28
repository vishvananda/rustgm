//! Latency adds one bounded registry payload and no per-expression contract work.
use super::*;
#[test]
fn comparison_registry_keeps_ordinary_owner_size_and_bounds_reserved_storage() {
    let old = std::mem::size_of::<[Option<Equality>; 4]>();
    let current = std::mem::size_of::<Comparisons>();
    // Language assignment demands move the existing map behind a cold optional owner.
    assert_eq!(std::mem::size_of::<Traits>(), 168);
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
#[test]
fn scalar_ordering_keeps_result_nominals_and_all_comparison_contracts_latent()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::support::library_types::LibraryType;
    let mut count = None;
    for width in [8, 32, 128] {
        let mut source = String::from("fn main(){");
        for _ in 0..width {
            source.push_str("let _=1u8<2u8;");
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let definitions = report.resolution.definitions.len();
        if let Some(count) = count {
            assert_eq!(definitions, count);
        }
        count = Some(definitions);
        let analysis = crate::pipeline::analyze_source(&report)?;
        for kind in Comparison::ALL {
            let declaration = report
                .resolution
                .builtin_traits
                .comparison(kind)
                .ok_or("identity")?;
            assert!(report.resolution.builtin_traits.comparison_latent(kind));
            assert!(
                analysis
                    .facts
                    .associated
                    .comparison(declaration.definition)
                    .is_none()
            );
        }
        for kind in [LibraryType::ComparisonOrdering, LibraryType::Option] {
            let declaration = report
                .resolution
                .library_types
                .get(kind)
                .ok_or("result identity")?;
            assert!(
                report
                    .resolution
                    .builtin_traits
                    .comparison_result_latent(kind)
            );
            assert!(analysis.facts.definitions[declaration.definition.0 as usize].is_none());
            assert!(
                analysis
                    .facts
                    .aggregates
                    .get(crate::types::ItemId(declaration.definition.0))
                    .is_none()
            );
        }
        assert_eq!(analysis.facts.associated.language_functions().count(), 0);
        assert_eq!(analysis.facts.associated.library_defaults().count(), 0);
        assert!(analysis.facts.primitive_methods.is_none());
        eprintln!("scalar ordering width={width} definitions={definitions}");
    }
    Ok(())
}

fn source_comparison_prefix() -> Result<String, Box<dyn std::error::Error>> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )?;
    let (foundation, _) = source
        .split_once("pub fn identity<T>")
        .ok_or("foundation")?;
    let destruct =
        crate::test_source::fixture("pa31/tests/spec/source-location/neg-destruct-declaration.rs")?;
    Ok(format!(
        "#![feature(staged_api,const_trait_impl)]#![unstable(feature=\"comparison_fixture\",issue=\"none\")]\n{foundation}\n{destruct}\nimpl Clone for bool{{fn clone(&self)->Self{{*self}}}}impl Copy for bool{{}}\n#[lang=\"not\"]#[rustc_const_unstable(feature=\"const_ops\",issue=\"143802\")]pub const trait Not{{type Output;fn not(self)->Self::Output;}}#[rustc_const_unstable(feature=\"const_ops\",issue=\"143802\")]const impl Not for bool{{type Output=bool;fn not(self)->bool{{!self}}}}"
    ))
}

#[test]
fn actual_source_partial_eq_default_body_keeps_its_real_call_without_bootstrap_preparation()
-> Result<(), Box<dyn std::error::Error>> {
    let actual = crate::test_source::fixture(
        "pa31/tests/spec/source-location/partial-eq-original-declaration.rs",
    )?;
    let report = crate::test_source::source_provider(
        &(source_comparison_prefix()? + &actual),
        "actual_partial_eq_default",
    )?;
    assert!(report.resolution.builtin_traits.comparisons.is_none());
    assert!(report.resolution.builtin_traits.library.is_none());
    assert!(
        report
            .resolution
            .method_calls
            .values()
            .any(|method| { report.package.names.get(method.name) == "eq" })
    );
    let actual_owner = report
        .resolution
        .source_bindings
        .as_deref()
        .and_then(|rows| rows.language(crate::support::bindings::LangItem::PartialEq))
        .ok_or("actual PartialEq language declaration")?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let ne = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, row)| report.package.names.get(row.name) == "ne")
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?
        .ok_or("actual ne body")?;
    assert!(analysis.facts.typed_function_bodies.contains(&ne));
    assert_ne!(actual_owner.definition, ne);
    for malformed in [
        actual.replace("#[lang = \"eq\"]", ""),
        String::from("#[lang=\"eq\"]struct Wrong;"),
        String::from("#[rustc_diagnostic_item=\"cmp_partialeq_eq\"]fn wrong(){}"),
    ] {
        assert!(
            crate::test_source::source_provider(
                &(source_comparison_prefix()? + &malformed),
                "wrong_source_comparison"
            )
            .is_err()
        );
    }
    Ok(())
}

#[test]
fn unrelated_source_eq_methods_still_require_their_actual_trait_and_implementation()
-> Result<(), Box<dyn std::error::Error>> {
    let header = "trait UserEq{fn eq(&self,other:&Self)->bool;}struct Item;";
    let implementation = "impl UserEq for Item{fn eq(&self,_:&Self)->bool{true}}";
    let use_method = "pub fn probe(x:&Item,y:&Item)->bool{x.eq(y)}";
    let report = crate::test_source::source_provider(
        &(source_comparison_prefix()? + header + implementation + use_method),
        "unrelated_source_eq",
    )?;
    assert!(report.resolution.builtin_traits.comparisons.is_none());
    crate::pipeline::analyze_source(&report)?;
    for malformed in [
        format!("{header}{use_method}"),
        format!(
            "{header}struct Other;impl UserEq for Other{{fn eq(&self,_:&Self)->bool{{true}}}}{use_method}"
        ),
    ] {
        let report = crate::test_source::source_provider(
            &(source_comparison_prefix()? + &malformed),
            "missing_source_eq_impl",
        )?;
        assert!(
            crate::pipeline::analyze_source(&report).is_err(),
            "missing/wrong actual impl"
        );
        assert!(report.resolution.builtin_traits.comparisons.is_none());
    }
    Ok(())
}

#[test]
fn source_comparisons_reject_missing_language_headers_and_implementations()
-> Result<(), Box<dyn std::error::Error>> {
    let actual = crate::test_source::fixture(
        "pa31/tests/spec/source-location/partial-eq-original-declaration.rs",
    )?;
    for (declaration, equality_header) in [
        ("", false),
        (actual.as_str(), true),
        ("trait UserEq{fn eq(&self,other:&Self)->bool;}", false),
    ] {
        for operator in ["==", "<"] {
            let input = format!(
                "{}{declaration}pub fn probe(x:u8,y:u8)->bool{{x {operator} y}}",
                source_comparison_prefix()?
            );
            let report = crate::test_source::source_provider(&input, "closed_source_comparison")?;
            let error = crate::pipeline::analyze_source(&report)
                .err()
                .ok_or("source scalar comparison acquired unproved execution")?;
            let expected = if equality_header && operator == "==" {
                "can't compare"
            } else {
                "source comparison requires its actual language declaration"
            };
            assert!(
                error.to_string().contains(expected),
                "operator {operator}: {error}"
            );
            assert!(report.resolution.builtin_traits.comparisons.is_none());
        }
    }
    Ok(())
}
