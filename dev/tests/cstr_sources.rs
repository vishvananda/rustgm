//! Actual CStr declarations, source methods and typed constant storage.
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
    for name in ["native", "local-storage-v2"] {
        let program = program(name)?;
        assert!(
            program.symbols.iter().any(|symbol| symbol.body.is_some()),
            "{name}"
        );
    }
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
        ("local-pointer", &["type", "mismatch", "mutable"][..]),
        ("local-const", &["const"][..]),
        ("local-reinterpret", &["type", "mismatch"][..]),
        ("local-nul", &["null", "NUL", "zero", "c-string"][..]),
        ("local-safe", &["unsafe"][..]),
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
    for name in ["source", "native", "local-storage-v2"] {
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
