//! Internal attributes whose meaning this compiler lacks: the bring-up
//! profile admits each one with a recorded waiver, every strict compilation
//! rejects it, and both profiles reject a misplaced, ungated or malformed use.
//!
//! `#[may_dangle]` (feature `dropck_eyepatch`) marks a lifetime or type
//! parameter of an `unsafe impl Drop`, as alloc's `Vec`, `Box`, `Rc` and
//! collection destructors do; drop checking here has no opt-out for it. The
//! allocator attributes (`rustc_allocator`, `rustc_deallocator`,
//! `rustc_reallocator`, `rustc_allocator_zeroed`,
//! `rustc_allocator_zeroed_variant`) only inform rustc's LLVM backend,
//! `rustc_std_internal_symbol` names a symbol, which declaration linkage
//! implements, and `rustc_conversion_suggestion` only feeds rustc's
//! diagnostics, so both profiles accept all of them without a waiver once
//! their form, owner and gate hold. The lang items `owned_box`,
//! `global_alloc_ty` and `String` have no role here; bring-up admits their
//! structs as ordinary ones after rustc's target and generic requirement.
//! Without that role no intrinsic signature position can be proved to be
//! `Box`, so bring-up lets any nominal type stand in a `Box` position of
//! `write_box_via_move`, which alloc's `vec!` calls.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --crate-name core --cap-lints=allow`).
//! - Eyepatches: it accepts `type-eyepatch` and `lifetime-eyepatch` and
//!   rejects the others: `ungated` E0658, `safe-impl` E0569, `unmarked-unsafe`
//!   E0199, `struct-parameter`, `other-trait` and `inherent-impl` with
//!   "`#[may_dangle]` must be applied to a lifetime or type generic parameter
//!   in `Drop` impl", and `with-arguments` as a malformed attribute input.
//! - Allocator attributes: it accepts `allocator-entries`,
//!   `internal-definition` and `conversion-suggestion` and rejects
//!   `ungated` E0658, `struct-owner` with "`#[rustc_allocator]` attribute
//!   cannot be used on structs", `variant-without-value` E0539 and
//!   `word-with-arguments` E0565.
//! - Lang roles: it accepts `owned-box`, `global-allocator` and `string` and
//!   rejects `box-without-parameter` E0718, `string-on-enum` with
//!   "`#[lang = "String"]` attribute cannot be used on enums" and
//!   `unknown-role` E0522.
//! - Box intrinsics: with `--emit=metadata`, which stops before the MIR pass
//!   that requires a `Unique` field in `Box` and so ends `write-box-via-move`
//!   with an internal error at `--emit=obj`, it accepts `write-box-via-move`
//!   and rejects `write-box-wrong-output` and `write-box-wrong-value` with
//!   E0308.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const EYEPATCH: &str = "#![feature(no_core, lang_items, rustc_attrs, dropck_eyepatch)]\n";
const INTERNAL: &str = "#![feature(no_core, lang_items, rustc_attrs)]\n";
const PLAIN: &str = "#![feature(no_core, lang_items)]\n";
const BOXED: &str =
    "#![feature(no_core, lang_items, rustc_attrs, intrinsics, transparent_unions)]\n";
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

const DANGLE_WAIVER: &str = "`#[may_dangle]` is admitted without its drop-check opt-out: drop checking still requires every argument of a marked Drop impl";
const DANGLE_STRICT: &str = "`#[may_dangle]` drop-check opt-out is not implemented";
const MISPLACED: &str =
    "`#[may_dangle]` must be applied to a lifetime or type generic parameter in `Drop` impl";

