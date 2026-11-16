//! Bring-up guest crates resolve as crates of their own inside one session.
//!
//! Every outcome was declared before either compiler ran. `rustc:` names what
//! rustc 1.98.0 does with the same sources as real separate crates (the host
//! as `--crate-type=lib --crate-name core`, each guest with `--extern` for the
//! host and the guests before it); `rustc: no analog` marks a driver-only
//! case.
use std::{
    path::{Path, PathBuf},
    process::{Command, Output},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// A minimal core. Its `Marker` is a unit struct and its root has names that
/// no guest declares, so a guest path that reached this root would not type.
const HOST: &str = r#"#![feature(no_core, lang_items, rustc_attrs)]
#![no_core]
#![allow(internal_features, dead_code, unused_variables, unused_macros)]
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
impl Clone for u8 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u8 {}
impl Clone for i32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for i32 {}
pub struct Marker;
impl Marker {
    pub(crate) const HIDDEN: usize = 1;
}
pub(crate) struct Hidden;
pub type Alias = Hidden;
pub const fn size<T>() -> usize {
    1
}
pub fn host_value() -> u8 {
    7
}
pub(crate) fn host_private() -> u8 {
    1
}
pub fn from_prelude() -> u8 {
    9
}
#[macro_export]
macro_rules! host_seven {
    () => {
        $crate::host_value()
    };
}
macro_rules! host_only {
    () => {
        0u8
    };
}
pub mod prelude {
    pub mod rust_2024 {
        pub use crate::{Clone, Copy, Sized, from_prelude};
    }
}
"#;

/// The first guest: its own `crate` root, restricted visibility, an exported
/// macro whose `$crate` names this crate, and every way to name the host.
const FIRST: &str = r"#![feature(no_core)]
#![no_core]
#![allow(dead_code)]
extern crate core as base;
pub struct Marker(pub u8);
pub(crate) fn hidden() -> u8 {
    5
}
pub mod inner {
    pub(in crate::inner) fn deep() -> u8 {
        crate::hidden()
    }
    pub fn through() -> u8 {
        self::deep()
    }
    pub(crate) fn sibling() -> u8 {
        super::hidden()
    }
}
pub fn own_root() -> u8 {
    crate::Marker(crate::inner::sibling()).0
}
#[macro_export]
macro_rules! first_marker {
    ($value:expr) => {
        $crate::Marker($value)
    };
}
pub fn exported_here() -> u8 {
    crate::first_marker!(3).0
}
pub fn host_macro() -> u8 {
    core::host_seven!()
}
pub fn host_path() -> u8 {
    core::host_value()
}
pub fn host_absolute() -> u8 {
    ::core::host_value()
}
pub fn host_alias() -> u8 {
    base::host_value()
}
pub fn same() -> u8 {
    1
}
";

/// The program: a second guest that depends on the first and on the host.
/// Its own `Marker` is a unit struct, so the first guest's `$crate::Marker(..)`
/// types only when `$crate` names the first guest.
const APP: &str = r#"#![feature(no_core)]
#![no_core]
#![no_main]
#![allow(dead_code, unused_variables)]
#[macro_use]
extern crate first;
pub struct Marker;
pub fn same() -> u8 {
    2
}
fn checks() -> u8 {
    let direct = first::Marker(1).0;
    let by_path = first::first_marker!(2).0;
    let by_macro_use = first_marker!(3).0;
    let own = crate::Marker;
    let host = core::host_seven!();
    let both = first::same();
    first::own_root();
    first::exported_here();
    first::host_macro();
    first::host_path();
    first::host_absolute();
    first::host_alias();
    first::inner::through();
    crate::same()
}
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    checks();
    0
}
"#;

/// A program with a Rust entry point instead of an exported C `main`.
const RUST_MAIN: &str = r"#![feature(no_core)]
#![no_core]
extern crate core;
fn main() {
    first::own_root();
}
";

/// A `no_std` program: the host's prelude module and its exported macros are
/// in scope without being named.
const NO_STD: &str = r#"#![no_std]
#![no_main]
fn checks() -> u8 {
    from_prelude();
    host_seven!()
}
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    checks();
    0
}
"#;

