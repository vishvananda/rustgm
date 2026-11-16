//! A call result is a coercion site, and its expected type only hints at the
//! callee's inputs. When the hint gives a by-value input an unsized type, as
//! alloc's `fn from(array: [T; N]) -> Box<[T]> { Box::new(array) }` does,
//! rustc checks that argument against its formal type alone: the result is
//! inferred as `Box<[T; N]>` and then unsized by `CoerceUnsized`. A sized
//! hint, such as `&[T]` for a reference input, is kept.
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
const UNSIZE: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

/// One call after `CORE` and its rejection, or `None` when it is accepted.
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

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

/// A by-value input whose hint is unsized takes its type from its argument,
/// and the call result then coerces to the expected type.
#[test]
fn unsized_input_hints_leave_the_call_result_to_its_coercion() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/coerced-results")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in CASES {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(&path, format!("{CORE}{}", case.source))?;
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