/// What both profiles do with one case.
#[derive(Clone, Copy)]
enum Outcome {
    /// Bring-up admits it and records `waiver`; strict mode rejects it with
    /// `strict`.
    Waived {
        waiver: &'static str,
        strict: &'static str,
    },
    /// Both profiles accept it and record no waiver.
    Accepted,
    /// Both profiles reject it with this message and record no waiver.
    Rejected(&'static str),
    /// Bring-up rejects it with `bringup` and strict mode with `strict`.
    Distinct {
        bringup: &'static str,
        strict: &'static str,
    },
}

/// One case: its crate feature header, the items it appends to the core and
/// its declared outcome.
struct Case {
    name: &'static str,
    header: &'static str,
    items: &'static str,
    outcome: Outcome,
}

const EYEPATCH_WAIVED: Outcome = Outcome::Waived {
    waiver: DANGLE_WAIVER,
    strict: DANGLE_STRICT,
};
const EYEPATCHES: [Case; 9] = [
    Case {
        name: "type-eyepatch",
        header: EYEPATCH,
        items: "unsafe impl<#[may_dangle] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        outcome: EYEPATCH_WAIVED,
    },
    Case {
        name: "lifetime-eyepatch",
        header: EYEPATCH,
        items: "unsafe impl<#[may_dangle] 'a, T> Drop for Pair<'a, T> {\n    fn drop(&mut self) {}\n}\n",
        outcome: EYEPATCH_WAIVED,
    },
    Case {
        name: "ungated",
        header: INTERNAL,
        items: "unsafe impl<#[may_dangle] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        outcome: Outcome::Rejected(
            "`#[may_dangle]` requires dropck_eyepatch or exact token allowance",
        ),
    },
    Case {
        name: "safe-impl",
        header: EYEPATCH,
        items: "impl<#[may_dangle] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        outcome: Outcome::Rejected("`#[may_dangle]` requires an `unsafe impl` of Drop"),
    },
    Case {
        name: "unmarked-unsafe",
        header: EYEPATCH,
        items: "unsafe impl<T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        outcome: Outcome::Rejected("Drop requires a positive safe impl without trait arguments"),
    },
    Case {
        name: "struct-parameter",
        header: EYEPATCH,
        items: "pub struct Marked<#[may_dangle] T>(T);\n",
        outcome: Outcome::Rejected(MISPLACED),
    },
    Case {
        name: "other-trait",
        header: EYEPATCH,
        items: "impl<#[may_dangle] T> Other for Holder<T> {\n    fn other(&self) {}\n}\n",
        outcome: Outcome::Rejected(MISPLACED),
    },
    Case {
        name: "inherent-impl",
        header: EYEPATCH,
        items: "impl<#[may_dangle] T> Holder<T> {}\n",
        outcome: Outcome::Rejected(MISPLACED),
    },
    Case {
        name: "with-arguments",
        header: EYEPATCH,
        items: "unsafe impl<#[may_dangle(all)] T> Drop for Holder<T> {\n    fn drop(&mut self) {}\n}\n",
        outcome: Outcome::Rejected("may_dangle requires an outer word attribute"),
    },
];

const ALLOCATORS: [Case; 7] = [
    Case {
        name: "allocator-entries",
        header: INTERNAL,
        items: r#"unsafe extern "Rust" {
    #[rustc_allocator]
    #[rustc_std_internal_symbol]
    #[rustc_allocator_zeroed_variant = "__rust_alloc_zeroed"]
    fn __rust_alloc(size: usize, align: usize) -> *mut u8;
    #[rustc_deallocator]
    #[rustc_std_internal_symbol]
    fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize);
    #[rustc_reallocator]
    #[rustc_std_internal_symbol]
    fn __rust_realloc(ptr: *mut u8, old_size: usize, align: usize, new_size: usize) -> *mut u8;
    #[rustc_allocator_zeroed]
    #[rustc_std_internal_symbol]
    fn __rust_alloc_zeroed(size: usize, align: usize) -> *mut u8;
}
"#,
        outcome: Outcome::Accepted,
    },
    Case {
        name: "internal-definition",
        header: INTERNAL,
        items: "#[rustc_std_internal_symbol]\npub unsafe fn __rdl_oom(size: usize, align: usize) {}\n",
        outcome: Outcome::Accepted,
    },
    Case {
        name: "conversion-suggestion",
        header: INTERNAL,
        items: "pub trait Render {\n    #[rustc_conversion_suggestion]\n    fn render(&self);\n}\nimpl<T> Holder<T> {\n    #[rustc_conversion_suggestion]\n    pub fn get(&self) {}\n}\n",
        outcome: Outcome::Accepted,
    },
    Case {
        name: "ungated",
        header: PLAIN,
        items: "#[rustc_std_internal_symbol]\npub fn internal() {}\n",
        outcome: Outcome::Rejected("compiler crate option requires rustc_attrs"),
    },
    Case {
        name: "struct-owner",
        header: INTERNAL,
        items: "#[rustc_allocator]\npub struct Marked;\n",
        outcome: Outcome::Rejected("rustc_allocator requires a function"),
    },
    Case {
        name: "variant-without-value",
        header: INTERNAL,
        items: "unsafe extern \"Rust\" {\n    #[rustc_allocator_zeroed_variant]\n    fn __rust_alloc(size: usize, align: usize) -> *mut u8;\n}\n",
        outcome: Outcome::Rejected(
            "rustc_allocator_zeroed_variant requires an outer `rustc_allocator_zeroed_variant = \"...\"` attribute",
        ),
    },
    Case {
        name: "word-with-arguments",
        header: INTERNAL,
        items: "unsafe extern \"Rust\" {\n    #[rustc_deallocator(all)]\n    fn __rust_dealloc(ptr: *mut u8, size: usize, align: usize);\n}\n",
        outcome: Outcome::Rejected("rustc_deallocator requires an outer word attribute"),
    },
];