/// The same program under `no_core`: nothing of the host is in scope unnamed.
const NO_CORE_PRELUDE: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    from_prelude()
}
";
const NO_CORE_MACRO_USE: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    host_seven!()
}
";

const PRIVATE_ACROSS: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    first::hidden()
}
";
const PRIVATE_IN_PATH: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    first::inner::deep()
}
";
const HOST_PRIVATE: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    core::host_private()
}
";
const HOST_ROOT_AS_CRATE: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    crate::host_value()
}
";
const UNKNOWN_EXTERN: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
extern crate missing;
";
const UNKNOWN_PATH: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    missing::value()
}
";
const TEXTUAL_ACROSS: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    first_marker!(3).0
}
";
const UNEXPORTED_MACRO: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() -> u8 {
    core::host_only!()
}
";
/// A guest names only the crates before it.
const LATER_GUEST: &str = r"#![feature(no_core)]
#![no_core]
extern crate core;
pub fn early() -> u8 {
    app::same()
}
";
/// The later crate that the guest above may not name.
const LATER_APP: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
pub fn same() -> u8 {
    2
}
";
/// An inherent method on a primitive, as `alloc` declares for slices.
const INCOHERENT_ALLOWED: &str = r"#![feature(no_core, rustc_attrs)]
#![no_core]
#![no_main]
#![allow(internal_features, dead_code)]
extern crate core;
impl u8 {
    #[rustc_allow_incoherent_impl]
    pub fn twice(self) -> u8 {
        self
    }
}
pub fn uses() -> u8 {
    7u8.twice()
}
";
const INCOHERENT_PLAIN: &str = r"#![feature(no_core, rustc_attrs)]
#![no_core]
#![no_main]
#![allow(internal_features, dead_code)]
extern crate core;
impl u8 {
    pub fn twice(self) -> u8 {
        self
    }
}
";
/// A crate that needs an allocator, as `alloc` declares itself.
const NEEDS_ALLOCATOR: &str = r"#![feature(no_core, allocator_internals)]
#![no_core]
#![no_main]
#![needs_allocator]
#![allow(internal_features)]
extern crate core;
pub fn same() -> u8 {
    2
}
";
const UNGATED_ALLOCATOR: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
#![needs_allocator]
extern crate core;
";
const DUPLICATE_LANG: &str = r#"#![feature(no_core, lang_items)]
#![no_core]
#![no_main]
#![allow(internal_features)]
extern crate core;
#[lang = "sized"]
pub trait AlsoSized {}
"#;
/// A lang item that the host lacks belongs to the whole session.
const GUEST_LANG: &str = r#"#![feature(no_core, lang_items)]
#![no_core]
#![no_main]
#![allow(internal_features, dead_code)]
extern crate core;
#[lang = "drop"]
pub trait Drop {
    fn drop(&mut self);
}
pub struct Guard(pub u8);
impl Drop for Guard {
    fn drop(&mut self) {}
}
fn checks() {
    let guard = Guard(1);
}
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    checks();
    0
}
"#;

