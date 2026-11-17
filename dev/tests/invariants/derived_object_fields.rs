//! Source derive object fields retain real callable, lifetime and payer records.
use super::*;
use std::cell::Cell;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

#[test]
fn object_field_copies_bound_rows_and_callable_children_with_original_spans() -> TestResult {
    let source = "struct Holder<'outer>((&'outer (dyn for<'local> Fn(&'local u8) -> &'local u8 + 'outer),));";
    let (unit, root) = super::constant_tests::selected(source)?;
    let ast = &unit.configured.ast;
    assert!(field(ast, root, &|_, _| Ok(())).is_err());
    let copied =
        owned_field(ast, source, &unit.literals, root, &|_, _| Ok(())).map_err(|e| e.message)?;
    let source_node = |kind: fn(Kind) -> bool| -> TestResult<_> {
        Ok(ast
            .nodes
            .iter()
            .find(|node| kind(node.kind))
            .ok_or("original node")?)
    };
    let trait_type = |kind| matches!(kind, Kind::TraitType { opaque: false, .. });
    let copied_type = copied
        .ast
        .nodes
        .iter()
        .find(|node| trait_type(node.kind))
        .ok_or("copied trait type")?;
    assert_eq!(copied_type.span, source_node(trait_type)?.span);
    let Kind::TraitType { bounds, .. } = copied_type.kind else {
        return Err("copied bounds".into());
    };
    let [principal, lifetime] = copied.ast.list(bounds) else {
        return Err("principal and lifetime rows".into());
    };
    let Kind::Bound {
        quantified: true,
        binders,
        value,
        ..
    } = copied.ast.node(*principal).kind
    else {
        return Err("quantified principal".into());
    };
    let [binder] = copied.ast.list(binders) else {
        return Err("one actual binder".into());
    };
    let Kind::Generic(index) = copied.ast.node(*binder).kind else {
        return Err("actual lifetime parameter".into());
    };
    assert_eq!(index, 0);
    assert_eq!(copied.ast.generics.len(), 1);
    let spelling =
        |span: crate::syntax::token_tree::Span| &source[span.start as usize..span.end as usize];
    assert_eq!(spelling(copied.ast.generics[index as usize].name), "'local");
    let Kind::Path { segments, .. } = copied.ast.node(value).kind else {
        return Err("callable principal path".into());
    };
    let [segment] = copied.ast.list(segments) else {
        return Err("one callable segment".into());
    };
    let Kind::Applied {
        arguments,
        output: Some(output),
        parenthesized: true,
        ..
    } = copied.ast.node(*segment).kind
    else {
        return Err("parenthesized callable inputs and result".into());
    };
    let [input] = copied.ast.list(arguments) else {
        return Err("one callable argument".into());
    };
    assert_eq!(spelling(copied.ast.node(*input).span), "&'local u8");
    assert_eq!(spelling(copied.ast.node(output).span), "&'local u8");
    let Kind::Bound {
        quantified: false,
        value,
        ..
    } = copied.ast.node(*lifetime).kind
    else {
        return Err("separate object lifetime".into());
    };
    assert_eq!(spelling(copied.ast.node(value).span), "'outer");
    Ok(())
}

#[test]
fn object_field_copy_pays_only_selected_nodes_and_rejects_opaque_types() -> TestResult {
    let shape = "struct Holder<'a>((&'a (dyn Fn() -> &'a u8 + 'a),));";
    let width = |source: &str| -> TestResult<(usize, usize)> {
        let (unit, root) = super::constant_tests::selected(source)?;
        let paid = Cell::new((0, 0));
        owned_field(
            &unit.configured.ast,
            source,
            &unit.literals,
            root,
            &|steps, bytes| {
                let (work, storage) = paid.get();
                paid.set((work + steps, storage + bytes));
                Ok(())
            },
        )
        .map_err(|e| e.message)?;
        Ok(paid.get())
    };
    assert_eq!(
        width(shape)?,
        width(&format!("{}{}", "struct Noise;".repeat(128), shape))?
    );
    let (unit, root) = super::constant_tests::selected("struct Holder((impl Trait,));")?;
    assert!(
        owned_field(
            &unit.configured.ast,
            unit.source.text(),
            &unit.literals,
            root,
            &|_, _| Ok(())
        )
        .is_err()
    );
    assert!(
        owned_field(
            &unit.configured.ast,
            unit.source.text(),
            &unit.literals,
            root,
            &|_, _| { Err(error(0, "controlled payer exhaustion")) }
        )
        .is_err_and(|error| error.message == "controlled payer exhaustion")
    );
    Ok(())
}
