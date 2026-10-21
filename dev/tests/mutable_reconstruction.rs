//! Mutable raw reconstruction requires existing pointer authority and checked storage.
#[path = "harness/checked_sources.rs"]
mod checked;
#[path = "harness/native_references.rs"]
mod native;
#[path = "harness/reconstruction_work.rs"]
mod work;
use std::path::PathBuf;
const CASES: &[(&str, i32, i32, i32, &str)] = &[
    ("local", 0, 0, 0, ""),
    ("raw-const-cast", 0, 0, 0, ""),
    (
        "shared-derived-cast",
        0,
        1,
        1,
        "constant mutable raw references from shared-derived pointers are not implemented",
    ),
    ("original-alias", 0, 0, 0, ""),
    ("call-product-copy", 0, 0, 0, ""),
    (
        "shared-call-product",
        0,
        1,
        1,
        "constant mutable raw references from shared-derived pointers are not implemented",
    ),
    ("transient", 0, 0, 0, ""),
    (
        "escape",
        1,
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    (
        "expired",
        0,
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    (
        "null",
        0,
        1,
        1,
        "constant pointer has no allocation provenance",
    ),
    (
        "packed-field-unaligned",
        0,
        1,
        1,
        "constant raw mutable reference is unaligned",
    ),
    ("packed-field-aligned", 0, 0, 0, ""),
    (
        "packed-source-aligned",
        1,
        0,
        1,
        "reference to an unaligned packed field",
    ),
    ("packed-whole", 0, 0, 0, ""),
    ("symbolic-zst", 0, 0, 0, ""),
    (
        "reinterpret-boundary",
        0,
        1,
        1,
        "raw pointer type reinterpretation in constant execution is not implemented",
    ),
    (
        "uninitialized",
        1,
        1,
        1,
        "borrowed place may be uninitialized or moved",
    ),
    (
        "union-boundary",
        0,
        1,
        1,
        "references into constant union images are not implemented",
    ),
    (
        "static-boundary",
        0,
        1,
        1,
        "constant mutable raw references to static storage are not implemented",
    ),
    (
        "shared-derived-write",
        1,
        1,
        1,
        "constant mutable raw references from shared-derived pointers are not implemented",
    ),
    (
        "shared-derived-read",
        0,
        1,
        1,
        "constant mutable raw references from shared-derived pointers are not implemented",
    ),
    (
        "expired-demanded",
        1,
        1,
        1,
        "constant pointer refers to expired storage",
    ),
    (
        "null-demanded",
        1,
        1,
        1,
        "constant pointer has no allocation provenance",
    ),
    (
        "packed-unaligned-demanded",
        1,
        1,
        1,
        "constant raw mutable reference is unaligned",
    ),
    ("shared-reborrow-then-original", 0, 0, 0, ""),
];
fn root() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?
        .to_owned())
}
#[test]
fn mutable_reconstruction_matches_independent_objects() -> Result<(), Box<dyn std::error::Error>> {
    let root = root()?;
    let evidence = root.join(format!(
        "obj/parallel/pa27/mutable-reference-source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&evidence)?;
    for &(name, rust, types, status, reason) in CASES {
        let input = root.join(format!("pa27/tests/spec/raw-mutable-reference/{name}.rs"));
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
fn mutable_reconstruction_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for name in ["local-and-casts", "calls-and-aliases", "packed-and-zst"] {
        let source = std::fs::read_to_string(root()?.join(format!(
            "pa27/tests/spec/mutable-reference-native/{name}.rs"
        )))?;
        native::check_cases(
            "parallel/pa27/mutable-reference-native",
            &[(name, &source, false)],
        )?;
    }
    Ok(())
}
#[test]
fn mutable_reconstruction_has_bounded_work() -> Result<(), Box<dyn std::error::Error>> {
    work::compare("parallel/pa27/mutable-reference-scaling", |raw, count| {
        let pointer = if raw {
            "let p=&raw mut x;"
        } else {
            "let p=&mut x;"
        };
        format!(
            "const fn f(n:u32)->u32{{let mut x=0u32;{pointer}let mut i=0u32;while i<n{{unsafe{{let r=&mut *p;*r+=1;}}i+=1;}}x}}const X:u32=f({count});fn main(){{let _=X;}}"
        )
    })
}
