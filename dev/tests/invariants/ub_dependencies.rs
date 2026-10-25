//! Only the actual typed UB direct operation omits ordinary callable demand.
use super::*;
use crate::{body, support::bindings::SourceIntrinsic, types::arguments::Argument};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[test]
fn ub_dependency_omission_checks_actual_role_complete_instance_and_direct_call_shape() -> Result {
    let source = r#"#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]
        #[lang="pointee_sized"]trait PointeeSized{}
        #[lang="meta_sized"]trait MetaSized:PointeeSized{}
        #[lang="sized"]trait Sized:MetaSized{}
        #[rustc_intrinsic]const fn ub_checks()->bool{false}
        mod ordinary{pub const fn ub_checks()->bool{false}}
        fn actual()->bool{ub_checks()}
        fn same_name()->bool{ordinary::ub_checks()}"#;
    let report = crate::test_source::source_provider(source, "ub_dependencies")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let mut facts = crate::sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    for name in ["actual", "same_name"] {
        let body = program.bodies.iter().find(|body| matches!(body.owner, body::ir::Owner::Function { definition, .. }
            if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == name)).ok_or("caller")?;
        let call = body
            .values
            .iter()
            .find(|value| matches!(value.kind, Rvalue::Call { .. }))
            .ok_or("direct call")?;
        let Rvalue::Call {
            callee,
            signature,
            arguments,
        } = call.kind
        else {
            return Err("call".into());
        };
        let ty = body.values[callee.0 as usize].ty;
        let expected = (name == "actual").then_some(SourceIntrinsic::UbChecks);
        assert_eq!(source_call(&facts, body, call, ty)?, expected);
        let mut indirect = *call;
        indirect.kind = Rvalue::Reify(callee);
        assert_eq!(source_call(&facts, body, &indirect, ty)?, None);
        if name == "actual" {
            let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(ty) else {
                return Err("item".into());
            };
            let extra = facts
                .types
                .intern_arguments(&[Argument::Type(TypeId::primitive(
                    crate::types::Primitive::U8,
                ))])?;
            let changed = facts.types.intern(Kind::FunctionItem {
                item,
                signature,
                arguments: extra,
            })?;
            assert!(source_call(&facts, body, call, changed).is_err());
            let mut malformed = *call;
            malformed.ty = TypeId::UNIT;
            assert!(source_call(&facts, body, &malformed, ty).is_err());
            malformed = *call;
            malformed.kind = Rvalue::Call {
                signature: TypeId::UNIT,
                callee,
                arguments,
            };
            assert!(source_call(&facts, body, &malformed, ty).is_err());
            malformed.kind = Rvalue::Call {
                signature,
                callee,
                arguments: body::ir::ValueList { start: 0, end: 1 },
            };
            assert!(source_call(&facts, body, &malformed, ty).is_err());
        }
    }
    Ok(())
}
