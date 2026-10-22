//! C objects retain canonical classes across imports, exports and callbacks.
use crate::foreign_frontend as frontend;
#[cfg(feature = "object-interop-tests")]
use crate::host_objects as objects;
use std::path::Path;
#[test]
fn aggregate_profile_has_explicit_language_and_implementation_boundaries()
-> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for name in [
        "interop",
        "aligned",
        "indirect",
        "rust-repr",
        "packed",
        "union",
        "enum",
        "empty",
        "nested",
        "array",
        "unsafe-call",
        "const-call",
        "wrong-type",
    ] {
        let source =
            std::fs::read_to_string(root.join(format!("pa30/tests/spec/aggregate-abi/{name}.rs")))?;
        let result = frontend::checked(&source);
        assert_eq!(
            result.is_ok(),
            matches!(name, "interop" | "aligned" | "indirect"),
            "{name}: {:?}",
            result.as_ref().err()
        );
        if matches!(
            name,
            "rust-repr" | "packed" | "union" | "enum" | "empty" | "nested" | "array"
        ) {
            let diagnostic = result.err().ok_or("missing profile boundary")?;
            assert!(
                diagnostic.contains("C aggregate ABI profile"),
                "{name}: {diagnostic}"
            );
        }
    }
    Ok(())
}
#[test]
#[cfg(feature = "object-interop-tests")]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn aggregate_calls_match_independent_rust_and_c() -> Result<(), Box<dyn std::error::Error>> {
    objects::check_cases(
        "aggregate-abi",
        "pa30/tests/spec/aggregate-abi/interop.c",
        &[("interop", "pa30/tests/spec/aggregate-abi/interop.rs")],
    )?;
    objects::check_cases(
        "aggregate-abi/aligned",
        "pa30/tests/spec/aggregate-abi/aligned.c",
        &[("aligned", "pa30/tests/spec/aggregate-abi/aligned.rs")],
    )
}

#[test]
fn classification_reuses_exact_plans_without_rewalking_fields()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        abi::classification::Plans,
        types::{Kind, callable::Abi},
    };
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let source = std::fs::read_to_string(root.join("pa30/tests/spec/aggregate-abi/interop.rs"))?;
    let (_, analysis) = frontend::analyze(&source)?;
    let mut facts = analysis.facts;
    let signatures:Vec<_>=facts.functions.iter().map(|f|f.signature).filter(|&ty|matches!(facts.types.kind(ty),Some(Kind::Function {convention,..}) if convention.abi==Abi::C)).collect();
    assert!(signatures.len() > 10);
    let mut plans = Plans::default();
    let mut originals = Vec::new();
    for &signature in &signatures {
        originals.push(plans.demand(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            signature,
        )?);
    }
    assert!(plans.steps > signatures.len());
    assert!(plans.retained_bytes > 0);
    let start = plans.steps;
    let retained = plans.retained_bytes;
    for _ in 0..128 {
        for (&signature, original) in signatures.iter().zip(&originals) {
            let selected = plans.demand(
                &facts.types,
                &facts.aggregates,
                &mut facts.layouts,
                signature,
            )?;
            assert!(std::sync::Arc::ptr_eq(original, &selected));
        }
    }
    assert_eq!(plans.steps - start, 128 * signatures.len());
    assert_eq!(plans.retained_bytes, retained);
    Ok(())
}

#[test]
#[cfg(feature = "object-interop-tests")]
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn indirect_object_calls_preserve_selected_classes_and_pointer_fields()
-> Result<(), Box<dyn std::error::Error>> {
    objects::check_cases(
        "aggregate-abi/indirect",
        "pa30/tests/spec/aggregate-abi/indirect.c",
        &[("indirect", "pa30/tests/spec/aggregate-abi/indirect.rs")],
    )
}
