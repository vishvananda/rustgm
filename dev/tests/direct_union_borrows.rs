//! Direct union borrows retain source permissions and scalar Image validity.
use crate::checked_sources as checked;
use crate::native_references as native;
use crate::reconstruction_work as work;
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, i32, &str)] = &[
    ("shared-local", 0, 0, 0, ""),
    ("mutable-local", 0, 0, 0, ""),
    ("cross-field-aliases", 0, 0, 0, ""),
    ("static-shared", 0, 0, 0, ""),
    ("generic", 0, 0, 0, ""),
    ("call-reference", 0, 0, 0, ""),
    ("copy-snapshot", 0, 0, 0, ""),
    (
        "missing-unsafe-shared",
        1,
        1,
        1,
        "union field access requires an unsafe context",
    ),
    (
        "missing-unsafe-mutable",
        1,
        1,
        1,
        "union field access requires an unsafe context",
    ),
    (
        "immutable-mutable-borrow",
        1,
        1,
        1,
        "cannot mutably borrow an immutable local",
    ),
    (
        "packed-aligned-source",
        1,
        0,
        1,
        "reference to an unaligned packed field",
    ),
    (
        "invalid-bool",
        1,
        1,
        1,
        "constant union scalar read has invalid bool representation",
    ),
    (
        "invalid-char",
        1,
        1,
        1,
        "constant union scalar read has invalid char representation",
    ),
    (
        "uninitialized-wide",
        1,
        1,
        1,
        "constant union scalar read uses uninitialized bytes",
    ),
    (
        "escape",
        1,
        1,
        1,
        "required lifetime does not outlive its target",
    ),
    (
        "partial-write-only-boundary",
        0,
        1,
        1,
        "constant union scalar read uses uninitialized bytes",
    ),
    (
        "repair-bool-boundary",
        0,
        1,
        1,
        "constant union scalar read has invalid bool representation",
    ),
    ("pointer-family-boundary", 0, 0, 0, ""),
    ("unit-family-boundary", 0, 0, 0, ""),
    ("ordinary-scalar", 0, 0, 0, ""),
    ("raw-base", 0, 0, 0, ""),
    (
        "packed-raw-base-aligned-source",
        1,
        0,
        1,
        "reference to an unaligned packed field",
    ),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn direct_union_borrows_matches_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/union-borrow-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, types, status, reason) in CASES {
        let input = root.join(format!("pa27/tests/spec/union-borrows/{name}.rs"));
        if types == status {
            checked::check(&input, &evidence, name, (rust, status, reason))?;
        } else {
            checked::check_modes(
                &input,
                &evidence,
                name,
                (rust, [types, status, status], reason),
            )?;
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn direct_union_borrows_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "local-aliases",
        "generic-call-copy",
        "static-scalars",
        "scaled-values",
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/union-borrow-native/{name}.rs")),
        )?;
        native::check_cases(
            "parallel/pa27/union-borrow-native",
            &[(name, &source, false)],
        )?;
    }
    let literal = std::fs::read_to_string(
        root()?.join("pa27/tests/spec/union-borrow-native/mixed-families.rs"),
    )?;
    native::check_literal_cases(
        "parallel/pa27/union-borrow-native",
        &[("mixed-families", &literal, false)],
    )?;
    Ok(())
}
#[test]
fn direct_union_borrows_has_bounded_work() -> Result<(), Box<dyn std::error::Error>> {
    work::compare("parallel/pa27/union-borrow-scaling", |raw, count| {
        if raw {
            format!(
                "union U{{a:u32,b:u32}}const fn f(n:u32)->u32{{let mut u=U{{a:0}};let mut i=0u32;while i<n{{unsafe{{let r=&mut u.a;*r+=1;}}i+=1;}}unsafe{{u.b}}}}const X:u32=f({count});fn main(){{let _=X;}}"
            )
        } else {
            format!(
                "const fn f(n:u32)->u32{{let mut x=0u32;let mut i=0u32;while i<n{{{{let r=&mut x;*r+=1;}}i+=1;}}x}}const X:u32=f({count});fn main(){{let _=X;}}"
            )
        }
    })
}
