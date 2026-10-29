//! A source declaration and signature cannot replace the actual direct-call proof.
use crate::{
    body,
    body::ir::Rvalue,
    support::bindings::SourceIntrinsic,
    types::{Kind, TypeId, arguments::Argument},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;
#[test]
fn actual_wrapping_dependency_rejects_foreign_reports_changed_instances_and_non_calls() -> TestResult
{
    let text = r#"#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
#[rustc_intrinsic]pub const unsafe fn arith_offset<T>(p:*const T,n:isize)->*const T;
fn actual(p:*const u32,n:isize)->*const u32{unsafe{arith_offset(p,n)}}
"#;
    let mut report = crate::test_source::source_provider(text, "signed_dependencies")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut facts = crate::sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    let body=program.bodies.iter().find(|b|matches!(b.owner,body::ir::Owner::Function{definition,..}
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)=="actual"))
        .ok_or("actual caller body")?;
    let call = body
        .values
        .iter()
        .find(|v| matches!(v.kind, Rvalue::Call { .. }))
        .ok_or("actual Call")?;
    let Rvalue::Call {
        signature,
        callee,
        arguments,
    } = call.kind
    else {
        return Err("Call".into());
    };
    let ty = body.values[callee.0 as usize].ty;
    assert_eq!(
        super::super::source_call(&report, &facts, body, call, ty)?,
        Some(SourceIntrinsic::ArithOffset)
    );
    let mut changed = *call;
    changed.kind = Rvalue::Reify(callee);
    assert_eq!(
        super::super::source_call(&report, &facts, body, &changed, ty)?,
        None
    );
    changed = *call;
    changed.ty = TypeId::UNIT;
    assert!(super::super::source_call(&report, &facts, body, &changed, ty).is_err());
    changed = *call;
    changed.kind = Rvalue::Call {
        signature: TypeId::UNIT,
        callee,
        arguments,
    };
    assert!(super::super::source_call(&report, &facts, body, &changed, ty).is_err());
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
        return Err("item".into());
    };
    let wrong = facts
        .types
        .intern_arguments(&[Argument::Type(TypeId::UNIT)])?;
    let altered = facts.types.intern(Kind::FunctionItem {
        item,
        signature,
        arguments: wrong,
    })?;
    assert!(super::super::source_call(&report, &facts, body, call, altered).is_err());
    let foreign = crate::test_source::source_provider(text, "signed_dependencies")?;
    assert!(super::super::source_call(&foreign, &facts, body, call, ty).is_err());
    let definition = crate::resolve::model::DefId(item.0);
    let node = facts.functions[*facts
        .function_indices
        .get(&definition)
        .ok_or("definition")?]
    .node;
    let unit = &mut report.package.units[node.file.0 as usize];
    let crate::syntax::ast::Kind::Item(index) = unit.configured.ast.node(node.node).kind else {
        return Err("header".into());
    };
    if let crate::syntax::item::ItemKind::Function { ref mut flags, .. } =
        unit.configured.ast.items[index as usize].kind
    {
        flags.constant = false;
    }
    assert!(super::super::source_call(&report, &facts, body, call, ty).is_err());
    Ok(())
}
