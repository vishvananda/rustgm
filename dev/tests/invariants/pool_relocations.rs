//! C ABI literals retain their actual source owner through both typed relocations.
use super::*;
use crate::{
    expand::{config::Config, crate_syntax},
    literal::{Value, storage::Bytes},
    resolve::unit::Unit,
    syntax::{
        ast::Ast,
        expression::fragment::FragmentKind,
        interpolation::{Nonterminal, Splice},
        item::Abi,
        token_tree::Span,
    },
};
use std::borrow::Cow;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn parsed(text: &str) -> TestResult<(Unit, NodeId)> {
    super::constant_tests::selected(text)
}

fn convention(
    ast: &Syntax<StoredValue>,
) -> TestResult<(NodeId, crate::syntax::item::FunctionType)> {
    let (index, node) = ast
        .nodes
        .iter()
        .enumerate()
        .rev()
        .find(|(_, node)| matches!(node.kind, Kind::FunctionType(_)))
        .ok_or("actual function type")?;
    let Kind::FunctionType(function) = node.kind else {
        return Err("function type".into());
    };
    Ok((
        NodeId::new(u32::try_from(index)?),
        ast.function_types[function as usize],
    ))
}

fn abi_value(
    ast: &Syntax<StoredValue>,
    storage: &Storage,
    source: &str,
    function: crate::syntax::item::FunctionType,
) -> TestResult<(Span, StoredValue, Vec<u8>)> {
    let literal = function.abi.literal().ok_or("explicit ABI literal")?;
    let node = ast.node(literal);
    let Kind::Literal(index) = node.kind else {
        return Err("typed ABI literal node".into());
    };
    let value = ast.literals[index as usize];
    let Value::String(bytes) = storage.view(source, value)? else {
        return Err("ABI string".into());
    };
    Ok((node.span, value, bytes.into_owned()))
}

#[test]
fn raw_and_escaped_c_abi_literals_keep_separate_decoded_pool_owners() -> TestResult {
    for spelling in [
        "extern \"C\" fn()",
        "extern r#\"C\"# fn()",
        "extern \"\\x43\" fn()",
        "extern \"C\" fn(u8,*const u8)->u8",
    ] {
        let text =
            format!("const SOURCE:&str=\"source\\x5fpool_prefix\";struct Holder(({spelling},));");
        let (unit, root) = parsed(&text)?;
        let (_, original) = convention(&unit.configured.ast)?;
        let (original_span, original_value, original_bytes) = abi_value(
            &unit.configured.ast,
            &unit.literals,
            unit.source.text(),
            original,
        )?;
        assert_eq!(original_bytes, b"C");
        assert!(unit.literals.decoded_bytes() > 1);
        let copied = owned_field(
            &unit.configured.ast,
            unit.source.text(),
            &unit.literals,
            root,
            &|_, _| Ok(()),
        )
        .map_err(|error| error.message)?;
        let (_, function) = convention(&copied.ast)?;
        let (copied_span, copied_value, copied_bytes) =
            abi_value(&copied.ast, &copied.literals, unit.source.text(), function)?;
        assert_eq!(copied_span, original_span);
        assert_eq!(copied_bytes, b"C");
        if matches!(original_value, StoredValue::String(Bytes::Decoded { .. })) {
            assert_ne!(copied_value, original_value);
            assert_eq!(copied.literals.decoded_bytes(), 1);
        } else {
            assert_eq!(copied_value, original_value);
            assert_eq!(copied.literals.decoded_bytes(), 0);
        }
        relocated_c_abi(&unit, root, &copied, original_span)?;
    }
    Ok(())
}

fn relocated_c_abi(
    unit: &Unit,
    root: NodeId,
    copied: &ParsedSyntax,
    original_span: Span,
) -> TestResult {
    let original = unit.configured.ast.node(root).span;
    let field = &unit.source.text()[original.start as usize..original.end as usize];
    let image = format!("prefix {field} suffix");
    let destination = Span {
        start: 7,
        end: u32::try_from(7 + field.len())?,
    };
    let mut target = Ast::default();
    target.literals.push(Value::String(Cow::Owned(
        b"different_destination_pool_prefix".to_vec(),
    )));
    target.root = Splice {
        fragment: Nonterminal::Syntax {
            kind: FragmentKind::Type,
            parsed: copied,
        },
        original_source: unit.source.text(),
        original,
        destination,
    }
    .import(&image, &mut target)
    .map_err(|error| error.message)?;
    let configured = crate_syntax::configure(&image, target, &Config::default())
        .map_err(|error| error.message)?;
    let (mut receiver, _) =
        parsed("const RECEIVER:&str=\"receiver\\x5fpool\";struct Prior((u8,));")?;
    let before = receiver.literals.decoded_bytes();
    assert!(before > 1);
    let (generated, _) = receiver
        .append_derived(&image, configured, &[])
        .map_err(|error| error.message)?;
    let (_, installed) = convention(&receiver.configured.ast)?;
    let (span, _, bytes) = abi_value(
        &receiver.configured.ast,
        &receiver.literals,
        receiver.source.text(),
        installed,
    )?;
    assert_eq!(bytes, b"C");
    assert_eq!(
        span.start,
        generated.start + destination.start + original_span.start - original.start
    );
    assert_eq!(
        span.end,
        generated.start + destination.start + original_span.end - original.start
    );
    assert!(
        receiver.literals.decoded_bytes() >= before + b"different_destination_pool_prefix".len()
    );
    Ok(())
}

