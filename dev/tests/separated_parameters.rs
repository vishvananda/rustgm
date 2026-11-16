//! A generic parameter and a member of its item may share a name and a
//! namespace: rustc resolves the parameters in their own rib and the members
//! in the item's namespace, so a lexical name inside the item is the
//! parameter and a qualified name is the member. alloc's B-tree declares
//! `enum ForceResult<Leaf, Internal> { Leaf(Leaf), Internal(Internal) }`.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --crate-name core --cap-lints=allow`). It accepts every case without a
//! message. It rejects `field-is-parameter` and `impl-parameter-not-member`
//! with E0308, `default-names-later-parameter` with E0128,
//! `duplicate-variant` with E0428 and `duplicate-parameter` with E0403.
//! `RustGM`'s "incompatible types" message for the two E0308 cases was pinned
//! after its first run; their rejection was declared before. The core first
//! lacked `Copy`, which rustc needs to move a constructor's argument, and then
//! `clone_fn`, which `RustGM` needs for a source `Clone`. Both were added
//! before any verdict was compared.
//!
//! A staged library's availability inherits through the parameter rib like
//! any lexical parent. The staged cases are ordinary crates checked by
//! `rustc --crate-type=lib --cap-lints=allow`: it accepts each one except
//! `staged-unmarked-variant-like-parameter`, which it rejects with "enum has
//! missing stability attribute". Its rejection was declared first; the full
//! `RustGM` message was pinned after the first run, which ended in
//! "attribute" where the declaration did not.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const CORE: &str = r#"#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
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
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
"#;

const FORCE_RESULT: &str =
    "pub enum ForceResult<Leaf, Internal> {\n    Leaf(Leaf),\n    Internal(Internal),\n}\n";
const KEEP: &str = "pub struct Holder<T>(T);\npub trait Keep<T> {\n    type T;\n    fn keep(&self, value: T) -> T;\n}\n";

/// One case: the items it appends to the core and its rejection, or `None`
/// when the core is accepted.
struct Case {
    name: &'static str,
    items: &'static [&'static str],
    rejected: Option<&'static str>,
}
const CASES: [Case; 11] = [
    Case {
        name: "variant-like-parameter",
        items: &[
            FORCE_RESULT,
            "pub fn leaf(x: u8) -> ForceResult<u8, ()> {\n    ForceResult::Leaf(x)\n}\npub fn internal() -> ForceResult<u8, ()> {\n    ForceResult::<u8, ()>::Internal(())\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "field-is-parameter",
        items: &[
            FORCE_RESULT,
            "pub fn wrong(x: u16) -> ForceResult<u8, ()> {\n    ForceResult::Leaf(x)\n}\n",
        ],
        rejected: Some("incompatible types"),
    },
    Case {
        name: "unit-variant-like-parameter",
        items: &[
            "pub enum Unit<T> {\n    T,\n    U(T),\n}\npub fn unit() -> Unit<u8> {\n    Unit::T\n}\npub fn wrapped(x: u8) -> Unit<u8> {\n    Unit::U(x)\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "variant-like-const-parameter",
        items: &[
            "pub enum Count<const N: usize> {\n    N,\n}\npub fn count() -> Count<3> {\n    Count::N\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "parameter-shadows-outer-type",
        items: &[
            "pub struct Leaf;\npub enum Shadow<Leaf> {\n    Leaf(Leaf),\n}\npub fn shadow(x: u8) -> Shadow<u8> {\n    Shadow::Leaf(x)\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "associated-like-parameter",
        items: &["pub trait Tr<T> {\n    type T;\n    fn get(&self, value: T) -> Self::T;\n}\n"],
        rejected: None,
    },
    Case {
        name: "impl-associated-like-parameter",
        items: &[
            KEEP,
            "impl<T> Keep<T> for Holder<T> {\n    type T = ();\n    fn keep(&self, value: T) -> T {\n        value\n    }\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "impl-parameter-not-member",
        items: &[
            KEEP,
            "impl<T> Keep<T> for Holder<T> {\n    type T = ();\n    fn keep(&self, value: T) -> T {\n        ()\n    }\n}\n",
        ],
        rejected: Some("incompatible types"),
    },
    Case {
        name: "default-names-later-parameter",
        items: &["pub enum Late<T = U, U = ()> {\n    T(T),\n    U(U),\n}\n"],
        rejected: Some("generic default references a parameter not yet declared"),
    },
    Case {
        name: "duplicate-variant",
        items: &["pub enum Twice {\n    A,\n    A,\n}\n"],
        rejected: Some("duplicate type name `A`"),
    },
    Case {
        name: "duplicate-parameter",
        items: &["pub enum Two<T, T> {\n    A(T),\n}\n"],
        rejected: Some("duplicate type name `T`"),
    },
];

const STAGED: &str =
    "#![feature(staged_api)]\n#![stable(feature = \"fixture_root\", since = \"1.0.0\")]\n";
const STABLE: &str = "#[stable(feature = \"fixture_root\", since = \"1.0.0\")]\n";
const STAGED_CASES: [Case; 4] = [
    Case {
        name: "staged-variant-like-parameter",
        items: &[
            STAGED,
            STABLE,
            FORCE_RESULT,
            STABLE,
            "pub fn leaf(x: u8) -> ForceResult<u8, ()> {\n    ForceResult::Leaf(x)\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "staged-unstable-variant-like-parameter",
        items: &[
            STAGED,
            "#[unstable(feature = \"fixture_force\", issue = \"none\")]\n",
            FORCE_RESULT,
        ],
        rejected: None,
    },
    Case {
        name: "staged-associated-like-parameter",
        items: &[
            STAGED,
            STABLE,
            "pub trait Tr<T> {\n    #[stable(feature = \"fixture_root\", since = \"1.0.0\")]\n    type T;\n    #[stable(feature = \"fixture_root\", since = \"1.0.0\")]\n    fn get(&self, value: T) -> Self::T;\n}\n",
        ],
        rejected: None,
    },
    Case {
        name: "staged-unmarked-variant-like-parameter",
        items: &[STAGED, FORCE_RESULT],
        rejected: Some("public declaration has missing library stability attribute"),
    },
];

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

/// Type every case, after `core` as a source core or as an ordinary crate;
/// the cases that differ from their declared outcome.
fn check(folder: &str, cases: &[Case], core: Option<&str>) -> Result<Vec<String>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/separated-parameters")
        .join(std::process::id().to_string())
        .join(folder);
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in cases {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(
            &path,
            format!("{}{}", core.unwrap_or_default(), case.items.concat()),
        )?;
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command.args(["--emit-types", "--edition=2024", "--cap-lints=allow"]);
        if core.is_some() {
            command
                .args(["--crate-name", "core", "--crate-disambiguator"])
                .args(["separated", "--source-core", "core@separated"]);
        }
        let output = command.arg(&path).output()?;
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
    Ok(failures)
}

/// Lexical names inside an item find its parameters and qualified names its
/// members, so a shared name is no collision; a repeated variant or parameter
/// and a default that names a later parameter still are.
#[test]
fn parameters_and_members_of_one_item_may_share_a_name() -> Result {
    let failures = check("source-core", &CASES, Some(CORE))?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// A staged library's availability passes through the rib that separates a
/// parameter from a member of the same name, and still requires every public
/// declaration's stability.
#[test]
fn staged_parameters_and_members_inherit_availability_through_their_rib() -> Result {
    let failures = check("staged", &STAGED_CASES, None)?;
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
