//! The actual union recipe copies Self and preserves typed source headers.
use super::*;
use crate::syntax::{
    ast::{List, NodeId, Unary},
    item::GenericFlavor,
    token_tree::Span,
};
fn input(name: &str) -> TestResult<String> {
    Ok(test_source::fixture(&format!(
        "pa31/tests/spec/union-clone/{name}.rs"
    ))?)
}
fn span(
    report: &crate::resolve::expansion::Report,
    name: &str,
) -> TestResult<(NodeKey, usize, Span)> {
    let unit = &report.package.units[0];
    let ast = &unit.configured.ast;
    for entry in &report
        .attribute_effects
        .as_deref()
        .and_then(|e| e.derives.as_deref())
        .ok_or("transactions")?
        .entries
    {
        let mut node = entry.owner.node;
        while let Kind::Attributed { value, .. } = ast.node(node).kind {
            node = value;
        }
        let Kind::Item(index) = ast.node(node).kind else {
            continue;
        };
        let item = ast.items[index as usize];
        if item
            .name
            .is_some_and(|span| &unit.source.text()[span.start as usize..span.end as usize] == name)
        {
            assert!(matches!(item.kind, ItemKind::Union { .. }));
            report.expanded_derive(entry.owner, entry.effect)?;
            return Ok((
                entry.owner,
                entry.effect,
                report.invocations[entry.invocation].generated,
            ));
        }
    }
    Err("actual union transaction".into())
}
fn text(unit: &crate::resolve::unit::Unit, node: NodeId) -> String {
    let span = unit.configured.ast.node(node).span;
    unit.source.text()[span.start as usize..span.end as usize]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect()
}
fn bounds(unit: &crate::resolve::unit::Unit, list: List) -> TestResult<Vec<String>> {
    unit.configured
        .ast
        .list(list)
        .iter()
        .map(|&node| {
            let Kind::Bound { value, .. } = unit.configured.ast.node(node).kind else {
                return Err("typed bound".into());
            };
            Ok(text(unit, value))
        })
        .collect()
}
fn body(unit: &crate::resolve::unit::Unit, items: List) -> TestResult {
    let ast = &unit.configured.ast;
    let &[function] = ast.list(items) else {
        return Err("one clone method".into());
    };
    let mut node = function;
    while let Kind::Attributed { value, .. } = ast.node(node).kind {
        node = value;
    }
    let Kind::Item(index) = ast.node(node).kind else {
        return Err("clone method item".into());
    };
    let ItemKind::Function {
        body: Some(root), ..
    } = ast.items[index as usize].kind
    else {
        return Err("actual clone body".into());
    };
    let mut work = vec![root];
    let mut assertions = 0;
    let mut dereferences = 0;
    while let Some(node) = work.pop() {
        match ast.node(node).kind {
            Kind::Let {
                ty: Some(ty),
                value: None,
                ..
            } => {
                assert_eq!(text(unit, ty), "crate::clone::AssertParamIsCopy<Self>");
                assertions += 1;
            }
            Kind::Unary {
                op: Unary::Dereference,
                ..
            } => dereferences += 1,
            Kind::Field { .. } | Kind::Call { .. } => {
                return Err("union Clone reads a field or calls field Clone".into());
            }
            _ => {}
        }
        ast.push_children(node, &mut work);
    }
    assert_eq!((assertions, dereferences), (1, 1));
    Ok(())
}
fn headers(report: &crate::resolve::expansion::Report, name: &str, generic: bool) -> TestResult {
    let unit = &report.package.units[0];
    let ast = &unit.configured.ast;
    let (_, _, range) = span(report, name)?;
    let mut count = 0;
    for node in &ast.nodes {
        if node.span.start < range.start || node.span.end > range.end {
            continue;
        }
        let Kind::Item(index) = node.kind else {
            continue;
        };
        let item = ast.items[index as usize];
        let ItemKind::Impl {
            trait_path: Some(path),
            items,
            unsafe_,
            ..
        } = item.kind
        else {
            continue;
        };
        let role = text(unit, path);
        assert!(matches!(
            role.as_str(),
            "crate::marker::Copy" | "crate::clone::Clone" | "crate::clone::TrivialClone"
        ));
        assert_eq!(unsafe_, role == "crate::clone::TrivialClone");
        let formals = ast.list(item.generics);
        assert_eq!(formals.len(), if generic { 3 } else { 1 });
        for &parameter in formals {
            let Kind::Generic(index) = ast.node(parameter).kind else {
                return Err("actual formal".into());
            };
            let parameter = ast.generics[index as usize];
            assert!(parameter.default.is_none());
            if parameter.flavor == GenericFlavor::Type && role != "crate::marker::Copy" {
                assert_eq!(
                    bounds(unit, parameter.bounds)?,
                    vec!["crate::marker::Copy", role.as_str(), "Copy", "'a"]
                );
            }
        }
        assert_eq!(ast.list(item.where_clause).len(), usize::from(generic));
        if role == "crate::clone::Clone" {
            body(unit, items)?;
        } else {
            assert!(ast.list(items).is_empty());
        }
        count += 1;
    }
    assert_eq!(count, 3);
    Ok(())
}
#[test]
fn exact_escape_and_generic_headers_use_copy_self_recipe_and_authenticated_replay() -> TestResult {
    let original = expanded_provider(&input("original-expansion")?, "escape_union")?;
    headers(&original, "MaybeEscapedCharacter", false)?;
    let mut report = expanded_provider(&input("header-expansion")?, "union_headers")?;
    headers(&report, "Borrowed", true)?;
    let (owner, effect, _) = span(&report, "Borrowed")?;
    let definition = report
        .invocations
        .iter()
        .find_map(|i| {
            matches!(
                report.definitions[i.definition].body,
                Body::Builtin(Builtin::Clone)
            )
            .then_some(i.definition)
        })
        .ok_or("actual Clone member")?;
    report.definitions[definition].body = Body::Builtin(Builtin::Copy);
    assert!(report.expanded_derive(owner, effect).is_err());
    report.definitions[definition].body = Body::Builtin(Builtin::Clone);
    report.expanded_derive(owner, effect)?;
    Ok(())
}
fn lowered(source: &str, tag: &str) -> TestResult<crate::lowir::program::Program> {
    let report = test_source::source_provider(source, tag)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn real_source_scalar_and_array_unions_copy_nonzero_bits_through_lowir() -> TestResult {
    let source = input("source-runtime")?;
    let program = lowered(&source, "source_union_clone")?;
    assert!(
        program
            .symbols
            .iter()
            .any(|s| s.name == "observed" && s.body.is_some())
    );
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/union-clone-v1/candidate");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn missing_copy_wrong_trait_and_counterfeit_helper_never_grant_self_copy() -> TestResult {
    for name in [
        "source-missing-copy",
        "source-copy-lookalike",
        "source-wrong-helper",
    ] {
        let error = lowered(&input(name)?, "union_copy_opponent")
            .err()
            .ok_or_else(|| format!("accepted {name}"))?
            .to_string();
        assert!(!error.contains("budget"), "{name}: {error}");
        assert!(!error.contains("wider profile"), "{name}: {error}");
        assert!(!error.contains("unresolved"), "{name}: {error}");
        eprintln!("union Clone opponent{name}: {error}");
    }
    Ok(())
}
#[test]
fn union_field_inventory_is_linear_but_generated_clone_is_constant_and_warm() -> TestResult {
    use std::fmt::Write as _;
    let base = input("source-runtime")?;
    let mut previous = None;
    let mut output = None;
    for count in [8, 32, 128] {
        let mut source = base.clone();
        source.push_str(
            "#[derive(crate::marker::Copy,crate::clone::Clone)]union Many<const N:usize=3>{",
        );
        for index in 0..count {
            write!(&mut source, "f{index}:u32,")?;
        }
        source.push('}');
        let report = expanded_provider(&source, "many_union_clones")?;
        let (owner, effect, range) = span(&report, "Many")?;
        let length = range.end - range.start;
        if let Some(old) = output {
            assert_eq!(length, old);
        }
        output = Some(length);
        let payer = &report.package.source_core().ok_or("payer")?.macros;
        let cold = payer.usage();
        report.expanded_derive(owner, effect)?;
        let warm = payer.usage();
        assert!(warm.0 > cold.0);
        assert_eq!(warm.1, cold.1);
        if let Some((work, bytes)) = previous {
            assert!(cold.0 <= work * 5);
            assert!(cold.1 <= bytes * 5);
        }
        previous = Some((cold.0, cold.1));
        eprintln!("union Clone fields={count} outputbytes={length} cold={cold:?} warm={warm:?}");
    }
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    Ok(())
}

#[test]
fn rejected_actual_union_payment_leaves_source_and_effect_unpublished() -> TestResult {
    let source = input("payment-source")?;
    crate::resolve::expansion::role_declarations::with_engine(&source, true, |engine| {
        let before = engine.loader.package.units[0].source.text().len();
        let payer = &engine
            .loader
            .package
            .source_core()
            .ok_or("actual source payer")?
            .macros;
        let usage = payer.usage();
        payer.charge(
            0,
            (64 * 1024 * 1024_usize)
                .saturating_sub(usage.1)
                .saturating_sub(1024),
        )?;
        let error = engine
            .run()
            .err()
            .ok_or("accepted exhausted union transaction")?
            .to_string();
        assert!(error.contains("attempted storage budget"), "{error}");
        assert_eq!(engine.loader.package.units[0].source.text().len(), before);
        assert!(
            engine.loader.package.units[0]
                .configured
                .effects
                .iter()
                .all(|e| e.attribute.effect != Effect::ExpandedDerive)
        );
        Ok(())
    })?;
    Ok(())
}
