//! A call result is a coercion site, and its expected type only hints at the
//! callee's inputs. When the hint gives a by-value input an unsized type, as
//! alloc's `fn from(array: [T; N]) -> Box<[T]> { Box::new(array) }` does,
//! rustc checks that argument against its formal type alone: the result is
//! inferred as `Box<[T; N]>` and then unsized by `CoerceUnsized`. A sized
//! hint, such as `&[T]` for a reference input, is kept.
//!
//! An `as` cast is likewise first tried as a coercion of its operand, and a
//! cast between references is valid only as such a coercion cast, as to an
//! object or to the same referent. alloc's `ThinBox::new_unsize` writes
//! `ptr::metadata(&value as &Dyn)` under `T: Unsize<Dyn>`: a reference to a
//! type parameter that the caller's own clause unsizes to. Each instance
//! resolves that conversion once its referents are known, here to an object
//! and to a slice.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --crate-name core --cap-lints=allow`). It accepts every case but
//! `unsizeless-call-rejected`, which it rejects with E0308. The cases were
//! first plain crates, which cannot name `std::marker::Unsize` here; as
//! source cores they then gained, after runs that both compilers rejected for
//! the missing items, the raw-pointer and reference `CoerceUnsized` impls, the
//! `copy` lang item rustc requires and the `clone_fn` `RustGM` requires.
//! `object-call-coerces` and `sized-hint-kept` were declared after the first
//! four had run. `unsizeless-call-rejected` was rejected as "incompatible
//! types" before the change; it is now rejected where its result coerces,
//! since `T: Unsize<[T]>` cannot hold, and that message was pinned then.
//!
//! The `CASTS` were declared later, before either compiler ran. rustc
//! accepts every one but `generic-unsizeless-reference-cast`, which it
//! rejects with E0605 ("non-primitive cast"). Their core first lacked the
//! mutable-reference `CoerceUnsized` impl, which `MUTABLE` adds; the
//! declarations were kept and `RustGM`'s rejection message was pinned after
//! the change. The two `T: Unsize<U>` casts were first rejected by `RustGM`
//! and accepted once a clause-proved unsizing existed. `generic-unsize-coercion`,
//! the same unsizing written as a coercion, was declared after them; rustc
//! accepts it and `RustGM` rejected it as "incompatible types" before.
//! `generic-unsize-raw-pointer-cast` was declared later still, when census
//! movement showed alloc's `ptr::dangling::<T>() as *const Dyn` passing; rustc
//! accepts it and `RustGM` rejected it as "pointer cast cannot invent or
//! change metadata" before.
//!
//! The `INSTANCES` were declared before either compiler ran; rustc accepts
//! both. `instance-object`'s core first lacked the reference
//! `DispatchFromDyn` impl, without which rustc rejects `Shape` with E0038;
//! it was added and the declaration kept.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const CORE: &str = concat!(
    "#![feature(no_core, lang_items, rustc_attrs)]\n",
    "#![no_core]\n",
    "#[lang = \"pointee_sized\"]\n",
    "pub trait PointeeSized {}\n",
    "#[lang = \"meta_sized\"]\n",
    "pub trait MetaSized: PointeeSized {}\n",
    "#[lang = \"sized\"]\n",
    "pub trait Sized: MetaSized {}\n",
    "#[lang = \"legacy_receiver\"]\n",
    "pub trait LegacyReceiver: PointeeSized {}\n",
    "impl<T: PointeeSized> LegacyReceiver for &T {}\n",
    "#[lang = \"clone\"]\n",
    "pub trait Clone: Sized {\n",
    "    #[lang = \"clone_fn\"]\n",
    "    fn clone(&self) -> Self;\n",
    "}\n",
    "#[lang = \"copy\"]\n",
    "pub trait Copy: Clone {}\n",
    "#[lang = \"unsize\"]\n",
    "#[rustc_deny_explicit_impl]\n",
    "#[rustc_dyn_incompatible_trait]\n",
    "pub trait Unsize<T: PointeeSized>: PointeeSized {}\n",
    "#[lang = \"coerce_unsized\"]\n",
    "pub trait CoerceUnsized<T: PointeeSized>: Sized {}\n",
    "impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*const U> for *const T {}\n",
    "impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}\n",
    "pub struct Ptr<T: PointeeSized>(*const T);\n",
    "impl<T> Ptr<T> {\n",
    "    pub fn new(_x: T) -> Self {\n",
    "        loop {}\n",
    "    }\n",
    "}\n",
    "impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<Ptr<U>> for Ptr<T> {}\n",
);
const MUTABLE: &str = "impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a mut U> for &'b mut T {}\n";
const UNSIZE: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
const UNSIZE_CAST: &str = "incompatible types";

