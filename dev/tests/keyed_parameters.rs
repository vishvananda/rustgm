//! Real marker, field and feature evidence governs aggregate const parameters.
use crate::provider_traits::{TestResult, lower};
use std::path::{Path, PathBuf};

const CASES: &[(&str, bool)] = &[
    ("struct-value", true),
    ("adt-feature", true),
    ("forward-default", true),
    ("enum-value", true),
    ("assume-fields", true),
    ("array-value", true),
    ("tuple-value", true),
    ("ctfe-struct", true),
    ("missing-feature", false),
    ("missing-marker", false),
    ("lookalike-marker", false),
    ("invalid-child", false),
    ("primitive-marker", false),
    ("unused-missing-marker", false),
];

#[test]
fn typed_const_inference_keeps_canonical_products_and_rigid_parameters_distinct() -> TestResult {
    use rustgm::types::{
        ItemId, Kind, Primitive, Store, TypeId,
        arguments::{ConstKind, Constant},
        infer::Context,
        parameters::{Parameter, ParameterKind},
    };
    let mut store = Store::default();
    let boolean = TypeId::primitive(Primitive::Bool);
    let fields = store.intern_list(&[boolean, boolean])?;
    let ty = store.intern(Kind::Tuple(fields))?;
    let yes = store.intern_constant(Constant {
        ty: boolean,
        bits: 1,
    })?;
    let no = store.intern_constant(Constant {
        ty: boolean,
        bits: 0,
    })?;
    let value = store.intern_aggregate_constant(ty, None, &[yes, no], |_, _| Ok(()))?;
    let other = store.intern_aggregate_constant(ty, None, &[no, yes], |_, _| Ok(()))?;
    let parameter = store.intern_parameter(Parameter {
        owner: ItemId(7),
        index: 0,
        kind: ParameterKind::Const { ty },
    })?;
    let rigid = store.intern_const(ConstKind::Parameter(parameter))?;
    let mut context = Context::default();
    let inferred = context.const_variable(&store, ty)?;
    let supplied = context.known_const(&store, value)?;
    context.equate_consts(inferred, supplied)?;
    assert_eq!(context.resolved_const(&mut store, inferred)?, value);
    let wrong = context.known_const(&store, other)?;
    assert!(context.equate_consts(inferred, wrong).is_err());
    let scalar = context.known_const(&store, yes)?;
    assert!(context.equate_consts(inferred, scalar).is_err());
    let parameter = context.known_const(&store, rigid)?;
    assert!(context.equate_consts(inferred, parameter).is_err());
    assert_eq!(store.constant(rigid), None);
    assert!(!store.const_is_value(rigid));
    Ok(())
}
fn root() -> TestResult<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?
        .to_owned())
}
fn source(name: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(root()?.join(format!(
        "pa25/tests/spec/aggregate-parameters/{name}.rs"
    )))?)
}

#[test]
fn source_const_parameters_require_actual_features_markers_and_fields() -> TestResult {
    let mut failures = Vec::new();
    for &(name, accepted) in CASES {
        let result = lower(&source(name)?);
        match result {
            Ok(_) if !accepted => failures.push(format!("{name}: accepted invalid source")),
            Err(error) if accepted => failures.push(format!("{name}: {error}")),
            Err(error) => {
                let message = if name == "missing-feature" {
                    "require min_adt_const_params"
                } else if name == "invalid-child" {
                    "every real child"
                } else {
                    "actual ConstParamTy"
                };
                if !error.to_string().contains(message) {
                    failures.push(format!("{name}: wrong reason: {error}"));
                }
            }
            Ok(_) => {}
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn canonical_source_arguments_execute_struct_enum_and_forward_defaults() -> TestResult {
    let root = root()?;
    let folder = root
        .join("obj/parallel/aggregate-parameters-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let observer =
        std::fs::read_to_string(root.join("pa25/tests/spec/aggregate-parameters/observer.c"))?;
    for (name, expected) in [
        ("struct-value", "7\n"),
        ("adt-feature", "7\n"),
        ("forward-default", "9\n"),
        ("enum-value", "11\n"),
        ("assume-fields", "13\n"),
        ("array-value", "15\n"),
        ("tuple-value", "17\n"),
        ("ctfe-struct", "19\n"),
    ] {
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &lower(&source(name)?)?,
            &observer,
            expected,
        )?;
    }
    Ok(())
}
