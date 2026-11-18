//! Actual `CStr` declarations, source methods and typed constant storage.
use crate::resolve as source;
use rustgm::{expand::config::Config, pipeline, support::diagnostics::LintCap};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/cstr-source";
fn report(name: &str) -> Result<rustgm::resolve::expansion::Report> {
    let text = source::fixture(&format!("{PACKET}/{name}.rs"))?;
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    Ok(source::source_provider_config(
        &text,
        "actual_cstr",
        &config,
    )?)
}
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let report = report(name)?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
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
#[test]
fn source_nominal_and_unused_alternative_headers_retain_ordinary_legality() -> Result {
    for name in ["source", "extra", "generic", "demand"] {
        pipeline::analyze_source(&report(name)?)?;
    }
    Ok(())
}
#[test]
fn real_source_methods_and_const_static_views_lower() -> Result {
    for name in ["native", "local-storage-v2", "local-roundtrip"] {
        let program = program(name)?;
        assert!(
            program.symbols.iter().any(|symbol| symbol.body.is_some()),
            "{name}"
        );
    }
    Ok(())
}
#[test]
fn cstr_and_nominal_references_reconstruct_from_actual_symbolic_byte_words() -> Result {
    program("byte-word")?;
    Ok(())
}
#[test]
fn constant_slice_casts_to_the_actual_cstr_and_back() -> Result {
    let original = source::fixture(&format!("{PACKET}/local-const.rs"))?;
    let (roles, _) = original.split_once("fn ordinary()").ok_or("source roles")?;
    let input = format!(
        r#"#![feature(const_destruct)]
        {roles}
        #[lang="unsize"] #[rustc_deny_explicit_impl] #[rustc_dyn_incompatible_trait]
        pub trait Unsize<T: PointeeSized>: PointeeSized {{}}
        #[lang="coerce_unsized"] pub trait CoerceUnsized<T:PointeeSized>:Sized {{}}
        impl<'a,'b:'a,T:PointeeSized+Unsize<U>,U:PointeeSized> CoerceUnsized<&'a U> for &'b T {{}}
        const fn convert(bytes: &[u8]) -> &ffi::CStr {{
            unsafe {{ &*(bytes as *const [u8] as *const ffi::CStr) }}
        }}
        const VALUE: &ffi::CStr = convert(b"\xFF\0");
        const VIEW: &[u8] = VALUE.to_bytes_with_nul();
        #[unsafe(no_mangle)] pub extern "C" fn bytes() -> *const u8 {{ VIEW.as_ptr() }}
        #[unsafe(no_mangle)] pub extern "C" fn chars() -> *const i8 {{ VALUE.as_ptr() }}
        "#
    );
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(&input, "constant_cstr_cast", &config)?;
    crate::provider_traits::lower_report(&report)?;
    Ok(())
}
#[test]
fn constant_signed_c_strings_pass_shared_backing_inspection() -> Result {
    let original = source::fixture(&format!("{PACKET}/local-const.rs"))?;
    let (cstr, _) = original.split_once("impl CStr {").ok_or("source CStr")?;
    let prefix = format!(
        "#![feature(const_destruct)]\n{cstr}}}\n\
        #[lang=\"unsafe_cell\"] #[repr(transparent)] pub struct Slot<T:?Sized>{{value:T}}"
    );
    let accepted = format!(
        r#"{prefix}
        const DEV_NULL: &ffi::CStr = c"/dev/null";
        const EMPTY: &ffi::CStr = c"";
        const BYTES: &[u8; 1] = b"/";
        "#
    );
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    let report = source::source_provider_config(&accepted, "signed_cstr", &config)?;
    pipeline::analyze_source(&report)?;
    let rejected = format!("{prefix} const BAD: &Slot<i8> = &Slot {{ value: 7i8 }};");
    let report = source::source_provider_config(&rejected, "mutable_backing", &config)?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("interior mutable allocation escaped")?;
    assert!(
        error
            .to_string()
            .contains("interior mutable allocation backing")
    );
    Ok(())
}
#[test]
fn ordinary_type_privacy_body_const_and_unsafe_checks_remain() -> Result {
    for (name, causes) in [
        ("privacy", &["private"][..]),
        ("body", &["type", "mismatch"][..]),
        ("wrong", &["required declaration target"][..]),
        ("repr", &["transparent"][..]),
        ("missing", &["C string literals require"][..]),
        ("local-type", &["type", "mismatch"][..]),
        ("local-mutable", &["type", "mismatch", "mutable"][..]),
        ("local-pointer", &["type", "mismatch", "mutability"][..]),
        ("local-const", &["const"][..]),
        ("local-reinterpret", &["type", "mismatch"][..]),
        ("local-nul", &["null", "NUL", "zero", "c-string"][..]),
        ("local-safe", &["unsafe"][..]),
        ("local-write", &["immutable", "read-only", "shared"][..]),
        (
            "local-overread-v2",
            &["bounds", "outside", "out of", "exceeds", "extent", "range"][..],
        ),
    ] {
        let error = program(name)
            .err()
            .ok_or_else(|| format!("accepted {name}"))?;
        let text = error.to_string();
        assert!(
            causes.iter().any(|cause| text.contains(cause)),
            "{name}: {text}"
        );
        assert!(
            !["not implemented", "budget", "exhausted", "missing layout"]
                .iter()
                .any(|fence| text.contains(fence)),
            "{name}: {text}"
        );
    }
    Ok(())
}
#[test]
fn original_nominal_and_method_intervals_are_preserved() -> Result {
    let cstr = source::fixture(&format!("{PACKET}/original-cstr.rs"))?;
    let slice = source::fixture(&format!("{PACKET}/original-slice.rs"))?;
    for name in ["source", "native", "local-storage-v2", "byte-word"] {
        let actual = source::fixture(&format!("{PACKET}/{name}.rs"))?;
        for (original, start, end) in [
            (&cstr, 3180, 4083),
            (&cstr, 18958, 19251),
            (&cstr, 22631, 23121),
            (&slice, 24530, 24839),
        ] {
            assert!(
                actual.contains(&original[start..end]),
                "{name}: {start}..{end}"
            );
        }
    }
    Ok(())
}
#[test]
fn c_string_interning_shares_ordinary_bytes_across_all_word_boundaries() -> Result {
    use rustgm::sema::text::Texts;
    for ordinary_first in [false, true] {
        let mut texts = Texts::default();
        for length in 0..=128 {
            let payload = (0..length)
                .map(|index| u8::try_from(index % 255 + 1))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            let mut expected = payload.clone();
            expected.push(0);
            let first = if ordinary_first {
                texts.intern(&expected)?
            } else {
                texts.intern_c_string(&payload)?
            };
            let byte_len = texts.byte_len();
            let count = texts.len();
            assert_eq!(texts.get(first)?, expected);
            assert_eq!(texts.intern_c_string(&payload)?, first);
            assert_eq!(texts.intern(&expected)?, first);
            assert_eq!(texts.byte_len(), byte_len);
            assert_eq!(texts.len(), count);
        }
    }
    Ok(())
}
#[test]
#[ignore = "requires the pinned CPPGM object backend and a C compiler"]
fn actual_methods_and_both_literal_storage_views_execute_natively() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root.join(format!(
        "obj/parallel/cstr-contracts/native-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    for (name, observer, expected) in [
        ("native", "observer.c", "CStr 65 2 0 195 169 255 92 1\n"),
        ("local-storage-v2", "local-storage.c", "storage 8 65 0\n"),
        ("local-roundtrip", "local-roundtrip.c", "roundtrip 255 0\n"),
        ("byte-word", "byte-word.c", "byte-word 255 0 17 23\n"),
    ] {
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program(name)?,
            &source::fixture(&format!("{PACKET}/{observer}"))?,
            expected,
        )?;
    }
    Ok(())
}
