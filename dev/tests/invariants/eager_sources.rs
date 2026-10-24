//! Real source invocations expose parser and budget dimensions for both eager roles.
use super::*;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn actual_byte_width_and_repeated_invocations_keep_one_argument_parse_each() -> Result {
    for kind in ["width", "uses"] {
        for width in [8_usize, 32, 128] {
            let source = crate::test_source::fixture(&format!(
                "pa31/tests/spec/source-concat-bytes/{kind}-{width}.rs"
            ))?;
            let report = crate::test_source::source_provider(&source, "eager_bytes_work")?;
            let invocations = if kind == "uses" { width } else { 1 };
            assert_eq!(report.invocations.len(), invocations);
            assert!(report.invocations.iter().all(|call| call.parser_calls == 1));
            let source = report.package.source_core().ok_or("source")?;
            let (work, payload, _) = source.macros.usage();
            assert!(work < 1000 + 256 * width, "{kind}/{width}: {work}");
            assert!(payload < 65536 + 8192 * width, "{kind}/{width}: {payload}");
            eprintln!(
                "byte source {kind}={width} actual-invocations={invocations} parser-calls={invocations} work={work} payload={payload}"
            );
        }
    }
    Ok(())
}
#[test]
fn ordinary_source_concat_keeps_the_shared_budgeted_eager_fast_path_absent() -> Result {
    let mut previous = None;
    for width in [8_usize, 32, 128] {
        let operands = std::iter::repeat_n("\"A\"", width)
            .collect::<Vec<_>>()
            .join(",");
        let source = format!(
            "#![feature(no_core,rustc_attrs)]#![no_core]#[rustc_builtin_macro]macro_rules!concat{{($($t:tt)*)=>{{}};}}fn f(){{let _=concat!({operands});}}"
        );
        let report = crate::test_source::source_provider(&source, "ordinary_eager_work")?;
        assert_eq!(report.invocations.len(), 1);
        let source = report.package.source_core().ok_or("source")?;
        let (work, payload, _) = source.macros.usage();
        if let Some(before) = previous {
            assert_eq!((work, payload), before);
        }
        previous = Some((work, payload));
        assert!(!Builtin::Concat.source_eager());
        eprintln!("ordinary concat width={width} source-budget-work={work} payload={payload}");
    }
    Ok(())
}
#[test]
fn source_eager_role_addition_keeps_actual_hot_and_definition_layouts() {
    eprintln!(
        "byte layouts Builtin={} Body={} Definition={} SyntaxKind={} Node={} SourceCore={} Package={}",
        size_of::<Builtin>(),
        size_of::<Body>(),
        size_of::<super::super::Definition>(),
        size_of::<crate::syntax::ast::Kind>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::resolve::bindings::SourceCore>(),
        size_of::<super::super::Package>()
    );
    assert_eq!(size_of::<Builtin>(), 1);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Value>(), 24);
    assert_eq!(size_of::<super::super::Definition>(), 136);
}
