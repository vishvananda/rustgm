//! Actual source unsizing roles retain independent goal and native contracts.
use crate::provider_traits::{TestResult, lower};
use std::path::Path;
#[path = "coercion_provider/discovery.rs"]
mod discovery;
#[path = "coercion_provider/envelopes.rs"]
mod envelopes;
#[path = "coercion_provider/erasure.rs"]
mod erasure;
#[path = "coercion_provider/referent_calls.rs"]
mod referent_calls;
#[path = "coercion_provider/struct_goals.rs"]
mod struct_goals;
#[path = "coercion_provider/unsize_binders.rs"]
mod unsize_binders;

const POSITIVE: &[&str] = &[
    "array-slice",
    "zero-array",
    "generic-length",
    "equal-lifetime",
    "object-goal",
    "object-scoped",
    "object-associated",
    "coerce-reference",
    "coerce-pointer",
    "coerce-mut-to-const",
    "array-element-inference",
    "callable-element-alpha",
    "dyn-width-8",
    "dyn-width-32",
    "dyn-width-128",
    "dyn-callers-8",
    "dyn-callers-32",
    "dyn-callers-128",
    "formation-declarations",
];
const NEGATIVE: &[&str] = &[
    "wrong-element",
    "identity-scalar",
    "identity-slice",
    "shortening-not-equality",
    "zero-still-equality",
    "object-unpaid",
    "object-escape",
    "object-associated-wrong",
    "coerce-shared-to-mut",
    "explicit-unsize-impl",
    "invalid-coerce-scalar",
    "invalid-coerce-element",
    "dyn-unsize-denied",
    "lookalike-no-builtin",
    "coerce-missing-outlives",
    "ordinary-no-roles",
    "other-owner-no-builtin",
    "callable-element-correlation",
    "formation-source-unpaid",
    "formation-target-unpaid",
    "formation-region-unpaid",
    "formation-projection-unpaid",
    "formation-valid-mismatch",
    "formation-deferred-other-invalid",
];
fn source(name: &str, extension: &str) -> TestResult<String> {
    Ok(std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../pa31/tests/spec/unsizing-roles/{name}.{extension}"
        )),
    )?)
}
#[test]
fn actual_unsizing_roles_retain_goal_equations_and_all_caller_debts() -> TestResult {
    let mut failures = Vec::new();
    for &name in POSITIVE {
        match lower(&source(name, "rs")?) {
            Ok(_) => println!("{name}: accepted"),
            Err(error) => failures.push(format!("{name}: {error}")),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn source_unsizing_never_substitutes_layout_name_or_one_way_lifetimes() -> TestResult {
    let mut failures = Vec::new();
    for &name in NEGATIVE {
        if lower(&source(name, "rs")?).is_ok() {
            failures.push(name);
        }
    }
    assert!(failures.is_empty(), "accepted: {failures:?}");
    Ok(())
}

#[test]
fn nominal_coercion_implementations_prove_one_actual_changed_field() -> TestResult {
    let folder = "pa31/tests/spec/nominal-coercions";
    let cases = crate::resolve::fixture(&format!("{folder}/cases.tsv"))?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in cases.lines() {
        let (name, expected) = line.split_once('\t').ok_or("nominal case columns")?;
        let text = crate::resolve::fixture(&format!("{folder}/{name}"))?;
        let result = lower(&text);
        match (expected, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error))
                if ![
                    "not implemented",
                    "budget",
                    "exhausted",
                    "lacks its completed",
                    "undefined",
                    "syntax",
                    "feature request",
                ]
                .iter()
                .any(|fence| error.to_string().contains(fence)) =>
            {
                println!("{name}: {error}");
            }
            (_, result) => failures.push(format!(
                "{name}: {}",
                result.map_or_else(|error| error.to_string(), |_| "accepted".into())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 16);
    // Previously qualified Rust-positive required fixture remains a real goal.
    lower(&source("nominal-coerce-required", "rs")?)?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn nominal_value_typing_retains_actual_goals_and_full_source_opponents() -> TestResult {
    let folder = "pa31/tests/spec/nominal-coercions/values";
    let text = crate::resolve::fixture(&format!("{folder}/native.rs"))?;
    let report = crate::resolve::source_provider(&text, "nominal_values")?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    let nominal = analysis
        .facts
        .unsizing_adjustments
        .values()
        .filter(|conversion| {
            matches!(
                conversion,
                rustgm::types::unsizing::Conversion::Nominal { .. }
            )
        })
        .count();
    assert_eq!(nominal, 6);
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    rustgm::lower::instances::inspect_prepared(&report, &mut facts, prepared, |_, _| ())?;
    for name in ["escape", "mutability", "element", "moved-source"] {
        let text = crate::resolve::fixture(&format!("{folder}/{name}.rs"))?;
        let report = crate::resolve::source_provider(&text, "nominal_opponent")?;
        let error = rustgm::pipeline::analyze_source(&report)
            .and_then(
                |rustgm::pipeline::Analysis {
                     mut facts,
                     prepared,
                 }| {
                    rustgm::lower::instances::inspect_prepared(
                        &report,
                        &mut facts,
                        prepared,
                        |_, _| (),
                    )
                },
            )
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(
            ![
                "not implemented",
                "budget",
                "exhausted",
                "undefined",
                "unsupported",
                "wrapper conversion plan",
                "aggregate conversion plan"
            ]
            .iter()
            .any(|fence| error.contains(fence)),
            "{name}: {error}"
        );
        println!("{name}: {error}");
        let cause = match name {
            "escape" => "required lifetime does not outlive its target",
            "moved-source" => "local may be uninitialized at this read",
            _ => "trait obligation requires an unambiguous matching assumption or implementation",
        };
        assert!(error.contains(cause), "{name}: {error}");
    }
    assert_eq!(
        size_of::<rustgm::types::unsizing::Conversion<rustgm::types::TypeId>>(),
        12
    );
    assert_eq!(size_of::<rustgm::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn nominal_call_results_infer_from_the_argument_before_unsizing() -> TestResult {
    let prefix =
        crate::resolve::fixture("pa31/tests/spec/explicit-dynamic-call-members/prefix.rs")?;
    let (prefix, _) = prefix
        .split_once("pub struct Unique")
        .ok_or("NonNull provider prefix")?;
    let declarations = r#"
        #[lang = "From"]
        pub trait From<T>: Sized { fn from(value: T) -> Self; }
        impl<T: PointeeSized> From<&T> for NonNull<T> {
            fn from(value: &T) -> Self { NonNull { pointer: value as *const T } }
        }
        #[repr(C)]
        pub struct Inner<T: ?Sized> { prefix: u8, data: T }
    "#;
    for (expression, target, expected) in [
        ("NonNull::from(value)", "[u8]", Some(true)),
        ("NonNull::<Inner<[u8; 1]>>::from(value)", "[u8]", Some(true)),
        ("NonNull::from(value)", "[u8; 1]", Some(false)),
        ("NonNull::<Inner<[u8]>>::from(value)", "[u8]", None),
        ("NonNull::from(value)", "[u16]", None),
    ] {
        let text = format!(
            "{prefix}{declarations}
             pub fn select(value: &Inner<[u8; 1]>) -> NonNull<Inner<{target}>> {{
                 let selected: NonNull<Inner<{target}>> = {expression};
                 selected
             }}"
        );
        let report = crate::resolve::source_provider(&text, "nominal_call_unsizing")?;
        let result = rustgm::pipeline::analyze_source(&report);
        if let Some(expected) = expected {
            let analysis = result?;
            let selected = analysis
                .facts
                .unsizing_adjustments
                .values()
                .any(|adjustment| {
                    matches!(
                        adjustment,
                        rustgm::types::unsizing::Conversion::Nominal { .. }
                    )
                });
            assert_eq!(selected, expected, "{expression} -> {target}");
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("accepted {expression} -> {target}"))?;
            assert!(
                error.message.contains("trait obligation requires"),
                "{expression} -> {target}: {error}"
            );
        }
    }
    let text = format!(
        "{prefix}{declarations}
         pub struct Tagged<T: PointeeSized, A>(NonNull<T>, PhantomData<A>);
         impl<T: PointeeSized + Unsize<U>, U: PointeeSized, A> CoerceUnsized<Tagged<U, A>>
             for Tagged<T, A> {{}}
         impl<T: PointeeSized, A: Copy> From<(&T, A)> for Tagged<T, A> {{
             fn from(value: (&T, A)) -> Self {{ Tagged(NonNull::from(value.0), PhantomData) }}
         }}
         pub fn with_tag(value: &Inner<[u8; 1]>) -> Tagged<Inner<[u8]>, u8> {{
             let selected: Tagged<Inner<[u8]>, u8> = Tagged::from((value, 7));
             selected
         }}"
    );
    let report = crate::resolve::source_provider(&text, "nominal_call_tag_hint")?;
    let analysis = rustgm::pipeline::analyze_source(&report)?;
    assert!(
        analysis
            .facts
            .unsizing_adjustments
            .values()
            .any(|adjustment| {
                matches!(
                    adjustment,
                    rustgm::types::unsizing::Conversion::Nominal { .. }
                )
            })
    );
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn nominal_wrapper_values_preserve_transfer_effects_and_real_cleanup() -> TestResult {
    let folder = "pa31/tests/spec/nominal-coercions/values";
    let text = crate::resolve::fixture(&format!("{folder}/native.rs"))?;
    let report = crate::resolve::source_provider(&text, "nominal_native")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let output = root.join("obj/parallel/nominal-coercions-v1/candidate-native");
    std::fs::create_dir_all(&output)?;
    crate::ffi_program::check(
        &root,
        &output,
        "wrappers",
        &program,
        &crate::resolve::fixture(&format!("{folder}/observer.c"))?,
        &crate::resolve::fixture(&format!("{folder}/native.expected"))?,
    )?;
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn nominal_slice_tails_keep_the_base_address_and_element_count() -> TestResult {
    let mut source = crate::resolve::fixture("pa31/tests/spec/nominal-coercions/values/native.rs")?;
    source.push_str(
        r#"
#[repr(C)]
struct InnerTail<T:?Sized> { head:u8, data:T }
#[repr(C)]
struct OuterTail<T:?Sized> { head:u8, inner:InnerTail<T> }
#[repr(transparent)]
struct SliceCarrier<T:PointeeSized> { ptr:*const T }
#[unstable(feature="coerce_unsized",issue="18598")]
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<*const U> for *const T {}
impl<T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<SliceCarrier<U>> for SliceCarrier<T> {}

#[unsafe(no_mangle)]
pub extern "C" fn observe_slice_tail()->u8 {
    let value = OuterTail { head:17, inner:InnerTail{head:19, data:[23u8,47]} };
    let thin = SliceCarrier { ptr:&value as *const OuterTail<[u8;2]> };
    let wide:SliceCarrier<OuterTail<[u8]>> = thin;
    let view = unsafe { &*wide.ptr };
    unsafe { record(view.head); record(view.inner.head); }
    second(&view.inner.data)
}
#[unsafe(no_mangle)]
pub extern "C" fn observe_empty_tail()->u8 {
    let value = OuterTail { head:29, inner:InnerTail{head:31, data:[]} };
    let thin = SliceCarrier { ptr:&value as *const OuterTail<[u8;0]> };
    let wide:SliceCarrier<OuterTail<[u8]>> = thin;
    let view = unsafe { &*wide.ptr };
    unsafe { record(view.head); record(view.inner.head); }
    match &view.inner.data { []=>91, _=>0 }
}
"#,
    );
    let program = crate::ffi_program::exported(&source, "nominal_slice_tail")?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join("obj/parallel/nominal-coercions-v1/slice-tail-native");
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        &root,
        &folder,
        "slice_tail",
        &program,
        r#"
#include <stdint.h>
#include <stdio.h>
extern uint8_t observe_slice_tail(void), observe_empty_tail(void);
void record(uint8_t tag) { printf("%u ", (unsigned)tag); }
int main(void) {
    printf("%u\n", (unsigned)observe_slice_tail());
    printf("%u\n", (unsigned)observe_empty_tail());
    return 0;
}
"#,
        "17 19 47\n29 31 91\n",
    )?;
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn source_unsizing_goal_native_answers() -> TestResult {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let folder = root
        .join("obj/parallel/pa31/coercion-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for &name in POSITIVE {
        let text = source(name, "rs")?;
        let program = lower(&text)?;
        std::fs::write(folder.join(format!("{name}.rs")), text)?;
        crate::ffi_program::check(root, &folder, name, &program, &source(name, "c")?, "7\n")?;
        println!("{name}: native7");
    }
    Ok(())
}

#[path = "coercion_provider/upcast_goals.rs"]
mod upcast_goals;
