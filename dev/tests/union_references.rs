//! Scalar union reference projections observe current Image bytes and validity.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "harness/reconstruction_work.rs"]
mod work;
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, &str)] = &[
    ("shared-local", 0, 0, ""),
    ("mutable-local", 0, 0, ""),
    ("alternate-aliases", 0, 0, ""),
    ("nested-product", 0, 0, ""),
    ("valid-bool", 0, 0, ""),
    (
        "invalid-bool",
        1,
        1,
        "constant union scalar read has invalid bool representation",
    ),
    ("valid-char", 0, 0, ""),
    (
        "invalid-char",
        1,
        1,
        "constant union scalar read has invalid char representation",
    ),
    (
        "uninitialized-wide",
        1,
        1,
        "constant union scalar read uses uninitialized bytes",
    ),
    ("initialized-small", 0, 0, ""),
    ("cross-field-init", 0, 0, ""),
    (
        "expired-demanded",
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    ("escape", 1, 1, "constant pointer refers to expired storage"),
    ("transient-call", 0, 0, ""),
    (
        "packed-unaligned",
        1,
        1,
        "constant raw shared reference is unaligned",
    ),
    ("packed-aligned", 0, 0, ""),
    ("static-shared", 0, 0, ""),
    (
        "shared-derived-mutable-boundary",
        1,
        1,
        "constant mutable raw references from shared-derived pointers are not implemented",
    ),
    (
        "direct-borrow-boundary",
        0,
        1,
        "constant union field address formation requires byte-offset provenance and is not implemented",
    ),
    (
        "pointer-family-boundary",
        0,
        1,
        "constant union images require an all-fields primitive scalar family; pointer and aggregate provenance storage is not implemented",
    ),
    ("ordinary-scalar", 0, 0, ""),
    (
        "initialize-through-mutable-boundary",
        0,
        1,
        "constant union scalar read uses uninitialized bytes",
    ),
    (
        "repair-bool-through-mutable-boundary",
        0,
        1,
        "constant union scalar read has invalid bool representation",
    ),
    ("static-demanded", 0, 0, ""),
    (
        "static-invalid-bool",
        1,
        1,
        "constant union scalar read has invalid bool representation",
    ),
    (
        "static-uninitialized-wide",
        1,
        1,
        "constant union scalar read uses uninitialized bytes",
    ),
    ("static-ordinal-aliases", 0, 0, ""),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn union_references_match_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/union-reference-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, status, reason) in CASES {
        checked::check(
            &root.join(format!("pa27/tests/spec/union-references/{name}.rs")),
            &evidence,
            name,
            (rust, status, reason),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires the qualified external CPPGM LowIR backend"]
fn union_reference_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in [
        "alternate-aliases",
        "valid-scalars",
        "static-nested-packed",
        "transient-mutable",
        "scaled-values",
    ] {
        let source = std::fs::read_to_string(
            root()?.join(format!("pa27/tests/spec/union-reference-native/{name}.rs")),
        )?;
        native::check_cases(
            "parallel/pa27/union-reference-native",
            &[(name, &source, false)],
        )?;
    }
    Ok(())
}
#[test]
fn union_reference_work_is_bounded() -> Result<(), Box<dyn std::error::Error>> {
    work::compare("parallel/pa27/union-reference-scaling", |raw, count| {
        if raw {
            format!(
                "union U{{a:u32,b:u32}}const fn f(n:u32)->u32{{let mut u=U{{a:0}};let p=&raw mut u.a;let mut i=0u32;while i<n{{unsafe{{{{let r=&mut *p;*r+=1;}}}}i+=1;}}unsafe{{u.a}}}}const X:u32=f({count});fn main(){{let _=X;}}"
            )
        } else {
            format!(
                "const fn f(n:u32)->u32{{let mut x=0u32;let p=&mut x;let mut i=0u32;while i<n{{{{let r=&mut *p;*r+=1;}}i+=1;}}x}}const X:u32=f({count});fn main(){{let _=X;}}"
            )
        }
    })
}