/// Constants typed in the guest's own crate: a `pub(crate)` associated
/// constant in a field length, a nested function's result length and body
/// lengths.
const LENGTHS: &str = r#"#![feature(no_core)]
#![no_core]
#![no_main]
#![allow(dead_code, unused_variables)]
extern crate core;
pub(crate) struct Group(u8);
impl Group {
    pub(crate) const WIDTH: usize = 2;
}
struct Aligned([u8; Group::WIDTH]);
pub mod inner {
    pub(crate) fn tags() -> [u8; crate::Group::WIDTH] {
        [0; crate::Group::WIDTH]
    }
}
fn checks() -> u8 {
    let tags: [u8; Group::WIDTH] = [0; Group::WIDTH];
    let aligned = Aligned(inner::tags());
    0
}
#[unsafe(no_mangle)]
pub extern "C" fn main() -> i32 {
    checks();
    0
}
"#;
/// A guest constant naming the host's `pub(crate)` associated constant, in
/// a body and in a field length.
const HIDDEN_BODY: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() {
    let tags = [0u8; core::Marker::HIDDEN];
}
";
const HIDDEN_FIELD: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
struct Shape([u8; core::Marker::HIDDEN]);
";
/// A guest body length naming the host's `pub(crate)` type through its
/// public alias.
const HIDDEN_TYPE: &str = r"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
fn checks() {
    let tags = [0u8; core::size::<core::Alias>()];
}
";
/// A host that requests `staged_api` and is stable, as the real core is.
const STAGED_HOST: &str = r#"#![feature(no_core, lang_items, rustc_attrs, staged_api)]
#![no_core]
#![stable(feature = "probe_core", since = "1.0.0")]
#![allow(internal_features)]
#[lang = "pointee_sized"]
#[stable(feature = "probe_core", since = "1.0.0")]
pub trait PointeeSized {}
#[lang = "meta_sized"]
#[stable(feature = "probe_core", since = "1.0.0")]
pub trait MetaSized: PointeeSized {}
#[lang = "sized"]
#[stable(feature = "probe_core", since = "1.0.0")]
pub trait Sized: MetaSized {}
"#;
/// A guest that requests `staged_api`, with a deprecation that names no
/// release.
const STAGED_UNSPECIFIED: &str = r#"#![feature(no_core, staged_api)]
#![no_core]
#![stable(feature = "staged_probe", since = "1.0.0")]
extern crate core;
#[stable(feature = "staged_probe", since = "1.0.0")]
pub fn staged() -> u8 {
    1
}
#[stable(feature = "staged_probe", since = "1.0.0")]
#[deprecated(note = "use staged")]
pub fn old() -> u8 {
    1
}
"#;
/// A guest without `staged_api`, whose deprecations need no release.
const PLAIN_DEPRECATIONS: &str = r#"#![feature(no_core)]
#![no_core]
#![no_main]
extern crate core;
#[deprecated(note = "use current")]
pub fn old() -> u8 {
    1
}
#[deprecated]
pub fn bare() -> u8 {
    1
}
pub fn current() -> u8 {
    2
}
"#;

const IDENTITY: &str = "guests_bringup";

