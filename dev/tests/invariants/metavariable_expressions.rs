//! Independent numeric macro protocols exercise actual expansion and body checking.
use crate::{
    expand::{
        config::Config,
        macros::{model, session, transcribe},
        stream::{Environment, OriginKind},
    },
    support::edition::Edition,
    test_source,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ROOT: &str = "pa31/tests/spec/metavariable-expressions";
fn fixture(name: &str) -> Result<String> {
    Ok(test_source::fixture(&format!("{ROOT}/{name}.rs"))?)
}
fn check(source: &str, edition: Edition) -> Result {
    let report = test_source::source_provider_config(
        source,
        "metavar_protocol",
        &Config::in_edition(edition),
    )?;
    crate::pipeline::analyze_source(&report)?;
    Ok(())
}
fn matching(source: &str) -> Result<session::Report> {
    Ok(session::analyze(source, &Config::default()).map_err(|error| format!("{error:?}"))?)
}
fn numbers(report: &session::Report) -> Vec<u32> {
    report.invocations[0]
        .transcription
        .pieces
        .iter()
        .filter_map(|piece| {
            if let transcribe::Piece::Integer { value, .. } = piece {
                Some(*value)
            } else {
                None
            }
        })
        .collect()
}
#[test]
fn positive_counts_depths_ignored_values_and_escaped_definitions_reach_body_checking() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "count-flat",
            "count-empty",
            "count-depth",
            "count-subtree",
            "count-empty-first",
            "count-complete-depth",
            "ignore-repeat",
            "ignore-repeating-value",
            "ignore-scalar",
            "index-length-nested",
            "length-flat",
            "depth-radix",
            "count-unknown-unused",
            "ignore-unknown-unused",
            "index-unused-outside",
            "maximum-depth-unused",
            "disabled-missing-feature",
            "escaped-dollar",
            "escaped-expression",
            "escaped-dollar-disabled",
            "concat-unknown-unused",
            "execute-counts-owned",
        ] {
            check(&fixture(name)?, edition).map_err(|error| format!("{name}: {error}"))?;
        }
    }
    Ok(())
}
#[test]
fn invalid_arity_variables_depths_and_missing_drivers_remain_rejected() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "count-empty-first-depth",
            "count-consumed",
            "count-scalar",
            "count-depth-bounds",
            "index-outside",
            "len-outside",
            "index-depth-bounds",
            "len-depth-bounds",
            "driver-required",
            "count-unknown",
            "ignore-unknown",
            "suffixed-depth",
            "negative-depth",
            "float-depth",
            "group-depth",
            "trailing-index",
            "extra-ignore",
            "trailing-count",
            "extra-tokens",
            "raw-variable",
            "overflow-depth-unused",
            "escaped-dollar-matcher",
        ] {
            assert!(check(&fixture(name)?, edition).is_err(), "{name}");
        }
    }
    Ok(())
}
#[test]
fn crate_gate_covers_unused_escaped_and_generated_syntax_without_allowance_fallback() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for name in [
            "missing-feature",
            "concat-only-feature",
            "unused-missing-feature",
            "template-cfg-missing-feature",
            "internal-allowance",
            "escaped-dollar-missing-feature",
            "escaped-dollar-allowance",
        ] {
            let source = fixture(name)?;
            let error = test_source::source_provider_config(
                &source,
                "metavar_protocol",
                &Config::in_edition(edition),
            )
            .err()
            .ok_or_else(|| format!("{name}: missing gate accepted"))?;
            assert!(error.contains("macro_metavar_expr"), "{name}: {error}");
        }
    }
    Ok(())
}
#[test]
fn numeric_tokens_retain_definition_anchor_context_and_edition() -> Result {
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        let source = fixture("count-flat")?;
        let report = test_source::source_provider_config(
            &source,
            "metavar_protocol",
            &Config::in_edition(edition),
        )?;
        let unit = &report.package.units[0];
        let values: Vec<_> = unit
            .origins
            .iter()
            .filter(|origin| {
                source.get(origin.original.span.start as usize..origin.original.span.end as usize)
                    == Some("{count($x)}")
                    && unit
                        .source
                        .text()
                        .get(origin.generated.start as usize..origin.generated.end as usize)
                        == Some("3")
            })
            .collect();
        assert!(!values.is_empty(), "no actual generated count origin");
        for origin in values {
            assert_eq!(origin.kind, OriginKind::Synthetic(edition));
            assert_ne!(origin.introduction.0, 0);
            assert_ne!(
                origin.context,
                Some(crate::expand::marks::SyntaxContextId::ROOT)
            );
        }
    }
    assert_eq!(size_of::<model::Node>(), 40);
    assert_eq!(size_of::<model::Kind>(), 32);
    assert_eq!(size_of::<transcribe::Piece>(), 20);
    assert_eq!(size_of::<transcribe::Transcription>(), 32);
    assert_eq!(size_of::<model::Definition>(), 72);
    assert_eq!(size_of::<Environment<'_>>(), 64);
    Ok(())
}
#[test]
fn repeated_subtree_counts_build_one_linear_index_and_share_depth_queries() -> Result {
    let mut previous = 0;
    for width in [8, 32, 128] {
        let input = vec!["[a,b,c]"; width].join(",");
        let template = "${count($x)} ".repeat(width);
        let source =
            format!("macro_rules!m{{($( [ $($x:ident),* ] ),*)=>{{{template}}}}}m!({input});");
        let report = matching(&source)?;
        assert_eq!(numbers(&report), vec![u32::try_from(width * 3)?; width]);
        let work = report.invocations[0].transcription.work;
        eprintln!("subtree-count width={width} work={work}");
        if previous != 0 {
            assert!(work <= previous * 6, "subtree counting became quadratic");
        }
        previous = work;
    }
    previous = 0;
    for depth in [8, 16, 32] {
        let matcher = format!(
            "{}$x:ident{}",
            "$( [".repeat(depth + 1),
            "] )?".repeat(depth + 1)
        );
        let input = format!("{}a{}", "[".repeat(depth + 1), "]".repeat(depth + 1));
        let template = format!(
            "{}{}{}",
            "$(".repeat(depth),
            "${count($x)} ".repeat(depth),
            ")*".repeat(depth)
        );
        let report = matching(&format!(
            "macro_rules!m{{({matcher})=>{{{template}}}}}m!({input});"
        ))?;
        assert_eq!(numbers(&report), vec![1; depth]);
        let work = report.invocations[0].transcription.work;
        eprintln!("active-count depth={depth} work={work}");
        if previous != 0 {
            assert!(
                work <= previous * 3,
                "nested driver queries became quadratic"
            );
        }
        previous = work;
    }
    previous = 0;
    for depth in [8, 16, 32] {
        let width = depth * 4;
        let matcher = format!(
            "{}$($x:ident),*{}",
            "$( [".repeat(depth),
            "] )?".repeat(depth)
        );
        let input = (0..width)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let input = format!("{}{input}{}", "[".repeat(depth), "]".repeat(depth));
        let template = format!(
            "{}$(fn $x(){{}})*{}",
            "$(".repeat(depth),
            ")*".repeat(depth)
        );
        let report = matching(&format!(
            "macro_rules!m{{({matcher})=>{{{template}}}}}m!({input});"
        ))?;
        let call = &report.invocations[0];
        assert_eq!(
            call.transcription
                .pieces
                .iter()
                .filter(|piece| matches!(
                    piece,
                    transcribe::Piece::Token {
                        origin: transcribe::Origin::Invocation,
                        ..
                    }
                ))
                .count(),
            width
        );
        let work = call.transcription.work;
        eprintln!("ordinary-nested depth={depth} width={width} work={work}");
        if previous != 0 {
            assert!(
                work <= previous * 3,
                "ordinary capture paths became quadratic"
            );
        }
        previous = work;
    }

    Ok(())
}
#[test]
fn driver_sets_cover_more_than_one_word_and_ordinary_sources_keep_zero_policy() -> Result {
    use std::fmt::Write;
    let mut matcher = String::new();
    let mut body = String::new();
    let mut row = String::new();
    for index in 0..80 {
        write!(matcher, "$v{index}:ident ")?;
        write!(body, "${{ignore($v{index})}} ")?;
        write!(row, "a{index} ")?;
    }
    let source =
        format!("macro_rules!m{{($( {matcher} );*)=>{{$({body} ${{index()}})*}}}}m!({row};{row});");
    let report = matching(&source)?;
    assert_eq!(numbers(&report), [0, 1]);
    let ordinary = test_source::resolve("fn plain(){}", true)?;
    assert!(ordinary.package.source_core().is_none());
    assert_eq!(report.invocations[0].matched.captures.parser_calls, 0);
    Ok(())
}