const OWNED_BOX_WAIVER: &str = "lang item `owned_box` is admitted as an ordinary struct: Box's built-in dereference, move-out, drop and allocation semantics are not implemented";
const GLOBAL_ALLOC_WAIVER: &str = "lang item `global_alloc_ty` is admitted as an ordinary struct: the global allocator's Box layout and calling convention are not implemented";
const STRING_WAIVER: &str = "lang item `String` is admitted as an ordinary struct: its diagnostics and the uniqueness of its role are not checked";
const UNSUPPORTED: &str = "unsupported source language item";

const LANG_ROLES: [Case; 6] = [
    Case {
        name: "owned-box",
        header: INTERNAL,
        items: "pub struct Global;\n#[lang = \"owned_box\"]\npub struct Box<T: ?Sized, A = Global>(*const T, A);\n",
        outcome: Outcome::Waived {
            waiver: OWNED_BOX_WAIVER,
            strict: UNSUPPORTED,
        },
    },
    Case {
        name: "global-allocator",
        header: INTERNAL,
        items: "#[lang = \"global_alloc_ty\"]\npub struct Global;\n",
        outcome: Outcome::Waived {
            waiver: GLOBAL_ALLOC_WAIVER,
            strict: UNSUPPORTED,
        },
    },
    Case {
        name: "string",
        header: INTERNAL,
        items: "#[lang = \"String\"]\npub struct String;\n",
        outcome: Outcome::Waived {
            waiver: STRING_WAIVER,
            strict: UNSUPPORTED,
        },
    },
    Case {
        name: "box-without-parameter",
        header: INTERNAL,
        items: "#[lang = \"owned_box\"]\npub struct Box;\n",
        outcome: Outcome::Rejected(
            "`owned_box` lang item must be applied to a struct with at least 1 generic argument",
        ),
    },
    Case {
        name: "string-on-enum",
        header: INTERNAL,
        items: "#[lang = \"String\"]\npub enum String {}\n",
        outcome: Outcome::Rejected("`String` lang item must be applied to a struct"),
    },
    Case {
        name: "unknown-role",
        header: INTERNAL,
        items: "#[lang = \"no_such_item\"]\npub struct Unknown;\n",
        outcome: Outcome::Rejected(UNSUPPORTED),
    },
];

/// The core items `write_box_via_move` names: `MaybeUninit` and its parts,
/// and the `owned_box` struct, which only bring-up admits.
const UNINIT: &str = "#[lang = \"clone\"]\npub trait Clone: Sized {\n    #[lang = \"clone_fn\"]\n    fn clone(&self) -> Self;\n}\n#[lang = \"copy\"]\npub trait Copy: Clone {}\n#[lang = \"manually_drop\"]\n#[repr(transparent)]\npub struct ManuallyDrop<T>(T);\n#[lang = \"bikeshed_guaranteed_no_drop\"]\npub trait NoDrop {}\n#[lang = \"maybe_uninit\"]\n#[repr(transparent)]\npub union MaybeUninit<T> {\n    empty: (),\n    value: ManuallyDrop<T>,\n}\n#[lang = \"owned_box\"]\npub struct Box<T>(*mut T);\n#[rustc_intrinsic]\n";
const BOX_POSITIONS_WAIVER: &str = "source intrinsic Box positions are admitted without the owned-box language role: any nominal type matches them";
const BOX_INTRINSICS: [Case; 3] = [
    Case {
        name: "write-box-via-move",
        header: BOXED,
        items: "pub fn write_box_via_move<T>(b: Box<MaybeUninit<T>>, x: T) -> Box<MaybeUninit<T>>;\n",
        outcome: Outcome::Waived {
            waiver: BOX_POSITIONS_WAIVER,
            strict: UNSUPPORTED,
        },
    },
    Case {
        name: "write-box-wrong-output",
        header: BOXED,
        items: "pub fn write_box_via_move<T>(b: Box<MaybeUninit<T>>, x: T) -> u8;\n",
        outcome: Outcome::Distinct {
            bringup: "source intrinsic Box signature requires its real owned-box producer",
            strict: UNSUPPORTED,
        },
    },
    Case {
        name: "write-box-wrong-value",
        header: BOXED,
        items: "pub fn write_box_via_move<T>(b: Box<MaybeUninit<T>>, x: u8) -> Box<MaybeUninit<T>>;\n",
        outcome: Outcome::Distinct {
            bringup: "source intrinsic signature differs from its pinned declaration",
            strict: UNSUPPORTED,
        },
    },
];