/// Compile `HOST` with the named guests, in order, as one bring-up session.
fn session(name: &str, guests: &[(&str, &str)]) -> Result<Output> {
    hosted_session(HOST, name, guests)
}
/// As [`session`], with `host` as the host's source.
fn hosted_session(host: &str, name: &str, guests: &[(&str, &str)]) -> Result<Output> {
    let (folder, mut command) = prepare(name, guests)?;
    std::fs::write(folder.join("core.rs"), host)?;
    let list = folder.join("skip.tsv");
    std::fs::write(&list, "")?;
    command.arg("--bringup-skip-list").arg(list);
    Ok(command.arg(folder.join("core.rs")).output()?)
}
fn prepare(name: &str, guests: &[(&str, &str)]) -> Result<(PathBuf, Command)> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/guest-crates")
        .join(format!("{name}-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("core.rs"), HOST)?;
    let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
    command
        .args(["--emit-lowir", "--cap-lints=allow", "--edition=2024"])
        .args(["--crate-name", "core", "--crate-disambiguator", IDENTITY])
        .args(["--source-core", &format!("core@{IDENTITY}")]);
    for (guest, source) in guests {
        let root = folder.join(format!("{guest}.rs"));
        std::fs::write(&root, source)?;
        command
            .arg("--bringup-guest")
            .arg(format!("{guest}={}", root.display()));
    }
    Ok((folder, command))
}
fn accepted(output: &Output) -> Result<&str> {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(std::str::from_utf8(&output.stdout)?)
}
/// The session must stop at `construct` of its `file`th source (the host is
/// file 0, the guests follow in order) with `message`.
fn rejected(
    output: &Output,
    (file, source): (usize, &str),
    construct: &str,
    message: &str,
) -> Result {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let offset = source
        .find(construct)
        .ok_or_else(|| format!("the source has no `{construct}`"))?;
    let expected = format!("ERROR: file {file}: {offset}: {message}");
    assert!(stderr.contains(&expected), "{expected}\n{stderr}");
    Ok(())
}

/// rustc: accepts `LENGTHS`; rejects `HIDDEN_BODY` and `HIDDEN_FIELD` with
/// E0624 (associated constant `HIDDEN` is private) and `HIDDEN_TYPE` with
/// "type `core::Hidden` is private".
#[test]
fn guest_constants_are_typed_in_their_own_crate() -> Result {
    accepted(&session("constant-lengths", &[("app", LENGTHS)])?)?;
    for (name, program, site, message) in [
        (
            "hidden-body",
            HIDDEN_BODY,
            "core::Marker::HIDDEN",
            "inherent method is private",
        ),
        (
            "hidden-field",
            HIDDEN_FIELD,
            "core::Marker::HIDDEN",
            "inherent method is private",
        ),
        (
            "hidden-type",
            HIDDEN_TYPE,
            "core::Alias",
            "semantic type or associated item is private",
        ),
    ] {
        rejected(
            &session(name, &[("app", program)])?,
            (1, program),
            site,
            message,
        )?;
    }
    Ok(())
}

/// rustc: accepts `plain` after the staged `core` and after `HOST`; rejects
/// the staged deprecation without `since` with E0542 (missing 'since').
#[test]
fn deprecations_follow_the_staged_rules_of_their_own_crate() -> Result {
    let plain = [("plain", PLAIN_DEPRECATIONS)];
    accepted(&hosted_session(STAGED_HOST, "deprecations", &plain)?)?;
    accepted(&session("plain-deprecations", &plain)?)?;
    rejected(
        &hosted_session(
            STAGED_HOST,
            "staged-deprecations",
            &[("staged", STAGED_UNSPECIFIED)],
        )?,
        (1, STAGED_UNSPECIFIED),
        "#[deprecated(note",
        "staged deprecation requires its since field",
    )
}

/// rustc: accepts `core`, `first` and `app` as three crates.
#[test]
fn each_guest_resolves_its_own_root_macros_and_dependencies() -> Result {
    let output = session("positive", &[("first", FIRST), ("app", APP)])?;
    let lowir = accepted(&output)?;
    // The program's exported C `main` is the executable's entry point.
    assert!(lowir.contains("function @main"), "{lowir}");
    Ok(())
}

/// rustc: accepts; a binary crate's `fn main` is its entry point.
#[test]
fn a_guest_program_may_supply_a_rust_entry_point() -> Result {
    let output = session("rust-main", &[("first", FIRST), ("app", RUST_MAIN)])?;
    accepted(&output)?;
    Ok(())
}

/// rustc: rejects both `no_core` programs (E0425 cannot find function, cannot
/// find macro). `NO_STD` was declared accepted, but rustc cannot show it over
/// a minimal core: its injected `core` is the toolchain's own, beside the
/// `--extern` one (E0152). The `no_std` consumers of real core cover it.
#[test]
fn the_root_attributes_select_the_prelude_and_macro_use_crate() -> Result {
    accepted(&session("no-std", &[("app", NO_STD)])?)?;
    rejected(
        &session("no-core-prelude", &[("app", NO_CORE_PRELUDE)])?,
        (1, NO_CORE_PRELUDE),
        "from_prelude()",
        "unresolved value name",
    )?;
    rejected(
        &session("no-core-macros", &[("app", NO_CORE_MACRO_USE)])?,
        (1, NO_CORE_MACRO_USE),
        "host_seven!()",
        "unresolved macro",
    )
}

/// rustc: rejects each program with E0603 (private function).
#[test]
fn restricted_visibility_stops_at_the_crate_that_declares_it() -> Result {
    for (name, program, path, item) in [
        (
            "private-across",
            PRIVATE_ACROSS,
            "first::hidden()",
            "hidden",
        ),
        (
            "private-in-path",
            PRIVATE_IN_PATH,
            "first::inner::deep()",
            "deep",
        ),
        (
            "host-private",
            HOST_PRIVATE,
            "core::host_private()",
            "host_private",
        ),
    ] {
        rejected(
            &session(name, &[("first", FIRST), ("app", program)])?,
            (2, program),
            path,
            &format!("private value name `{item}`"),
        )?;
    }
    Ok(())
}

/// rustc: rejects with E0425 (`host_value` is not in the program's root),
/// E0463 (can't find crate for `missing`), E0433 (unresolved `missing`) and
/// E0433 (`app` is not a crate that `first` can name).
#[test]
fn a_guest_names_only_its_own_root_the_host_and_earlier_guests() -> Result {
    rejected(
        &session("host-root", &[("app", HOST_ROOT_AS_CRATE)])?,
        (1, HOST_ROOT_AS_CRATE),
        "crate::host_value()",
        "unresolved value name",
    )?;
    rejected(
        &session("unknown-extern", &[("app", UNKNOWN_EXTERN)])?,
        (1, UNKNOWN_EXTERN),
        "extern crate missing;",
        "can't find crate for `missing`",
    )?;
    rejected(
        &session("unknown-path", &[("app", UNKNOWN_PATH)])?,
        (1, UNKNOWN_PATH),
        "missing::value()",
        "unresolved value name",
    )?;
    rejected(
        &session("later-guest", &[("first", LATER_GUEST), ("app", LATER_APP)])?,
        (1, LATER_GUEST),
        "app::same()",
        "unresolved value name",
    )
}

/// rustc: rejects both (cannot find macro `first_marker` in this scope;
/// E0433 could not find `host_only` in `core`).
#[test]
fn macros_cross_a_crate_boundary_only_when_exported_and_named() -> Result {
    rejected(
        &session("textual", &[("first", FIRST), ("app", TEXTUAL_ACROSS)])?,
        (2, TEXTUAL_ACROSS),
        "first_marker!(3)",
        "unresolved macro",
    )?;
    rejected(
        &session("unexported", &[("app", UNEXPORTED_MACRO)])?,
        (1, UNEXPORTED_MACRO),
        "core::host_only!()",
        "unresolved macro",
    )
}

/// rustc: rejects the duplicate with E0152 and accepts the guest-only item.
#[test]
fn lang_items_belong_to_the_session() -> Result {
    rejected(
        &session("duplicate-lang", &[("app", DUPLICATE_LANG)])?,
        (1, DUPLICATE_LANG),
        "pub trait AlsoSized",
        "duplicate source language item",
    )?;
    accepted(&session("guest-lang", &[("app", GUEST_LANG)])?)?;
    Ok(())
}

/// rustc: accepts the method that carries the allowance and rejects the plain
/// impl with E0390 (inherent impl for a primitive outside `core`).
#[test]
fn a_guest_extends_a_primitive_only_through_the_incoherent_allowance() -> Result {
    accepted(&session(
        "incoherent-allowed",
        &[("app", INCOHERENT_ALLOWED)],
    )?)?;
    rejected(
        &session("incoherent-plain", &[("app", INCOHERENT_PLAIN)])?,
        (1, INCOHERENT_PLAIN),
        "u8 {",
        "inherent impl requires a local nominal type",
    )
}

/// rustc (the guest as `--crate-type=bin`): rejects the gated crate, a
/// program that needs an allocator and has no `#[global_allocator]`, with
/// "no global memory allocator found but one is required; ...", and the
/// ungated one with E0658 at `#![needs_allocator]`. The program gets the
/// allocator shim; `global_allocators.rs` runs programs that have one.
#[test]
fn a_guest_program_that_needs_an_allocator_must_have_one() -> Result {
    let output = session("needs-allocator", &[("app", NEEDS_ALLOCATOR)])?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let missing = "no global memory allocator found but one is required; link to std or add `#[global_allocator]` to a static item that implements the GlobalAlloc trait";
    assert!(stderr.contains(missing), "{stderr}");
    rejected(
        &session("ungated-allocator", &[("app", UNGATED_ALLOCATOR)])?,
        (1, UNGATED_ALLOCATOR),
        "#![needs_allocator]",
        "`#![needs_allocator]` requires `#![feature(allocator_internals)]`",
    )
}

/// rustc: no analog. Guest crates exist only in the bring-up profile, over a
/// source core, with distinct plain names and existing root sources.
#[test]
fn guest_options_are_refused_outside_their_profile() -> Result {
    let (folder, mut command) = prepare("no-profile", &[("app", APP)])?;
    let strict = command.arg(folder.join("core.rs")).output()?;
    assert_eq!(strict.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&strict.stderr).contains("requires the bring-up profile"));
    let usage = |arguments: &[&str]| -> Result<String> {
        let (folder, mut command) = prepare("usage", &[])?;
        let list = folder.join("skip.tsv");
        std::fs::write(&list, "")?;
        command.arg("--bringup-skip-list").arg(list).args(arguments);
        let output = command.arg(folder.join("core.rs")).output()?;
        assert_ne!(output.status.code(), Some(0));
        Ok(String::from_utf8(output.stderr)?)
    };
    assert!(usage(&["--bringup-guest", "app"])?.contains("requires NAME=ROOT.rs"));
    assert!(usage(&["--bringup-guest", "a-b=x.rs"])?.contains("plain identifier"));
    assert!(usage(&["--bringup-guest", "crate=x.rs"])?.contains("plain identifier"));
    assert!(usage(&["--bringup-guest", "app=absent.rs"])?.contains("has no root source"));
    let (folder, _) = prepare("usage", &[("app", APP), ("first", FIRST)])?;
    let root = format!("app={}", folder.join("app.rs").display());
    let again = format!("app={}", folder.join("first.rs").display());
    let repeated = usage(&["--bringup-guest", &root, "--bringup-guest", &again])?;
    assert!(repeated.contains("repeats the name"), "{repeated}");
    let shared = usage(&["--bringup-guest", &root, "--bringup-guest", &root])?;
    assert!(shared.contains("shares its root source"), "{shared}");
    let host = format!("core={}", folder.join("app.rs").display());
    assert!(usage(&["--bringup-guest", &host])?.contains("repeats the name"));
    Ok(())
}

