//! C objects retain canonical classes across imports, exports and callbacks.
#[path = "aggregate_calling/composites.rs"]
mod composites;
use crate::foreign_frontend as frontend;
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
            matches!(
                name,
                "interop" | "aligned" | "indirect" | "nested" | "array"
            ),
            "{name}: {:?}",
            result.as_ref().err()
        );
        if matches!(name, "rust-repr" | "packed" | "union" | "enum" | "empty") {
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
fn direct_wide_raw_pointer_calls_preserve_both_integer_words()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lowir::boundary::{Class, Classes, Type};

    let source = r#"
trait Payload {}
unsafe extern "C" { fn imported(pointer: *mut dyn Payload) -> *mut dyn Payload; }
#[unsafe(no_mangle)]
unsafe extern "C" fn round_trip(pointer: *mut dyn Payload) -> *mut dyn Payload {
    unsafe { imported(pointer) }
}
#[unsafe(no_mangle)]
extern "C" fn round_trip_slice(pointer: *mut [u8]) -> *mut [u8] { pointer }
fn main() {}
"#;
    let program = frontend::checked(source)?;
    for name in ["imported", "round_trip", "round_trip_slice"] {
        let symbol = program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .ok_or_else(|| format!("missing C symbol {name}"))?;
        let boundary = program.object_boundary(symbol.signature)?;
        let Type::Object(result) = boundary.result else {
            return Err(format!("{name} returned a scalar").into());
        };
        assert_eq!((result.span.bytes, result.span.align), (16, 8));
        assert_eq!(
            result.classes,
            Classes::Eightbytes {
                first: Class::Integer,
                second: Some(Class::Integer),
            }
        );
        assert_eq!(boundary.parameters.as_ref(), &[Type::Object(result)]);
        assert_eq!(symbol.body.is_some(), name != "imported");
    }
    Ok(())
}

#[test]
fn transparent_scalar_calls_preserve_integer_and_float_classes()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::lowir::{
        boundary::{Class, Classes, Type},
        vocabulary::Type as Scalar,
    };

    let source = r#"
#[repr(transparent)] struct CInt((), i32);
#[repr(transparent)] struct Nested(CInt, ());
#[repr(transparent)] struct Float(f64);
#[repr(C)] struct Mixed { lhs: Nested, rhs: Float }
unsafe extern "C" { fn imported(value: Nested) -> Nested; }
#[unsafe(no_mangle)]
unsafe extern "C" fn call(pointer: unsafe extern "C" fn(Nested) -> Nested, n: Nested) -> Nested {
    unsafe { pointer(imported(n)) }
}
#[unsafe(no_mangle)]
extern "C" fn mixed(value: Mixed) -> Mixed { value }
fn main() {}
"#;
    let program = frontend::checked(source)?;
    for (name, expected) in [
        ("imported", &[Scalar::I32][..]),
        ("call", &[Scalar::Ptr, Scalar::I32][..]),
    ] {
        let symbol = program
            .symbols
            .iter()
            .find(|symbol| symbol.name == name)
            .ok_or(name)?;
        assert_eq!(program.parameters(symbol.signature)?, expected);
        assert_eq!(
            program.result_type(symbol.signature)?,
            Type::Scalar(Scalar::I32)
        );
    }
    let mixed = program
        .symbols
        .iter()
        .find(|symbol| symbol.name == "mixed")
        .ok_or("mixed")?;
    let Type::Object(result) = program.result_type(mixed.signature)? else {
        return Err("mixed result lacks its object boundary".into());
    };
    assert_eq!(
        result.classes,
        Classes::Eightbytes {
            first: Class::Integer,
            second: Some(Class::Sse)
        }
    );
    assert_eq!(
        program.parameter_type(mixed.signature, 0)?,
        Type::Object(result)
    );
    Ok(())
}

#[test]
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
#[ignore = "requires the optional pinned CPPGM object profile and host C compiler"]
fn indirect_object_calls_preserve_selected_classes_and_pointer_fields()
-> Result<(), Box<dyn std::error::Error>> {
    objects::check_cases(
        "aggregate-abi/indirect",
        "pa30/tests/spec/aggregate-abi/indirect.c",
        &[("indirect", "pa30/tests/spec/aggregate-abi/indirect.rs")],
    )
}
