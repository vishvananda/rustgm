//! Solved static arguments, rather than item shape, authorize canonical Code targets.
use super::*;
use crate::{
    body::ir::Rvalue,
    resolve::model::DefId,
    types::{Kind, instance::Instance},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn returned_callback_code_preserves_raw_roots_and_uses_each_proved_static_target_once() -> TestResult
{
    let source = crate::test_source::fixture(
        "pa31/tests/spec/const-eval-select-ctfe/mapped-through-return.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "proved_callback_snapshot")?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let context = analysis
        .prepared
        .constants
        .as_ref()
        .ok_or("constant context")?;
    let engine = context.engine.try_borrow()?;
    let facts = &analysis.facts;
    let definition = |name: &str| -> Result<DefId, Box<dyn std::error::Error>> {
        let index = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == name)
            .ok_or("actual definition")?;
        Ok(DefId(u32::try_from(index)?))
    };
    let choose = definition("choose")?;
    let code = engine
        .catalog
        .code(
            *engine
                .catalog
                .definitions
                .get(&choose)
                .ok_or("choose code")?,
        )
        .ok_or("actual code")?;
    assert!(
        matches!(code.owner, crate::body::ir::Owner::Function{ definition, .. } if definition == choose)
    );
    for callback in [definition("ct")?, definition("rt")?] {
        let value = code
            .values
            .iter()
            .find(|v| matches!(v.kind, Rvalue::Function(d) if d == callback))
            .ok_or("actual operand producer")?;
        let raw = Instance::new(&facts.types, value.ty)?;
        assert_eq!(facts.types.needs_region_context(raw.ty()), Some(true));
        let actual = code.callable_target(raw);
        assert_ne!(raw, actual);
        assert_eq!(facts.types.needs_region_context(actual.ty()), Some(false));
        assert_eq!(code.callable_target(actual), actual);
        let Some(Kind::FunctionItem {
            item,
            signature,
            arguments,
        }) = facts.types.kind(raw.ty())
        else {
            return Err("raw item".into());
        };
        let Some(Kind::FunctionItem {
            item: selected,
            signature: public,
            arguments: actual_arguments,
        }) = facts.types.kind(actual.ty())
        else {
            return Err("actual item".into());
        };
        assert_eq!(item, selected);
        assert_eq!(item.0, callback.0);
        assert_eq!(signature, public);
        assert_ne!(arguments, actual_arguments);
        if callback == definition("ct")? {
            let child = engine
                .catalog
                .code(
                    *engine
                        .catalog
                        .functions
                        .get(&actual)
                        .ok_or("selected F code")?,
                )
                .ok_or("selected code")?;
            assert_eq!(child.instance, Some(actual));
            assert_eq!(
                child.owner.signatures().map(|(signature, _)| signature),
                Some(public)
            );
        }
        println!(
            "callback={} raw={} actual={} raw-needs-context=true actual-needs-context=false",
            callback.0,
            raw.ty().0,
            actual.ty().0
        );
    }
    println!(
        "CheckedInput={} Prepared={} Code={} Machine={} State={}",
        size_of::<super::super::CheckedInput>(),
        size_of::<const_eval::code::Prepared>(),
        size_of::<const_eval::code::Code>(),
        size_of::<const_eval::Machine<'_, '_>>(),
        size_of::<const_eval::State>()
    );
    Ok(())
}