/// rustc: no analog. A guest function's skip-list key names its crate, and
/// two crates may define the same path without sharing a symbol.
#[test]
fn guest_functions_have_their_own_census_keys_and_symbols() -> Result {
    let offset = FIRST
        .find("pub fn same")
        .ok_or("the first guest has no `same` function")?;
    let guests = [("first", FIRST), ("app", APP)];
    let (folder, mut command) = prepare("census", &guests)?;
    let list = folder.join("skip.tsv");
    // The host has no such row; only the guest's key selects the function.
    std::fs::write(&list, format!("first:first.rs\t{offset}\tsame\n"))?;
    command.arg("--bringup-skip-list").arg(&list);
    let stubbed = command.arg(folder.join("core.rs")).output()?;
    let stubbed = accepted(&stubbed)?;
    let whole = session("census-whole", &guests)?;
    let whole = accepted(&whole)?;
    // The stub replaces one body: the same definitions, each named once.
    let defined = |lowir: &str| -> Vec<String> {
        let names = lowir.lines().filter_map(|line| {
            let name = line.strip_prefix("function @")?;
            Some(name.split('(').next().unwrap_or(name).to_owned())
        });
        names.collect()
    };
    let (stubs, names) = (defined(stubbed), defined(whole));
    assert_ne!(stubbed, whole);
    assert_eq!(stubs, names);
    let distinct: std::collections::BTreeSet<_> = names.iter().collect();
    assert_eq!(distinct.len(), names.len(), "{names:?}");
    std::fs::write(&list, format!("first.rs\t{offset}\tsame\n"))?;
    let (_, mut command) = prepare("census", &guests)?;
    command.arg("--bringup-skip-list").arg(&list);
    let unkeyed = command.arg(folder.join("core.rs")).output()?;
    assert_eq!(accepted(&unkeyed)?, whole);
    Ok(())
}