/// One item after the core and its rejection, or `None` when it is accepted.
struct Case {
    name: &'static str,
    source: &'static str,
    rejected: Option<&'static str>,
}
const CASES: [Case; 6] = [
    Case {
        name: "returned-call-coerces",
        source: "pub fn from_array<T, const N: usize>(array: [T; N]) -> Ptr<[T]> {\n    Ptr::new(array)\n}\n",
        rejected: None,
    },
    Case {
        name: "annotated-let-coerces",
        source: "pub fn from_array<T, const N: usize>(array: [T; N]) -> Ptr<[T]> {\n    let unsized_: Ptr<[T]> = Ptr::new(array);\n    unsized_\n}\n",
        rejected: None,
    },
    Case {
        name: "explicit-self-coerces",
        source: "pub fn from_array<T, const N: usize>(array: [T; N]) -> Ptr<[T]> {\n    Ptr::<[T; N]>::new(array)\n}\n",
        rejected: None,
    },
    Case {
        name: "object-call-coerces",
        source: "pub trait Shape {}\nimpl Shape for u8 {}\npub fn object(value: u8) -> Ptr<dyn Shape> {\n    Ptr::new(value)\n}\n",
        rejected: None,
    },
    Case {
        name: "sized-hint-kept",
        source: "pub fn borrowed<'a, T, const N: usize>(array: &'a [T; N]) -> Ptr<&'a [T]> {\n    Ptr::new(array)\n}\n",
        rejected: None,
    },
    Case {
        name: "unsizeless-call-rejected",
        source: "pub fn from_value<T>(x: T) -> Ptr<[T]> {\n    Ptr::new(x)\n}\n",
        rejected: Some(UNSIZE),
    },
];
const CASTS: [Case; 7] = [
    Case {
        name: "generic-unsize-raw-pointer-cast",
        source: "pub fn raw<T: Unsize<U>, U: PointeeSized>(p: *const T) -> *const U {\n    p as *const U\n}\n",
        rejected: None,
    },
    Case {
        name: "generic-unsize-coercion",
        source: "pub fn view<T: Unsize<U>, U: PointeeSized>(value: &T) -> &U {\n    let unsized_value: &U = value;\n    unsized_value\n}\n",
        rejected: None,
    },
    Case {
        name: "generic-unsize-reference-cast",
        source: "pub fn view<T: Unsize<U>, U: PointeeSized>(value: &T) -> &U {\n    value as &U\n}\n",
        rejected: None,
    },
    Case {
        name: "generic-unsize-mut-reference-cast",
        source: "pub fn view<T: Unsize<U>, U: PointeeSized>(value: &mut T) -> &mut U {\n    value as &mut U\n}\n",
        rejected: None,
    },
    Case {
        name: "object-reference-cast",
        source: "pub trait Shape {}\nimpl Shape for u8 {}\npub fn object(value: &u8) -> &dyn Shape {\n    value as &dyn Shape\n}\n",
        rejected: None,
    },
    Case {
        name: "same-parameter-reference-cast",
        source: "pub fn same<T: PointeeSized>(value: &T) -> &T {\n    value as &T\n}\n",
        rejected: None,
    },
    Case {
        name: "generic-unsizeless-reference-cast",
        source: "pub fn view<T, U: PointeeSized>(value: &T) -> &U {\n    value as &U\n}\n",
        rejected: Some(UNSIZE_CAST),
    },
];

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

