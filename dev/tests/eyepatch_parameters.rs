//! `#[may_dangle]` (feature `dropck_eyepatch`) marks a lifetime or type
//! parameter of an `unsafe impl Drop`, as alloc's `Vec`, `Box`, `Rc` and
//! collection destructors do. Drop checking here has no opt-out for it, so
//! the bring-up profile admits a marked impl with a recorded waiver and every
//! strict compilation rejects it.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --crate-name core --cap-lints=allow`); it accepts `type-eyepatch` and
//! `lifetime-eyepatch` and rejects the others: `ungated` E0658, `safe-impl`
//! E0569, `unmarked-unsafe` E0199, `struct-parameter`, `other-trait` and
//! `inherent-impl` with "`#[may_dangle]` must be applied to a lifetime or type
//! generic parameter in `Drop` impl", and `with-arguments` as a malformed
//! attribute input.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const FEATURES: &str = "#![feature(no_core, lang_items, rustc_attrs, dropck_eyepatch)]\n";
const UNGATED: &str = "#![feature(no_core, lang_items, rustc_attrs)]\n";
const CORE: &str = r#"#![no_core]
#![allow(internal_features, dead_code)]
#[lang = "pointee_sized"]
pub trait PointeeSized {}
#[lang = "meta_sized"]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
pub trait Sized: MetaSized {}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "drop"]
pub trait Drop {
    fn drop(&mut self);
}
pub trait Other {
    fn other(&self);
}
pub struct Holder<T>(T);
pub struct Pair<'a, T>(&'a T);
"#;

const ADMITTED: &str = "bringup-waived\t`#[may_dangle]` is admitted without its drop-check opt-out: drop checking still requires every argument of a marked Drop impl\n";
const STRICT: &str = "`#[may_dangle]` drop-check opt-out is not implemented";
const MISPLACED: &str =
    "`#[may_dangle]` must be applied to a lifetime or type generic parameter in `Drop` impl";

/// One case: whether its core requests the feature, the items it appends and
/// its rejection in both profiles, or `None` for the marked impls that only
/// bring-up admits.
struct Case {
    name: &'static str,
    gated: bool,
    items: &'static str,
    rejected: Option<&'static str>,
}
const CASES: [Case; 9] = [
    Case {
        name: "type-eyepatch",
        gated: true,
        items: "unsafe impl<#[may_dangle] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        rejected: None,
    },
    Case {
        name: "lifetime-eyepatch",
        gated: true,
        items: "unsafe impl<#[may_dangle] 'a, T> Drop for Pair<'a, T> {\n    fn drop(&mut self) {}\n}\n",
        rejected: None,
    },
    Case {
        name: "ungated",
        gated: false,
        items: "unsafe impl<#[may_dangle] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        rejected: Some("`#[may_dangle]` requires dropck_eyepatch or exact token allowance"),
    },
    Case {
        name: "safe-impl",
        gated: true,
        items: "impl<#[may_dangle] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        rejected: Some("`#[may_dangle]` requires an `unsafe impl` of Drop"),
    },
    Case {
        name: "unmarked-unsafe",
        gated: true,
        items: "unsafe impl<T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        rejected: Some("Drop requires a positive safe impl without trait arguments"),
    },
    Case {
        name: "struct-parameter",
        gated: true,
        items: "pub struct Marked<#[may_dangle] T>(T);\n",
        rejected: Some(MISPLACED),
    },
    Case {
        name: "other-trait",
        gated: true,
        items: "impl<#[may_dangle] T> Other for Holder<T> {\n    fn other(&self) {}\n}\n",
        rejected: Some(MISPLACED),
    },
    Case {
        name: "inherent-impl",
        gated: true,
        items: "impl<#[may_dangle] T> Holder<T> {}\n",
        rejected: Some(MISPLACED),
    },
    Case {
        name: "with-arguments",
        gated: true,
        items: "unsafe impl<#[may_dangle(all)] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        rejected: Some("may_dangle requires an outer word attribute"),
    },
];

/// Type `path` as a source core, in the bring-up profile when `skip` names
/// its (empty) skip list.
fn typed(path: &Path, skip: Option<&Path>) -> Result<(bool, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
        .args(["--crate-name", "core", "--crate-disambiguator"])
        .args(["eyepatch_bringup", "--source-core", "core@eyepatch_bringup"]);
    if let Some(skip) = skip {
        command.arg("--bringup-skip-list").arg(skip);
    }
    let output = command.arg(path).output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

/// Marked `Drop` impls pass only in bring-up and only with a recorded waiver;
/// every misplaced, ungated or mismatched use is rejected in both profiles.
#[test]
fn eyepatches_are_waived_in_bringup_and_rejected_in_strict_mode() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/eyepatch-parameters")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let skip = folder.join("skip.tsv");
    std::fs::write(&skip, "")?;
    let mut failures = Vec::new();
    for case in CASES {
        let header = if case.gated { FEATURES } else { UNGATED };
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(&path, format!("{header}{CORE}{}", case.items))?;
        let (strict, strict_err) = typed(&path, None)?;
        let (bringup, bringup_err) = typed(&path, Some(&skip))?;
        let matches = match case.rejected {
            None => {
                !strict && rejects(&strict_err, STRICT) && bringup && bringup_err.contains(ADMITTED)
            }
            Some(message) => {
                !strict
                    && rejects(&strict_err, message)
                    && !bringup
                    && rejects(&bringup_err, message)
                    && !bringup_err.contains(ADMITTED)
            }
        };
        if !matches {
            failures.push(format!(
                "{}: strict={strict} {strict_err}bringup={bringup} {bringup_err}",
                case.name
            ));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