/// A guest program whose bodies `mistyped` and `hidden` do not type. `caller`
/// calls the first; `holds` keeps a value of the opaque type that the second
/// would define. `main` reaches `caller` only when given an argument.
const MISTYPED_APP: &str = r#"#![feature(no_core)]
#![no_core]
#![no_main]
#![allow(dead_code, unused_variables)]
extern crate core;
fn mistyped() -> u8 {
    let value: u8 = 1i32;
    value
}
fn caller(reached: bool) -> u8 {
    if reached { mistyped() } else { 0 }
}
fn hidden() -> impl core::Copy {
    let value: u8 = 2i32;
    value
}
fn holds() -> u8 {
    let held = hidden();
    3
}
#[unsafe(no_mangle)]
pub extern "C" fn main(argc: i32, argv: *const *const u8) -> i32 {
    match argc {
        1 => 0,
        2 => caller(true) as i32,
        _ => holds() as i32,
    }
}
"#;

/// Grow the session's skip list as the census driver does until it compiles:
/// each failing round names only functions that are not listed yet. Returns
/// every round's blocker rows, the session's folder and its `LowIR`.
fn census(name: &str, guests: &[(&str, &str)]) -> Result<(Vec<Vec<String>>, PathBuf, String)> {
    let mut rounds: Vec<Vec<String>> = Vec::new();
    let mut listed = String::new();
    while rounds.len() < 5 {
        let (folder, mut command) = prepare(name, guests)?;
        let list = folder.join("skip.tsv");
        std::fs::write(&list, &listed)?;
        command.arg("--bringup-skip-list").arg(list);
        let output = command.arg(folder.join("core.rs")).output()?;
        if output.status.success() {
            return Ok((rounds, folder, String::from_utf8(output.stdout)?));
        }
        let stderr = String::from_utf8(output.stderr)?;
        assert_eq!(output.status.code(), Some(1), "{stderr}");
        assert!(!stderr.contains("bringup-unattributed"), "{stderr}");
        let rows = stderr
            .lines()
            .filter_map(|line| line.strip_prefix("bringup-blocker\t"))
            .map(str::to_owned)
            .collect::<Vec<_>>();
        assert!(!rows.is_empty(), "no new function: {stderr}");
        for row in &rows {
            let key = row.splitn(3, '\t').take(2).collect::<Vec<_>>().join("\t");
            assert!(!listed.contains(&format!("{key}\t")), "{row} failed again");
            listed.push_str(row);
            listed.push('\n');
        }
        rounds.push(rows);
    }
    Err(format!("the census did not converge: {rounds:?}").into())
}