/// Type `path` as a source core, in the bring-up profile when `skip` names
/// its (empty) skip list.
fn typed(path: &Path, skip: Option<&Path>) -> Result<(bool, String)> {
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
        .args(["--crate-name", "core", "--crate-disambiguator"])
        .args(["waived_bringup", "--source-core", "core@waived_bringup"]);
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

/// Whether bring-up recorded exactly this waiver, or any waiver for `None`.
fn waived(stderr: &str, waiver: Option<&str>) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("bringup-waived\t")
            .is_some_and(|actual| waiver.is_none_or(|waiver| actual == waiver))
    })
}

/// Run every case, after the core and `prelude`, in both profiles; the
/// failures that differ from their declared outcome.
fn check(folder: &str, cases: &[Case], prelude: &str) -> Result<Vec<String>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/waived-attributes")
        .join(std::process::id().to_string())
        .join(folder);
    std::fs::create_dir_all(&folder)?;
    let skip = folder.join("skip.tsv");
    std::fs::write(&skip, "")?;
    let mut failures = Vec::new();
    for case in cases {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(
            &path,
            format!("{}{CORE}{prelude}{}", case.header, case.items),
        )?;
        let (strict, strict_err) = typed(&path, None)?;
        let (bringup, bringup_err) = typed(&path, Some(&skip))?;
        let matches = match case.outcome {
            Outcome::Waived {
                waiver,
                strict: message,
            } => {
                !strict
                    && rejects(&strict_err, message)
                    && !waived(&strict_err, None)
                    && bringup
                    && waived(&bringup_err, Some(waiver))
            }
            Outcome::Accepted => {
                strict && bringup && !waived(&strict_err, None) && !waived(&bringup_err, None)
            }
            Outcome::Rejected(message) => {
                !strict
                    && rejects(&strict_err, message)
                    && !bringup
                    && rejects(&bringup_err, message)
                    && !waived(&bringup_err, None)
            }
            Outcome::Distinct {
                bringup: expected,
                strict: message,
            } => {
                !strict
                    && rejects(&strict_err, message)
                    && !bringup
                    && rejects(&bringup_err, expected)
            }
        };
        if !matches {
            failures.push(format!(
                "{}: strict={strict} {strict_err}bringup={bringup} {bringup_err}",
                case.name
            ));
        }
    }
    Ok(failures)
}

/// Marked `Drop` impls pass only in bring-up and only with a recorded waiver;
/// every misplaced, ungated or mismatched use is rejected in both profiles.
#[test]
fn eyepatches_are_waived_in_bringup_and_rejected_in_strict_mode() -> Result {
    let failures = check("eyepatch", &EYEPATCHES, "")?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Allocator entries, internal symbols and conversion suggestions pass in
/// both profiles without a waiver, as rustc accepts them; an ungated,
/// misplaced or malformed use is rejected in both.
#[test]
fn allocator_attributes_are_accepted_in_both_profiles_after_validation() -> Result {
    let failures = check("allocator", &ALLOCATORS, "")?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// alloc's `owned_box`, `global_alloc_ty` and `String` declarations pass only
/// in bring-up and only with a recorded waiver, after rustc's target and
/// generic requirement; a misplaced or unknown role is rejected in both.
#[test]
fn unimplemented_lang_roles_are_waived_in_bringup_and_rejected_in_strict_mode() -> Result {
    let failures = check("lang-roles", &LANG_ROLES, "")?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `write_box_via_move` passes only in bring-up, with a recorded waiver for
/// its `Box` positions; its other positions are still matched exactly.
#[test]
fn box_intrinsic_positions_are_waived_in_bringup_and_rejected_in_strict_mode() -> Result {
    let failures = check("box-intrinsics", &BOX_INTRINSICS, UNINIT)?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
