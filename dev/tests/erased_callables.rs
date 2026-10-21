//! Object callable context supplies signatures while ordinary proofs retain authority.
#[path = "harness/checked_sources.rs"]
mod checked_sources;
use std::path::PathBuf;

const CASES: &[(&str, i32, i32, &str)] = &[
    ("shared", 0, 0, ""),
    ("mutable", 0, 0, ""),
    ("function", 0, 0, ""),
    ("pointer", 0, 0, ""),
    ("binder", 0, 0, ""),
    ("generic", 0, 0, ""),
    (
        "mutable-as-shared",
        1,
        1,
        "trait obligation requires an unambiguous matching assumption or implementation",
    ),
    (
        "once-borrowed",
        0,
        1,
        "cannot move a non-Copy value out of a reference",
    ),
    (
        "local-output",
        0,
        1,
        "required lifetime does not outlive its target",
    ),
    ("contextual", 0, 0, ""),
    ("captured-output", 0, 0, ""),
    ("once-object", 0, 0, ""),
    ("generic-consumer", 0, 0, ""),
    ("upcast", 0, 0, ""),
    (
        "wrong-input",
        1,
        1,
        "associated type requires an unambiguous validated implementation",
    ),
    ("wrong-output", 1, 1, "incompatible types"),
    (
        "unsafe-function",
        1,
        1,
        "associated type requires an unambiguous validated implementation",
    ),
    (
        "shared-mutable",
        0,
        1,
        "cannot mutably borrow shared or static storage",
    ),
    (
        "output-binder",
        1,
        1,
        "call output lifetime is unconstrained by the trait inputs",
    ),
    (
        "escape",
        0,
        1,
        "required lifetime does not outlive its target",
    ),
    (
        "contextual-crossed",
        0,
        1,
        "required lifetime does not outlive its target",
    ),
    (
        "borrow-conflict",
        0,
        1,
        "access conflicts with a live reference loan",
    ),
    ("inherited-annotated", 0, 0, ""),
];
#[test]
fn declared_object_callable_semantics() -> Result<(), Box<dyn std::error::Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let evidence = root.join(format!(
        "../obj/parallel/pa24/erased-callables/semantics/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, types, constants, reason) in CASES {
        let source = root.join(format!("../pa24/tests/spec/erased-callables/{name}.rs"));
        if constants != 0 {
            if types == constants {
                checked_sources::check(&source, &evidence, name, (constants, constants, reason))?;
            } else {
                checked_sources::check_modes(
                    &source,
                    &evidence,
                    name,
                    (constants, [types, constants, constants], reason),
                )?;
            }
            continue;
        }
        checked_sources::check_views(
            &source,
            &evidence,
            name,
            constants,
            &[("--emit-types", types), ("--emit-constants", constants)],
            reason,
        )?;
    }
    Ok(())
}
