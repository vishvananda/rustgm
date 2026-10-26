//! Current declaration authority is distinct from a typed physical origin row.
use super::*;
use crate::{body, support::bindings::SourceIntrinsic, types::arguments::Argument};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const BASE: &str = r#"#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]
#[lang="pointee_sized"]trait PointeeSized{}
#[lang="meta_sized"]trait MetaSized:PointeeSized{}
#[lang="sized"]trait Sized:MetaSized{}
mod intrinsics {#[rustc_intrinsic]pub const unsafe fn transmute<Src,Dst>(src:Src)->Dst;}
"#;
#[test]
fn source_transmute_authenticates_full_call_header_instance_and_pre_mono_size_relation() -> Result {
    let source = format!(
        "{BASE} mod ordinary{{pub fn transmute(x:u32)->u32{{x}}}} fn actual(x:u32)->u32{{unsafe{{intrinsics::transmute(x)}}}} fn same_name(x:u32)->u32{{ordinary::transmute(x)}}"
    );
    let mut report = crate::test_source::source_provider(&source, "transmute_dependencies")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut facts = crate::sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    for name in ["actual", "same_name"] {
        let body=program.bodies.iter().find(|body|matches!(body.owner,body::ir::Owner::Function {definition,..} if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)==name)).ok_or("caller")?;
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
            (name == "actual").then_some(SourceIntrinsic::Transmute)
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
        let wrong = facts
            .types
            .intern_arguments(&[Argument::Type(TypeId::UNIT)])?;
        let altered = facts.types.intern(Kind::FunctionItem {
            item,
            signature,
            arguments: wrong,
        })?;
        assert!(source_call(&report, &facts, body, call, altered).is_err());
        changed = *call;
        changed.ty = TypeId::UNIT;
        assert!(source_call(&report, &facts, body, &changed, ty).is_err());
        changed = *call;
        changed.kind = Rvalue::Call {
            callee,
            signature: TypeId::UNIT,
            arguments,
        };
        assert!(source_call(&report, &facts, body, &changed, ty).is_err());
        let foreign = crate::test_source::source_provider(&source, "transmute_dependencies")?;
        assert!(source_call(&foreign, &facts, body, call, ty).is_err());
        let def = crate::resolve::model::DefId(item.0);
        let node = facts.functions[*facts.function_indices.get(&def).ok_or("function")?].node;
        let unit = &mut report.package.units[node.file.0 as usize];
        let crate::syntax::ast::Kind::Item(index) = unit.configured.ast.node(node.node).kind else {
            return Err("item".into());
        };
        let original = unit.configured.ast.items[index as usize].kind;
        if let crate::syntax::item::ItemKind::Function { ref mut flags, .. } =
            unit.configured.ast.items[index as usize].kind
        {
            flags.constant = false;
        }
        assert!(source_call(&report, &facts, body, call, ty).is_err());
        report.package.units[node.file.0 as usize]
            .configured
            .ast
            .items[index as usize]
            .kind = original;
        assert_eq!(
            source_call(&report, &facts, body, call, ty)?,
            Some(SourceIntrinsic::Transmute)
        );
    }
    pre_mono_opponents()
}
fn pre_mono_opponents() -> Result {
    for (name, tail, message) in [
        (
            "unused_size",
            "fn unused(x:u8)->u16{unsafe{intrinsics::transmute(x)}}",
            "equal source and destination sizes",
        ),
        (
            "generic_size",
            "unsafe fn unused<T,U>(x:T)->U{unsafe{intrinsics::transmute(x)}}",
            "closed normalized type",
        ),
        (
            "unsafe",
            "fn unused(x:u8)->u8{intrinsics::transmute(x)}",
            "unsafe",
        ),
    ] {
        let input = format!("{BASE}{tail}");
        let report = crate::test_source::source_provider(&input, name)?;
        let result = crate::pipeline::analyze_source(&report);
        let error = match result {
            Ok(_) => return Err(format!("{name}: incorrectly accepted").into()),
            Err(error) => error.to_string(),
        };
        assert!(error.contains(message), "{name}: {error}");
    }
    Ok(())
}
