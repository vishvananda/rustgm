//! Original source contracts and argument/return ABI independence are observable.
use super::*;
use crate::{pipeline, test_source};
#[path = "indirect_arguments/indirect_observers.rs"]
mod native;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "pa30/tests/spec/forced-indirect-abi";
fn fixture(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!("{ROOT}/{name}.rs"))?)
}
fn facts(source: &str) -> TestResult<crate::sema::model::Declarations> {
    let report = test_source::resolve(source, true)?;
    Ok(pipeline::analyze_source(&report)?.facts)
}
fn signatures(facts: &crate::sema::model::Declarations) -> Vec<TypeId> {
    facts
        .functions
        .iter()
        .filter_map(|function| {
            matches!(facts.types.kind(function.signature), Some(Kind::Function { convention, .. })
            if convention.abi == types::callable::Abi::C)
            .then_some(function.signature)
        })
        .collect()
}
#[test]
fn original_contract_preserves_actual_struct_feature_form_and_macro_authority() -> TestResult {
    for name in [
        "word",
        "rust-representation",
        "generic",
        "packed",
        "empty",
        "original-const-shape",
        "transparent-wrapper",
        "rust-convention",
        "macro-allowance",
        "cfg-removed",
        "native-export",
        "native-return",
        "native-wrapper",
        "native-caller-copy",
        "native-register-pressure",
    ] {
        facts(&fixture(name)?).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in [
        "missing-gate",
        "list",
        "value",
        "duplicate",
        "unsafe-wrapper",
        "function",
        "enum",
        "union",
        "alias",
        "field",
        "transparent-conflict",
        "transparent-conflict-reversed",
        "macro-without-allowance",
    ] {
        let error = facts(&fixture(name)?)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(
            !error.contains("not in path scope"),
            "attribute was never validated: {name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn pointer_arguments_and_normal_results_keep_distinct_cached_authority() -> TestResult {
    assert_eq!(size_of::<types::aggregate::Repr>(), 20);
    assert_eq!(size_of::<types::aggregate::Definition>(), 36);
    assert_eq!(size_of::<Plans>(), 88);
    assert_eq!(size_of::<Passing>(), 16);
    let mut facts = facts(
        "#![feature(rustc_attrs)]#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);extern \"C\" fn observed(x:S)->S{x}fn main(){}",
    )?;
    let signature = *signatures(&facts).first().ok_or("C signature")?;
    let Some(Kind::Function { result, .. }) = facts.types.kind(signature) else {
        return Err("C signature disappeared".into());
    };
    let mut plans = Plans::default();
    let normal = plans.passing(&facts.types, &facts.aggregates, &mut facts.layouts, result)?;
    let plan = plans.demand(
        &facts.types,
        &facts.aggregates,
        &mut facts.layouts,
        signature,
    )?;
    assert_eq!(&*plan.parameters, &[Passing::Indirect(result)]);
    assert_eq!(plan.result, normal);
    assert_eq!(
        normal,
        Passing::Object(Object {
            bytes: 8,
            align: 8,
            classes: Classes::Eightbytes {
                first: Class::Integer,
                second: None
            }
        })
    );
    let retained = plans.retained_bytes;
    for _ in 0..128 {
        assert_eq!(
            plans.argument(&facts.types, &facts.aggregates, &mut facts.layouts, result)?,
            plan.parameters[0]
        );
        assert_eq!(
            plans.passing(&facts.types, &facts.aggregates, &mut facts.layouts, result)?,
            normal
        );
    }
    assert_eq!(plans.retained_bytes, retained);
    let mut reversed = Plans::default();
    assert_eq!(
        reversed.argument(&facts.types, &facts.aggregates, &mut facts.layouts, result)?,
        plan.parameters[0]
    );
    assert_eq!(
        reversed.passing(&facts.types, &facts.aggregates, &mut facts.layouts, result)?,
        normal
    );
    Ok(())
}
fn graph(width: usize) -> TestResult<String> {
    use std::fmt::Write;
    let mut source = String::from(
        "#![feature(rustc_attrs)]#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct N0(u64);",
    );
    for index in 1..=width {
        write!(
            source,
            "#[repr(transparent)]struct N{index}(N{});extern \"C\" fn f{index}(_:N{index}){{}}",
            index - 1
        )?;
    }
    source.push_str("fn main(){}");
    Ok(source)
}
#[test]
fn shared_transparent_chains_pay_once_and_ordinary_paths_stay_cold() -> TestResult {
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut facts = facts(&graph(width)?)?;
        let signatures = signatures(&facts);
        let mut plans = Plans::default();
        for signature in signatures.iter().rev() {
            let plan = plans.demand(
                &facts.types,
                &facts.aggregates,
                &mut facts.layouts,
                *signature,
            )?;
            assert!(matches!(plan.parameters[0], Passing::Indirect(_)));
        }
        let before = (plans.steps, plans.retained_bytes);
        for signature in &signatures {
            plans.demand(
                &facts.types,
                &facts.aggregates,
                &mut facts.layouts,
                *signature,
            )?;
        }
        assert_eq!(
            (plans.steps - before.0, plans.retained_bytes - before.1),
            (width, 0)
        );
        observations.push(before);
        eprintln!(
            "indirect width={width} work={} bytes={}",
            before.0, before.1
        );
    }
    for index in [0, 1] {
        let a = if index == 0 {
            observations[1].0 - observations[0].0
        } else {
            observations[1].1 - observations[0].1
        };
        let b = if index == 0 {
            observations[2].0 - observations[1].0
        } else {
            observations[2].1 - observations[1].1
        };
        assert_eq!(b, 4 * a);
    }
    let mut facts = facts("#[repr(C)]struct S(u64);extern \"C\" fn f(_:S){}fn main(){}")?;
    let mut plans = Plans::default();
    for signature in signatures(&facts) {
        plans.demand(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            signature,
        )?;
    }
    assert!(plans.composites.is_none());
    Ok(())
}
#[test]
fn old_record_metadata_cannot_erase_argument_policy() -> TestResult {
    let report = test_source::resolve(
        "#![feature(rustc_attrs)]#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]pub struct S{pub value:u64}pub extern \"C\" fn f(x:S)->u64{x.value}",
        true,
    )?;
    let mut facts = pipeline::analyze_source(&report)?.facts;
    let error = crate::sema::crates::export(
        &report,
        &mut facts,
        crate::support::crates::CrateIdentity::new("indirect", "contract")?,
    )
    .err()
    .ok_or("metadata erased argument policy")?
    .to_string();
    assert!(
        error.contains("representation is outside the metadata profile"),
        "{error}"
    );
    Ok(())
}
#[test]
fn rejected_argument_work_and_storage_do_not_publish_policy_cache() -> TestResult {
    let mut facts = facts(
        "#![feature(rustc_attrs)]#[repr(C)]#[rustc_pass_indirectly_in_non_rustic_abis]struct S(u64);extern \"C\" fn f(_:S){}fn main(){}",
    )?;
    let signature = *signatures(&facts).first().ok_or("C signature")?;
    let Some(Kind::Function { parameters, .. }) = facts.types.kind(signature) else {
        return Err("missing signature".into());
    };
    let ty = facts
        .types
        .list(parameters)
        .ok_or("missing parameter list")?[0];
    for (steps, retained_bytes) in [(4_000_000, 0), (0, 64_000_000)] {
        let mut plans = Plans {
            steps,
            retained_bytes,
            ..Plans::default()
        };
        let error = plans
            .argument(&facts.types, &facts.aggregates, &mut facts.layouts, ty)
            .err()
            .ok_or("unpaid argument policy")?;
        assert!(error.contains("budget"), "{error}");
        assert!(plans.composites.is_none());
        assert!(plans.types.is_empty());
        assert!(plans.signatures.is_empty());
    }
    Ok(())
}
#[test]
fn actual_export_import_wrapper_copy_pressure_and_return_bodies_emit_checked_lowir() -> TestResult {
    for name in [
        "native-export",
        "native-return",
        "native-wrapper",
        "native-caller-copy",
        "native-register-pressure",
    ] {
        let input = fixture(name)?;
        let report = test_source::resolve(&input, true)?;
        let pipeline::Analysis {
            mut facts,
            prepared,
        } = pipeline::analyze_source(&report)?;
        let program = crate::lower::instances::build_roots_prepared(
            &report,
            &mut facts,
            crate::lower::Roots {
                entry: None,
                exports: &[],
            },
            prepared,
        )?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../obj/parallel/forced-indirect-abi-v1/candidate/{name}"
        ));
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("provider.rs"), input)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}
