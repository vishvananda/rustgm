//! Builtin field syntax retains its arena edges and source authority.
use crate::syntax::{
    ast::{Kind, Syntax},
    expression, gate_sites,
    interpolation::EMPTY_INPUTS,
    transfer,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct NoFiles;
impl crate::resolve::load::Sources for NoFiles {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> std::result::Result<Option<crate::resolve::load::InputFile>, String> {
        Ok(None)
    }
}
fn fixture_report(
    path: &str,
    edition: crate::support::edition::Edition,
) -> Result<crate::resolve::expansion::Report> {
    let source =
        crate::test_source::fixture(&format!("pa31/tests/spec/source-offset-of/{path}.rs"))?;
    let input = crate::resolve::load::InputFile {
        identity: "offset-fixture.rs".into(),
        path: "offset-fixture.rs".into(),
        source: crate::support::source::SourceFile::new(source.into_bytes())?,
    };
    let expanded = crate::resolve::expansion::analyze(
        input,
        &mut NoFiles,
        &crate::expand::config::Config::in_edition(edition),
    )?;
    Ok(crate::resolve::analyze_expanded(expanded)?)
}

#[test]
fn genuine_field_captures_and_name_authority_follow_independent_parser_observations() -> Result {
    use crate::support::edition::Edition;
    for edition in [Edition::Rust2021, Edition::Rust2024] {
        for path in [
            "const-api-followup/bare-numeric-chain",
            "const-api-followup/captured-numeric-chain",
            "const-api-followup/captured-base-zero",
            "const-api-followup/captured-literal-zero",
            "const-api-followup/macro-literal-builtin-allowance",
            "original/double-forwarded-fields",
            "original/original-generic-option",
        ] {
            let found =
                fixture_report(path, edition).map_err(|error| format!("{path}: {error}"))?;
            let mut count = 0;
            for unit in &found.package.units {
                if let Some(gates) = &unit.configured.ast.gate_sites {
                    for entry in gates.expressions() {
                        if entry
                            .kind(&unit.configured.ast)
                            .map_err(|error| error.message)?
                            == gate_sites::ExpressionKind::OffsetOf
                        {
                            assert!(entry.context_key().is_some(), "{path}");
                            count += 1;
                        }
                    }
                }
            }
            assert!(count > 0, "{path}");
        }
        for path in [
            "original/raw-builtin-name",
            "original/raw-builtin-prefix",
            "original/cfg-disabled-missing-builtin-gate",
            "const-api-followup/no-builtin-gate",
            "const-api-followup/captured-builtin-does-not-inherit",
            "const-api-followup/captured-grouped-name",
            "const-api-followup/grouped-field",
            "const-api-followup/captured-literal-hex-zero",
        ] {
            assert!(fixture_report(path, edition).is_err(), "{path}");
        }
    }
    Ok(())
}

#[test]
fn field_paths_are_typed_without_expression_or_layout_permission() -> Result {
    for (source, expected) in [
        ("builtin # offset_of(Option<B>, Some.0)", vec!["0", "Some"]),
        ("builtin # offset_of((u8,(u16,u32)), 1.1,)", vec!["1", "1"]),
        (
            "builtin # offset_of(S, r#type.inner)",
            vec!["inner", "r#type"],
        ),
    ] {
        let ast = expression::parse(source).map_err(|error| error.message)?;
        let Kind::OffsetOf {
            container,
            fields,
            gate,
            ..
        } = ast.node(ast.root).kind
        else {
            return Err("missing actual builtin expression".into());
        };
        let mut names = Vec::new();
        crate::syntax::offset_fields::visit(&ast, fields, |_, span| {
            names.push(&source[span.start as usize..span.end as usize]);
            Ok(())
        })
        .map_err(|error| error.message)?;
        assert_eq!(names, expected);
        let entry = ast.expression_gate(gate).ok_or("actual builtin gate")?;
        assert_eq!(
            entry.validate(&ast, gate).map_err(|error| error.message)?,
            fields
        );
        assert_eq!(
            &source[entry.keyword.start as usize..entry.keyword.end as usize],
            "offset_of"
        );
        assert!(entry.context_key().is_none());
        let mut children = Vec::new();
        ast.push_children(ast.root, &mut children);
        assert_eq!(children, [container, fields]);
        let pretty =
            expression::pretty::expression(source, &ast, ast.root, &EMPTY_INPUTS, &|_, _| Ok(()))
                .map_err(|error| error.message)?;
        let reparsed = expression::parse(&pretty).map_err(|error| error.message)?;
        assert!(matches!(
            reparsed.node(reparsed.root).kind,
            Kind::OffsetOf { .. }
        ));
    }
    for source in [
        "builtin # unknown(S, x)",
        "builtin # offset_of(S)",
        "builtin # offset_of(S,)",
        "builtin # offset_of(S, x, y)",
        "builtin # offset_of(S, x.)",
        "builtin # offset_of(S, x[0])",
        "builtin # offset_of(S, x + y)",
        "builtin # offset_of(S, x())",
        "builtin # offset_of(S, (x))",
        "builtin # offset_of(S, m::x)",
        "builtin # offset_of(S, 01)",
        "builtin # offset_of(S, 0usize)",
    ] {
        assert!(expression::parse(source).is_err(), "{source}");
    }
    Ok(())
}

#[test]
fn transfer_and_failed_retry_preserve_the_actual_gate_edges() -> Result {
    let ast =
        expression::parse("builtin # offset_of(S, inner.value)").map_err(|error| error.message)?;
    let original = *ast.expression_gate(0).ok_or("original gate")?;
    let mut target = Syntax::default();
    let mut prior = 0;
    for _ in 0..2 {
        assert!(
            transfer::append(
                &ast,
                &mut target,
                |span| {
                    if span == original.keyword {
                        Err(crate::syntax::token_tree::error(
                            span.start,
                            "declared name transfer failure",
                        ))
                    } else {
                        Ok(span)
                    }
                },
                |literal| Ok(literal.clone())
            )
            .is_err()
        );
        assert!(target.nodes.is_empty());
        let (rows, _, work, _) = target
            .gate_sites
            .as_ref()
            .ok_or("attempt debt")?
            .expression_storage();
        assert_eq!(rows, 0);
        assert!(work > prior);
        prior = work;
    }
    target.root = transfer::append(&ast, &mut target, Ok, |literal| Ok(literal.clone()))
        .map_err(|error| error.message)?;
    let entry = target.expression_gate(0).ok_or("transferred gate")?;
    entry.validate(&target, 0).map_err(|error| error.message)?;
    let mut forged = *entry;
    forged.keyword.start += 1;
    assert!(forged.validate(&target, 0).is_err());
    Ok(())
}

#[test]
fn sparse_offset_storage_has_bounded_growth_and_unchanged_hot_owners() -> Result {
    use std::fmt::Write;
    assert_eq!(size_of::<Kind>(), 28);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert!(
        expression::parse("{}")
            .map_err(|error| error.message)?
            .gate_sites
            .is_none()
    );
    let mut preceding = (0, 0);
    for width in [1, 8, 32] {
        let mut source = String::from("{");
        for _ in 0..width {
            write!(source, "builtin # offset_of(S, inner.value);")?;
        }
        source.push('}');
        let ast = expression::parse(&source).map_err(|error| error.message)?;
        let entries = ast.gate_sites.as_ref().ok_or("offset gate owner")?;
        let (rows, capacity, work, bytes) = entries.expression_storage();
        assert_eq!(rows, width);
        assert!(capacity <= width.max(4));
        assert!(work <= 32 * width);
        assert!(
            bytes
                <= size_of::<gate_sites::Declarations>()
                    + 8 * width * size_of::<gate_sites::Expression>()
        );
        assert!(work > preceding.0 && bytes > preceding.1);
        preceding = (work, bytes);
        println!("offset width={width} rows={rows} capacity={capacity} steps={work} bytes={bytes}");
    }
    Ok(())
}