fn check(folder: &str, prelude: &str, cases: &[Case]) -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj")
        .join(folder)
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in cases {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(&path, format!("{CORE}{prelude}{}", case.source))?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
            .args(["--crate-name", "core", "--crate-disambiguator", "unsizing"])
            .args(["--source-core", "core@unsizing"])
            .arg(&path)
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let matches = match case.rejected {
            None => output.status.success(),
            Some(message) => !output.status.success() && rejects(&stderr, message),
        };
        if !matches {
            failures.push(format!(
                "{}: success={} {stderr}",
                case.name,
                output.status.success()
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// A by-value input whose hint is unsized takes its type from its argument,
/// and the call result then coerces to the expected type.
#[test]
fn unsized_input_hints_leave_the_call_result_to_its_coercion() -> Result {
    check("coerced-results", "", &CASES)
}

/// A reference cast is accepted exactly where its operand coerces to it.
#[test]
fn reference_casts_are_coercion_casts() -> Result {
    check("coerced-casts", MUTABLE, &CASTS)
}

/// `view` casts under its own `T: Unsize<U>` clause and `twice` forwards to
/// it generically; each exported instance resolves the conversion.
const INSTANCES: &str = concat!(
    "impl Clone for u8 {\n    fn clone(&self) -> u8 {\n        *self\n    }\n}\n",
    "impl Copy for u8 {}\n",
    "#[lang = \"dispatch_from_dyn\"]\n",
    "pub trait DispatchFromDyn<T>: Sized {}\n",
    "impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}\n",
    "pub trait Shape {\n    fn area(&self) -> u8;\n}\n",
    "impl Shape for u8 {\n    fn area(&self) -> u8 {\n        *self\n    }\n}\n",
    "pub fn view<T: Unsize<U>, U: PointeeSized>(value: &T) -> &U {\n    value as &U\n}\n",
    "pub fn twice<T: Unsize<U>, U: PointeeSized>(value: &T) -> &U {\n    view::<T, U>(value)\n}\n",
    "#[unsafe(no_mangle)]\n",
    "pub extern \"C\" fn entry(x: &u8) -> u8 {\n    twice::<u8, dyn Shape>(x).area()\n}\n",
    "#[unsafe(no_mangle)]\n",
    "pub extern \"C\" fn first(a: &[u8; 3]) -> u8 {\n",
    "    let s: &[u8] = twice::<[u8; 3], [u8]>(a);\n",
    "    unsafe { *(s as *const [u8] as *const u8) }\n",
    "}\n",
);
fn instances() -> Result<rustgm::lowir::program::Program> {
    let source = format!("{CORE}{INSTANCES}");
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(&source, "bound_unsizing", &config)?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    let exports = facts
        .functions
        .iter()
        .filter(|function| {
            let name = report.resolution.definitions[function.definition.0 as usize].name;
            ["entry", "first"].contains(&report.package.names.get(name))
        })
        .map(|function| function.definition)
        .collect::<Vec<_>>();
    assert_eq!(exports.len(), 2);
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &exports,
        },
        prepared,
    )?)
}

/// Instances of a clause-proved unsizing lower to an object and a slice.
#[test]
fn bound_unsizing_instances_select_their_conversions() -> Result {
    instances().map(drop)
}

/// The object instance dispatches through its vtable and the slice instance
/// reads through its wide pointer.
#[test]
#[ignore = "requires the qualified CPPGM native backend"]
fn bound_unsizing_instances_execute() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/coerced-instances")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let observer = folder.join("observe.c");
    std::fs::write(
        &observer,
        "#include <stdint.h>\n#include <stdio.h>\nextern uint8_t entry(const uint8_t *);\nextern uint8_t first(const uint8_t *);\nint main(void){uint8_t x=7;uint8_t a[3]={4,5,6};printf(\"%u %u\\n\",entry(&x),first(a));return 0;}\n",
    )?;
    crate::observed_program::execute(&folder, &instances()?, &observer, b"7 4\n")
}

#[test]
fn weak_unsized_borrow_guides_calls_without_binding_their_result() -> Result {
    const CASES: &str = "pa31/tests/spec/unsized-borrow-expectations";
    let analyze = |name: &str| -> Result<Option<rustgm::resolve::package::Error>> {
        let source = crate::resolve::fixture(&format!("{CASES}/{name}.rs"))?;
        let report = crate::resolve::source_provider(&source, "borrow_expectation")?;
        Ok(rustgm::pipeline::analyze_source(&report).err())
    };
    for name in [
        "foundation",
        "nested-str",
        "direct-owner",
        "nested-owner",
        "bound-owner",
        "explicit-owner",
        "block-owner",
        "macro-owner",
    ] {
        if let Some(error) = analyze(name)? {
            return Err(format!("{name}: {error}").into());
        }
    }
    let error = analyze("sized-owner-opponent")?.ok_or("sized identity opponent accepted")?;
    assert_eq!(error.message, "incompatible types", "{error}");
    Ok(())
}

#[test]
fn unsized_borrow_keeps_inline_interfaces_macro_places_and_thin_calls() -> Result {
    const CASES: &str = "pa31/tests/spec/unsized-borrow-expectations";
    for name in ["inline-const-owner", "macro-place", "thin-nominal"] {
        let source = crate::resolve::fixture(&format!("{CASES}/{name}.rs"))?;
        let report = crate::resolve::source_provider(&source, "borrow_expectation")?;
        rustgm::pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
