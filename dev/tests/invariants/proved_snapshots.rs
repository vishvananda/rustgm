//! Solved static arguments, rather than item shape, authorize canonical Code targets.
use super::*;
use crate::{
    body::ir::Rvalue,
    resolve::model::DefId,
    types::{Kind, instance::Instance},
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

fn assert_transferred_named_body(
    prepared: &crate::pipeline::Prepared,
    initializer: crate::sema::model::Initializer,
) -> TestResult {
    // analyze_source transfers demanded CFGs into the ordinary pipeline owner.
    let key = crate::body::ir::BodyKey::Declaration(initializer.definition);
    let body = prepared.concrete.get(&key).ok_or("retained named Body")?;
    assert_eq!(body.key(), key);
    assert_eq!(body.source.source(), Some(initializer.semantic_source()));
    assert_eq!(body.result, initializer.ty);
    assert!(matches!(
        initializer.kind,
        crate::sema::model::InitializerKind::Const
    ));
    assert!(matches!(body.owner, crate::body::ir::Owner::Initializer {
        definition, kind: crate::sema::model::InitializerKind::Const
    } if definition == initializer.definition));
    assert!(body.const_context);
    assert_eq!(body.instance, None);
    Ok(())
}

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

#[test]
fn early_named_reference_initializer_reuses_its_exact_code_without_repeating_proof() -> TestResult {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/const-eval-select-ctfe/reference-early-named.rs",
    )?;
    let report = crate::test_source::source_provider(&source, "early_named_reference")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    assert!(facts.array_lengths.values().any(|&length| length == 7));
    let initializer = *facts
        .initializers
        .iter()
        .find(|initializer| {
            report
                .package
                .names
                .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                == "N"
        })
        .ok_or("N")?;
    let context = prepared.constants.as_ref().ok_or("provider")?;
    let mut engine = context.engine.try_borrow_mut()?;
    let root = *engine
        .catalog
        .definitions
        .get(&initializer.definition)
        .ok_or("named Code")?;
    assert_transferred_named_body(&prepared, initializer)?;
    let key = crate::body::ir::BodyKey::Declaration(initializer.definition);
    assert!(!engine.concrete.contains_key(&key));
    let expression = super::super::Target::Expression {
        source: initializer.value,
        result: initializer.ty,
        context: None,
    };
    assert!(
        !engine.roots.contains_key(&expression),
        "actual early named requests must not create an anonymous proof owner"
    );
    let before = (
        engine.construction.work(),
        engine.catalog.bytes,
        engine.catalog.functions.len(),
        engine.catalog.definitions.len(),
        engine.catalog.expressions.len(),
        engine
            .const_calls
            .as_deref()
            .ok_or("Inputs")?
            .retained_bytes(),
        facts.body_work.cfg_functions.get(),
        facts.body_work.cfg_expressions.get(),
        facts.body_work.steps,
        facts.body_work.canonical_steps,
    );
    let pointer = std::ptr::from_ref(engine.catalog.code(root).ok_or("Code")?);
    for _ in 0..2 {
        let target = super::super::Target::Initializer(initializer.definition);
        engine.roots.insert(target, usize::MAX);
        assert_eq!(engine.root(&report, &mut facts, target)?, root);
        assert!(engine.prepare(
            &report,
            &mut facts,
            crate::sema::values::execution::Requirement::InitializerBody(initializer.definition),
            None
        )?);
        assert_eq!(
            std::ptr::from_ref(engine.catalog.code(root).ok_or("Code")?),
            pointer
        );
    }
    assert_eq!(
        before,
        (
            engine.construction.work(),
            engine.catalog.bytes,
            engine.catalog.functions.len(),
            engine.catalog.definitions.len(),
            engine.catalog.expressions.len(),
            engine
                .const_calls
                .as_deref()
                .ok_or("Inputs")?
                .retained_bytes(),
            facts.body_work.cfg_functions.get(),
            facts.body_work.cfg_expressions.get(),
            facts.body_work.steps,
            facts.body_work.canonical_steps,
        )
    );
    Ok(())
}

#[test]
fn warmed_named_initializer_rejects_changed_type_kind_and_canonical_owner() -> TestResult {
    let source =
        crate::test_source::fixture("pa31/tests/spec/const-eval-select-ctfe/reference-binder.rs")?;
    let report = crate::test_source::source_provider(&source, "warm_named_opponents")?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let index = facts
        .initializers
        .iter()
        .position(|initializer| {
            report
                .package
                .names
                .get(report.resolution.definitions[initializer.definition.0 as usize].name)
                == "N"
        })
        .ok_or("N")?;
    let original = facts.initializers[index];
    let context = prepared.constants.as_ref().ok_or("provider")?;
    let mut engine = context.engine.try_borrow_mut()?;
    let target = super::super::Target::Initializer(original.definition);
    let root = engine.root(&report, &mut facts, target)?;
    facts.initializers[index].ty = crate::types::TypeId::UNIT;
    assert!(engine.root(&report, &mut facts, target).is_err());
    facts.initializers[index] = original;
    facts.initializers[index].kind = crate::sema::model::InitializerKind::Static { mutable: false };
    assert!(engine.root(&report, &mut facts, target).is_err());
    facts.initializers[index] = original;
    let foreign = *engine
        .catalog
        .functions
        .values()
        .find(|&&code| code != root)
        .ok_or("actual callback Code")?;
    engine
        .catalog
        .definitions
        .insert(original.definition, foreign);
    assert!(engine.root(&report, &mut facts, target).is_err());
    engine.catalog.definitions.insert(original.definition, root);
    assert_eq!(engine.root(&report, &mut facts, target)?, root);
    Ok(())
}
