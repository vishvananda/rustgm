//! A skipped intrinsic body demand requires a current exact source Call.
use super::*;
use crate::{body, support::bindings::SourceIntrinsic, types::arguments::Argument};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn abort_dependency_requires_current_header_full_instance_and_direct_call() -> Result {
    let source = r#"#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]
        #[lang="pointee_sized"]trait PointeeSized{}
        #[lang="meta_sized"]trait MetaSized:PointeeSized{}
        #[lang="sized"]trait Sized:MetaSized{}
        #[rustc_intrinsic]fn abort()->!;
        mod ordinary{pub fn abort()->!{loop{}}}
        fn actual()->!{abort()}
        fn same_name()->!{ordinary::abort()}"#;
    let mut report = crate::test_source::source_provider(source, "abort_dependencies")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut facts = crate::sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    for name in ["actual", "same_name"] {
        let body=program.bodies.iter().find(|body| matches!(body.owner,body::ir::Owner::Function{definition,..}
            if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)==name)).ok_or("caller")?;
        let call = body
            .values
            .iter()
            .find(|v| matches!(v.kind, Rvalue::Call { .. }))
            .ok_or("call")?;
        let Rvalue::Call {
            callee,
            signature,
            arguments,
        } = call.kind
        else {
            return Err("call".into());
        };
        let ty = body.values[callee.0 as usize].ty;
        assert_eq!(
            source_call(&report, &facts, body, call, ty)?,
            (name == "actual").then_some(SourceIntrinsic::Abort)
        );
        let mut changed = *call;
        changed.kind = Rvalue::Reify(callee);
        assert_eq!(source_call(&report, &facts, body, &changed, ty)?, None);
        if name == "same_name" {
            continue;
        }
        let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
            return Err("item".into());
        };
        let extra = facts
            .types
            .intern_arguments(&[Argument::Type(TypeId::UNIT)])?;
        let altered = facts.types.intern(Kind::FunctionItem {
            item,
            signature,
            arguments: extra,
        })?;
        assert!(source_call(&report, &facts, body, call, altered).is_err());
        changed = *call;
        changed.ty = TypeId::UNIT;
        assert!(source_call(&report, &facts, body, &changed, ty).is_err());
        changed = *call;
        changed.kind = Rvalue::Call {
            signature: TypeId::UNIT,
            callee,
            arguments,
        };
        assert!(source_call(&report, &facts, body, &changed, ty).is_err());
        changed.kind = Rvalue::Call {
            signature,
            callee,
            arguments: body::ir::ValueList { start: 0, end: 1 },
        };
        assert!(source_call(&report, &facts, body, &changed, ty).is_err());
        let mut changed_body = body.clone();
        changed_body.const_context = true;
        assert!(source_call(&report, &facts, &changed_body, call, ty).is_err());
        let foreign = crate::test_source::source_provider(source, "abort_dependencies")?;
        assert!(source_call(&foreign, &facts, body, call, ty).is_err());
        changed_live_header(&mut report, &facts, body, call, ty)?;
    }
    Ok(())
}

/// Mutating a completed declaration cannot borrow a warm source-header answer.
fn changed_live_header(
    report: &mut crate::resolve::expansion::Report,
    facts: &Declarations,
    body: &Body,
    call: &crate::body::ir::Value,
    ty: TypeId,
) -> Result {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
        return Err("item".into());
    };
    let def = crate::resolve::model::DefId(item.0);
    let node = facts.functions[*facts.function_indices.get(&def).ok_or("function")?].node;
    let unit = report
        .package
        .units
        .get_mut(node.file.0 as usize)
        .ok_or("current unit")?;
    let crate::syntax::ast::Kind::Item(index) = unit.configured.ast.node(node.node).kind else {
        return Err("item".into());
    };
    let original = unit.configured.ast.items[index as usize].kind;
    if let crate::syntax::item::ItemKind::Function { ref mut flags, .. } =
        unit.configured.ast.items[index as usize].kind
    {
        flags.constant = true;
    }
    assert!(source_call(report, facts, body, call, ty).is_err());
    report
        .package
        .units
        .get_mut(node.file.0 as usize)
        .ok_or("current unit")?
        .configured
        .ast
        .items[index as usize]
        .kind = original;
    assert_eq!(
        source_call(report, facts, body, call, ty)?,
        Some(SourceIntrinsic::Abort)
    );
    Ok(())
}