#[test]
fn owned_callable_copy_preserves_safety_default_abi_and_existing_profile_fences() -> TestResult {
    signature_profiles()?;
    function_children()?;
    literal_owner_opponents()
}

fn signature_profiles() -> TestResult {
    for spelling in ["fn()", "extern fn()", "unsafe extern \"C\" fn()"] {
        let text = format!("struct Holder(({spelling},));");
        let (unit, root) = parsed(&text)?;
        let (_, original) = convention(&unit.configured.ast)?;
        let copied = owned_field(
            &unit.configured.ast,
            unit.source.text(),
            &unit.literals,
            root,
            &|_, _| Ok(()),
        )
        .map_err(|error| error.message)?;
        let (_, actual) = convention(&copied.ast)?;
        assert_eq!(actual.unsafe_, original.unsafe_);
        if matches!(original.abi, Abi::Rust | Abi::Extern(None)) {
            assert_eq!(actual.abi, original.abi);
        }
        assert_eq!(actual.binders.start, actual.binders.end);
        assert_eq!(actual.parameters.start, actual.parameters.end);
        assert_eq!(actual.result, None);
    }
    for spelling in [
        "extern \"system\" fn()",
        "extern \"Rust\" fn()",
        "for<'a> fn(&'a u8)->&'a u8",
        "unsafe extern \"C\" fn(u8,...)",
    ] {
        let text = format!("struct Holder(({spelling},));");
        let (unit, root) = parsed(&text)?;
        let error = owned_field(
            &unit.configured.ast,
            unit.source.text(),
            &unit.literals,
            root,
            &|_, _| Ok(()),
        )
        .err()
        .ok_or("unsupported callable copied")?;
        assert!(error.message.contains("profile"), "{}", error.message);
    }
    let (unit, root) = parsed("struct Holder(([u8;take(\"\\x43\")],));")?;
    let error = owned_field(
        &unit.configured.ast,
        unit.source.text(),
        &unit.literals,
        root,
        &|_, _| Ok(()),
    )
    .err()
    .ok_or("ordinary string count copied")?;
    assert!(
        error
            .message
            .contains("derived constant literal requires its actual byte-pool owner")
    );
    Ok(())
}

fn function_children() -> TestResult {
    let (unit, root) = parsed("struct Holder((extern \"C\" fn(u8,*const u8)->u8,));")?;
    let (_, original) = convention(&unit.configured.ast)?;
    let copied = owned_field(
        &unit.configured.ast,
        unit.source.text(),
        &unit.literals,
        root,
        &|_, _| Ok(()),
    )
    .map_err(|error| error.message)?;
    let (_, actual) = convention(&copied.ast)?;
    assert_eq!(unit.configured.ast.list(original.parameters).len(), 2);
    assert_eq!(copied.ast.list(actual.parameters).len(), 2);
    for (&source, &target) in unit
        .configured
        .ast
        .list(original.parameters)
        .iter()
        .zip(copied.ast.list(actual.parameters))
    {
        assert_eq!(
            unit.configured.ast.node(source).span,
            copied.ast.node(target).span
        );
    }
    let source_result = unit
        .configured
        .ast
        .node(original.result.ok_or("original result")?);
    let copied_result = copied.ast.node(actual.result.ok_or("copied result")?);
    assert_eq!(copied_result.span, source_result.span);
    assert_eq!(copied_result.kind, source_result.kind);
    Ok(())
}

fn literal_owner_opponents() -> TestResult {
    let (mut unit, root) =
        parsed("const PREFIX:&str=\"real\\x5fowner\";struct Holder((extern \"\\x43\" fn(),));")?;
    let wrong = owned_field(
        &unit.configured.ast,
        unit.source.text(),
        &Storage::default(),
        root,
        &|_, _| Ok(()),
    )
    .err()
    .ok_or("ABI string used a foreign decoded pool")?;
    assert!(wrong.message.contains("invalid stored literal range"));
    unit.configured.ast.function_types[0].abi = Abi::Extern(Some(NodeId::new(u32::MAX)));
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
    Ok(())
}
