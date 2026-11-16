//! `#[lang = "owned_box"]` makes its struct the built-in owning pointer, as
//! rustc 1.98.0 does for alloc's `Box<T, A>(Unique<T>, A)`. `*b` is a place of
//! its own behind the pointer the box's leading fields hold: a field access
//! steps through it without a `Deref` call, a move out of it leaves the box to
//! free its allocation without dropping the content, and the content can be
//! assigned again. Dropping a box drops whatever content remains, then runs
//! the box's own `Drop`, then drops its fields.
//!
//! The `owned-boxes` packet is a mini core and mini alloc whose `Box` keeps
//! alloc's field shape and allocates and frees through a C observer, which
//! prints every allocation, free and drop. The `pattern-*` programs hold the
//! content pointer as core does, in `*const T is !null`, which converts to a
//! trait object or slice pointer through its own `CoerceUnsized`. Outcomes
//! were declared before either compiler ran; rustc 1.98.0's are recorded in
//! its `outcomes.json`.
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const PACKET: &str = "pa31/tests/spec/owned-boxes";
/// Programs rustc accepts, with what rustc's build of each prints.
const ACCEPTED: [(&str, &str); 12] = [
    ("move-out", "alloc 1\ndrop 7\nfree\nvalue 7\n"),
    ("drop-whole", "alloc 1\ndrop 5\nfree\nvalue 5\n"),
    ("partial-move", "alloc 2\ndrop 1\ndrop 2\nfree\nvalue 1\n"),
    ("reinitialize", "alloc 1\ndrop 1\ndrop 2\nfree\nvalue 2\n"),
    (
        "nested",
        "alloc 24\nalloc 24\ndrop 1\ndrop 2\nfree\nfree\nvalue 1\n",
    ),
    ("method-move", "alloc 2\ndrop 2\nfree\ndrop 1\nvalue 1\n"),
    ("slice-content", "alloc 2\ndrop 3\ndrop 6\nfree\nvalue 9\n"),
    ("borrowed-content", "alloc 1\ndrop 8\nfree\nvalue 8\n"),
    ("dyn-content", "alloc 1\ndrop 4\nfree\nvalue 4\n"),
    ("pattern-move", "alloc 1\ndrop 7\nfree\nvalue 7\n"),
    ("pattern-dyn", "alloc 1\ndrop 4\nfree\nvalue 4\n"),
    ("pattern-slice", "alloc 2\ndrop 3\ndrop 6\nfree\nvalue 9\n"),
];
/// Programs rustc rejects, with the message `RustGM` rejects each with.
const REJECTED: [(&str, &str); 6] = [
    ("use-after-move", MOVED),
    (
        "move-from-borrowed-box",
        "cannot move a non-Copy value out of a reference",
    ),
    ("use-whole-after-partial", MOVED),
    (
        "move-from-drop-content",
        "cannot move a non-Copy field out of a type that implements Drop",
    ),
    ("immutable-box", "cannot assign through an immutable local"),
    (
        "unimplemented-dyn",
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
];
const MOVED: &str = "local may be uninitialized or moved at this read";

fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(
        &crate::resolve::fixture(&format!("{PACKET}/{name}.rs"))?,
        "owned_boxes",
        &config,
    )?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}

/// Every accepted program lowers, and every control is rejected with its
/// own message.
#[test]
fn owned_boxes_lower_and_their_controls_reject() {
    let mut failures = Vec::new();
    for (name, _) in ACCEPTED {
        if let Err(error) = program(name) {
            failures.push(format!("{name}: {error}"));
        }
    }
    for (name, message) in REJECTED {
        match program(name) {
            Ok(_) => failures.push(format!("{name}: accepted")),
            Err(error) if !error.to_string().contains(message) => {
                failures.push(format!("{name}: {error}"));
            }
            Err(_) => {}
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

/// Each accepted program prints exactly what rustc's build of it prints.
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn owned_boxes_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for (name, expected) in ACCEPTED {
        crate::observed_program::execute(
            &root.join(format!("obj/parallel/owned-boxes/native/{name}")),
            &program(name)?,
            &root.join(PACKET).join("observe.c"),
            expected.as_bytes(),
        )
        .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
