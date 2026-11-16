//! `#[diagnostic::on_move]` (feature `diagnostic_on_move`) rewords rustc's
//! use-after-move error for a type, as alloc's `Rc` and `Arc` do; it grants
//! nothing and rejects nothing. Like `on_const` and `on_unmatched_args`, a
//! session with capped lints validates the directive's literals and defers
//! its rendering, an uncapped session that enables the feature fails closed,
//! and without the feature the directive is ignored, as rustc ignores it.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 is the
//! oracle (`RUSTC_BOOTSTRAP=1 rustc +1.98.0 --edition=2024 --crate-type=lib
//! --crate-name core`, with `--cap-lints=allow` unless a case is uncapped).
//! It accepts `move-directive`, `ungated-move-directive`,
//! `function-move-directive`, `uncapped-move-directive` and
//! `uncapped-ungated-move-directive`. It rejects `escaped-move-directive`
//! with "unknown character escape", `misspelled-move-feature` with E0635,
//! `unsafe-move-directive` as a safe attribute in an unsafe wrapper and
//! `moved-twice` with E0382. `uncapped-move-directive` is the one declared
//! divergence: the uncapped profile is not implemented, so `RustGM` rejects
//! it. The cases first ran with `--emit-types`, which stops before ownership
//! checking and so accepted `moved-twice`; they now run through
//! `--emit-ownership`, and its message for `moved-twice` was pinned after
//! that run. Every other verdict was unchanged.
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const GATED: &str = "#![feature(no_core, lang_items, rustc_attrs, diagnostic_on_move)]\n";
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
#[lang = "clone"]
pub trait Clone: Sized {
    #[lang = "clone_fn"]
    fn clone(&self) -> Self;
}
#[lang = "copy"]
pub trait Copy: Clone {}
"#;
const DIRECTIVE: &str = "#[diagnostic::on_move(\n    message = \"the type `{Self}` does not implement `Copy`\",\n    label = \"this move could be avoided by cloning the original `{Self}`\",\n    note = \"consider using `Holder::clone`\"\n)]\n";
const HOLDER: &str = "pub struct Holder<T>(T);\n";

/// One case: its crate header, the items after the core, whether lints are
/// capped, and its rejection, or `None` when it is accepted.
struct Case {
    name: &'static str,
    header: &'static str,
    items: &'static [&'static str],
    capped: bool,
    rejected: Option<&'static str>,
}
const CASES: [Case; 9] = [
    Case {
        name: "move-directive",
        header: GATED,
        items: &[DIRECTIVE, HOLDER],
        capped: true,
        rejected: None,
    },
    Case {
        name: "ungated-move-directive",
        header: UNGATED,
        items: &[DIRECTIVE, HOLDER],
        capped: true,
        rejected: None,
    },
    Case {
        name: "function-move-directive",
        header: GATED,
        items: &[DIRECTIVE, "pub fn moved() {}\n"],
        capped: true,
        rejected: None,
    },
    Case {
        name: "escaped-move-directive",
        header: GATED,
        items: &["#[diagnostic::on_move(message = \"\\q\")]\n", HOLDER],
        capped: true,
        rejected: Some("invalid escape"),
    },
    Case {
        name: "misspelled-move-feature",
        header: "#![feature(no_core, lang_items, rustc_attrs, diagnostic_on_moves)]\n",
        items: &[DIRECTIVE, HOLDER],
        capped: true,
        rejected: Some(
            "unsupported active source-core feature or undefined library feature request",
        ),
    },
    Case {
        name: "unsafe-move-directive",
        header: GATED,
        items: &[
            "#[unsafe(diagnostic::on_move(message = \"moved\"))]\n",
            HOLDER,
        ],
        capped: true,
        rejected: Some("diagnostic directive cannot use an unsafe attribute wrapper"),
    },
    Case {
        name: "moved-twice",
        header: GATED,
        items: &[
            DIRECTIVE,
            HOLDER,
            "pub fn twice(holder: Holder<u8>) {\n    let first = holder;\n    let second = holder;\n}\n",
        ],
        capped: true,
        rejected: Some(MOVED_TWICE),
    },
    Case {
        name: "uncapped-move-directive",
        header: GATED,
        items: &[DIRECTIVE, HOLDER],
        capped: false,
        rejected: Some("uncapped move diagnostic directive profile is not implemented"),
    },
    Case {
        name: "uncapped-ungated-move-directive",
        header: UNGATED,
        items: &[DIRECTIVE, HOLDER],
        capped: false,
        rejected: None,
    },
];
const MOVED_TWICE: &str = "local may be uninitialized or moved at this read";

fn rejects(stderr: &str, message: &str) -> bool {
    stderr.lines().any(|line| {
        line.strip_prefix("ERROR: file 0: ")
            .and_then(|rest| rest.split_once(": "))
            .is_some_and(|(_, actual)| actual == message)
    })
}

/// A move directive changes no verdict: a well-formed one is accepted, a
/// use after move is still rejected, and a malformed or misspelled request
/// is rejected.
#[test]
fn move_directives_reword_without_changing_any_verdict() -> Result {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/move-directives")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut failures = Vec::new();
    for case in CASES {
        let path = folder.join(format!("{}.rs", case.name));
        std::fs::write(
            &path,
            format!("{}{CORE}{}", case.header, case.items.concat()),
        )?;
        let mut command = Command::new(env!("CARGO_BIN_EXE_rustgm"));
        command.args(["--emit-ownership", "--edition=2024"]);
        if case.capped {
            command.arg("--cap-lints=allow");
        }
        let output = command
            .args(["--crate-name", "core", "--crate-disambiguator", "moves"])
            .args(["--source-core", "core@moves"])
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
