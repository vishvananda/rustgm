//! Pinned concat syntax, actual generated-name lookup and source provenance.
use crate::{
    expand::{
        config::Config,
        macros::{model, session, transcribe},
        stream::{Environment, OriginKind},
    },
    resolve::expansion::Report,
    support::edition::Edition,
    test_source,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "pa31/tests/spec/metavariable-concat";
fn fixture(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!("{ROOT}/{name}.rs"))?)
}
fn report(source: &str, edition: Edition) -> Result<Report> {
    Ok(test_source::source_provider_config(
        source,
        "concat_protocol",
        &Config::in_edition(edition),
    )?)
}
fn check(source: &str, edition: Edition) -> Result {
    crate::pipeline::analyze_source(&report(source, edition)?)?;
    Ok(())
}

#[test]
fn real_identifiers_literals_repetitions_and_forwarded_literals_reach_body_checking() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "identifier",
            "protocol/direct-cooked",
            "protocol/literal-integer",
            "protocol/literal-spelling",
            "protocol/repetition",
            "protocol/literal-forwarding",
            "protocol/matching-definition-hygiene",
            "protocol/literal-string",
            "protocol/literal-wide-spelling",
            "protocol/normalized-name",
            "protocol/scoped-expression",
        ] {
            let report =
                report(&fixture(name)?, edition).map_err(|error| format!("{name}: {error}"))?;
            crate::pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        }
    }
    Ok(())
}

#[test]
fn invalid_operand_categories_arity_and_binding_depth_remain_rejected() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "direct-raw-string",
            "captured-raw-string",
            "raw-identifier",
            "suffixed-integer",
            "tt-integer",
            "expression-fragment",
            "one-operand",
            "trailing-comma",
            "unknown-variable",
            "still-repeating",
            "escaped-string",
            "literal-boolean",
        ] {
            let error = report(&fixture(&format!("protocol/{name}"))?, edition)
                .err()
                .ok_or_else(|| format!("{name}: invalid concat accepted"))?;
            assert!(error.to_string().contains("concat"), "{name}: {error}");
        }
        let error = check(&fixture("protocol/local-definition-hygiene")?, edition)
            .err()
            .ok_or("concat leaked a local to the caller")?;
        assert!(error.to_string().contains("unresolved value"), "{error}");
    }
    Ok(())
}

#[test]
fn crate_gate_applies_to_unused_and_generated_definitions_without_allowance_fallback() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "missing-feature",
            "protocol/unused-missing-feature",
            "protocol/allowances/definition-braces",
            "protocol/allowances/captured-braces",
            "protocol/allowances/same-definition-attribute",
        ] {
            let error = report(&fixture(name)?, edition)
                .err()
                .ok_or_else(|| format!("{name}: missing crate feature accepted"))?;
            assert!(
                error.to_string().contains("macro_metavar_expr_concat"),
                "{name}: {error}"
            );
        }
        report(&fixture("protocol/disabled-missing-feature")?, edition)?;
    }
    Ok(())
}

#[test]
fn synthesized_identifiers_keep_definition_context_edition_and_real_source_anchor() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        let source = fixture("identifier")?;
        let report = report(&source, edition)?;
        let unit = &report.package.units[0];
        let generated: Vec<_> = unit
            .origins
            .iter()
            .filter(|origin| {
                unit.source
                    .text()
                    .get(origin.generated.start as usize..origin.generated.end as usize)
                    == Some("fmt_small")
            })
            .collect();
        assert_eq!(generated.len(), 1);
        let origin = generated[0];
        assert_eq!(origin.kind, OriginKind::Synthetic(edition));
        assert_ne!(origin.introduction.0, 0);
        assert_ne!(
            origin.context,
            Some(crate::expand::marks::SyntaxContextId::ROOT)
        );
        assert!(
            source
                .get(origin.original.span.start as usize..origin.original.span.end as usize)
                .is_some_and(|text| text.starts_with("{concat("))
        );
    }
    Ok(())
}

#[test]
fn typed_concat_work_and_name_storage_scale_with_actual_emitted_bytes() -> Result {
    for width in [8, 32, 128] {
        let mut source =
            "macro_rules!m{($($n:ident),*)=>{$(fn ${concat($n,_x)}(){})*}}m!(".to_owned();
        for index in 0..width {
            use std::fmt::Write;
            if index != 0 {
                source.push(',');
            }
            write!(source, "v{index}")?;
        }
        source.push_str(");");
        let report =
            session::analyze(&source, &Config::default()).map_err(|error| format!("{error:?}"))?;
        let call = &report.invocations[0];
        assert_eq!(call.matched.captures.parser_calls, 0);
        assert_eq!(
            call.transcription
                .pieces
                .iter()
                .filter(|piece| matches!(piece, transcribe::Piece::Identifier { .. }))
                .count(),
            width
        );
        assert!(call.transcription.work < 128 + width * 128);
        assert!(report.names.byte_len() < 256 + width * 32);
        eprintln!(
            "concat width={width} work={} name_bytes={}",
            call.transcription.work,
            report.names.byte_len()
        );
    }
    let source = format!(
        "macro_rules!m{{()=>{{${{concat(\"{}\",\"{}\")}}}}}}m!();",
        "x".repeat(260_000),
        "y".repeat(260_000)
    );
    let error = session::analyze(&source, &Config::default())
        .err()
        .ok_or("concat work limit bypassed")?;
    assert!(error.message.contains("work budget"), "{error:?}");
    Ok(())
}

#[test]
fn ordinary_policy_and_retained_macro_record_widths_stay_explicit() -> Result {
    let source = "#![feature(macro_metavar_expr_concat)]macro_rules!m{()=>{fn ${concat(foo,_x)}()->u8{7}}}m!();fn main(){let _=foo_x();}";
    let report = test_source::resolve(source, true)?;
    assert!(report.package.source_core().is_none());
    crate::pipeline::analyze_source(&report)?;
    assert!(
        test_source::resolve(
            &source.replace("macro_metavar_expr_concat", "macro_metavar_expr"),
            true
        )
        .is_err()
    );
    assert_eq!(size_of::<model::Node>(), 40);
    assert_eq!(size_of::<model::Kind>(), 32);
    assert_eq!(size_of::<transcribe::Piece>(), 20);
    assert_eq!(size_of::<transcribe::Transcription>(), 32);
    assert_eq!(size_of::<model::Definition>(), 72);
    assert_eq!(size_of::<Environment<'_>>(), 64);
    Ok(())
}