/// rustc: rejects `app` with E0308 (mismatched types) in `mistyped` and in
/// `hidden`. One round names both: each keeps its checked signature and
/// becomes a trapping stub, so `caller` and `holds` still type. The next
/// round names `holds`, whose local needs the hidden type that `hidden`
/// would define: it fails closed instead of guessing that type.
#[test]
fn guest_bodies_that_do_not_type_are_named_and_stubbed() -> Result {
    let (rounds, _, lowir) = census("mistyped", &[("app", MISTYPED_APP)])?;
    let row = |item: &str, name: &str, message: &str| -> Result<String> {
        let offset = MISTYPED_APP
            .find(item)
            .ok_or_else(|| format!("the program has no `{item}`"))?;
        Ok(format!("app:app.rs\t{offset}\t{name}\t{message}"))
    };
    let opaque = "opaque storage requires a checked defining type";
    assert_eq!(
        rounds,
        [
            vec![
                row("fn mistyped", "mistyped", "incompatible types")?,
                row("fn hidden", "hidden", "incompatible types")?,
            ],
            vec![row("fn holds", "holds", opaque)?],
        ]
    );
    // `main` and `caller` keep their bodies; the two reached stubs trap.
    let trapping = lowir
        .split("\nfunction @")
        .filter(|function| function.contains("call void @abort()"))
        .count();
    assert_eq!(trapping, 2, "{lowir}");
    assert!(lowir.contains("function @main"), "{lowir}");
    Ok(())
}

/// rustc: no analog (it rejects the program). The stubbed program returns 0
/// when `main` reaches no stub and aborts when `caller` reaches `mistyped`.
#[test]
#[ignore = "requires the pinned CPPGM backend"]
fn a_reached_guest_stub_traps_at_run_time() -> Result {
    let (_, folder, lowir) = census("mistyped-native", &[("app", MISTYPED_APP)])?;
    let source = folder.join("app.lowir");
    let binary = folder.join("app");
    std::fs::write(&source, lowir)?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("the manifest has no parent")?;
    let built = Command::new("python3")
        .arg(root.join("scripts/backend.py"))
        .arg(&source)
        .arg("-o")
        .arg(&binary)
        .output()?;
    assert!(
        built.status.success(),
        "{}",
        String::from_utf8_lossy(&built.stderr)
    );
    assert_eq!(Command::new(&binary).output()?.status.code(), Some(0));
    // The course runtime's `abort` exits with status 134, as SIGABRT would.
    let trapped = Command::new(&binary).arg("reach").output()?;
    assert_eq!(trapped.status.code(), Some(134), "{trapped:?}");
    assert!(trapped.stdout.is_empty() && trapped.stderr.is_empty());
    Ok(())
}
