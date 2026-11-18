//! String words preserve byte validity, metadata and original storage restrictions.
use crate::{checked_sources as checked, native_references as native};
use std::path::Path;
type Case = (&'static str, i32, [i32; 3], bool);
// These unchanged originals remain required0 in the assignment manifest. The
// current source-library panic macro dependency is not a negative contract.
const REQUIRED_SOURCE_BINDINGS: &[&str] = &["string-union-roundtrip", "string-raw-word-roundtrip"];
const CASES: &[Case] = &[
    ("published-string-runtime", 0, [0, 0, 0], true),
    ("utf8-empty-one-past", 0, [0, 0, 0], true),
    ("string-union-roundtrip", 0, [0, 0, 0], true),
    ("string-raw-word-roundtrip", 0, [0, 0, 0], true),
    ("string-union-roundtrip-check", 0, [0, 0, 0], true),
    ("string-raw-word-roundtrip-check", 0, [0, 0, 0], true),
    ("ordinary-string-control", 0, [0, 0, 0], true),
    ("utf8-valid-widths", 0, [0, 0, 0], true),
    ("utf8-mutable-roundtrip", 0, [0, 0, 0], true),
    ("utf8-empty-roundtrip", 1, [1, 1, 1], false),
    ("utf8-offset-view", 0, [0, 0, 0], true),
    ("raw-invalid-utf8", 0, [0, 0, 0], true),
    ("invalid-utf8-reference", 0, [0, 0, 0], false),
    ("truncated-utf8-reference", 0, [0, 0, 0], false),
    ("oversized-raw-string", 0, [0, 0, 0], true),
    ("oversized-string-reference", 1, [1, 1, 1], false),
    ("shared-mutable-string", 1, [1, 1, 1], false),
    ("expired-string-reference", 1, [1, 1, 1], false),
    ("utf8-reread-after-write", 0, [0, 0, 0], false),
    ("utf8-empty-local-cell", 0, [0, 0, 0], true),
];

#[test]
fn constant_string_ranges_keep_byte_backing_and_shared_reference_checks()
-> Result<(), Box<dyn std::error::Error>> {
    let source = crate::resolve::fixture("pa31/tests/spec/unsafe-cell-source/nullable-shapes.rs")?;
    let (roles, _) = source
        .split_once("pub enum Maybe<T>")
        .ok_or("source roles")?;
    let roles = format!(
        r#"{roles}
        #[lang="unsize"] #[rustc_deny_explicit_impl] #[rustc_dyn_incompatible_trait]
        pub trait Unsize<T:PointeeSized>:PointeeSized {{}}
        #[lang="coerce_unsized"] pub trait CoerceUnsized<T:PointeeSized>:Sized {{}}
        impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a U> for &'b T {{}}
        pub mod intrinsics {{ #[rustc_intrinsic] pub const unsafe fn transmute<Src,Dst>(src:Src)->Dst; }}
        "#
    );
    let accepted = format!(
        r"{roles}
        pub const SEPARATORS: &[&str] = &[
            unsafe {{ intrinsics::transmute::<&[u8], &str>(&[47u8]) }},
            unsafe {{ intrinsics::transmute::<&[u8], &str>(&[195u8, 169u8]) }},
            unsafe {{ intrinsics::transmute::<&[i8], &str>(&[47i8]) }},
        ];
        pub const BYTES: &[i8] = unsafe {{ intrinsics::transmute::<&[u8], &[i8]>(&[47u8]) }};
        "
    );
    let mut config = rustgm::expand::config::Config::default();
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(&accepted, "string_ranges", &config)?;
    rustgm::pipeline::analyze_source(&report)?;

    let rejected = format!("{roles} pub const BAD: &Slot<u32> = &Slot {{ value: 7u32 }};");
    let report = crate::resolve::source_provider_config(&rejected, "mutable_range", &config)?;
    let Err(error) = rustgm::pipeline::analyze_source(&report) else {
        return Err("interior mutable allocation escaped".into());
    };
    assert!(
        error
            .to_string()
            .contains("interior mutable allocation backing")
    );
    Ok(())
}

#[test]
fn string_word_source_contracts() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let out = root.join(format!(
        "obj/parallel/pa27/string-byte-words/source-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&out)?;
    for &(name, rust, views, _) in CASES {
        if REQUIRED_SOURCE_BINDINGS.contains(&name) {
            continue;
        }
        checked::check_modes(
            &root.join(format!("pa27/tests/spec/string-byte-words/{name}.rs")),
            &out,
            name,
            (rust, views, ""),
        )?;
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned Rust and CPPGM backend; literal source/value comparisons"]
fn string_word_native_values() -> Result<(), Box<dyn std::error::Error>> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let mut owned = Vec::new();
    for &(name, _, _, valid) in CASES {
        if valid && !REQUIRED_SOURCE_BINDINGS.contains(&name) {
            owned.push((
                name,
                std::fs::read_to_string(
                    root.join(format!("pa27/tests/spec/string-byte-words/{name}.rs")),
                )?,
            ));
        }
    }
    let cases: Vec<_> = owned
        .iter()
        .map(|(name, text)| (*name, text.as_str(), false))
        .collect();
    native::check_literal_cases("parallel/pa27/string-byte-words/native", &cases)
}
