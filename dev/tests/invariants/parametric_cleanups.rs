//! Genuine const function cleanup remains defining legality, never executable glue.
use super::*;
use crate::borrow::{copy_view::CopyView, destruction, loans, ownership::initialization};
use crate::types::{copy::Copies, destruction::NeedsDrop};

fn report(conditional: bool) -> Result<crate::resolve::expansion::Report> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-const-callable/original-native29.rs")?;
    let (prefix, _) = source
        .split_once("const fn apply")
        .ok_or("actual callable declarations")?;
    let effect = if conditional { "[const] " } else { "" };
    let source = format!(
        "#![feature(const_destruct)]\n{prefix}\n\
        #[lang=\"destruct\"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]\n\
        pub const trait Destruct:PointeeSized{{}}\n\
        pub const fn consume<T:{effect}Destruct>(x:T)->u8{{7u8}}\n\
        pub const fn plain()->u8{{3u8}}\n\
        pub const fn borrowed<'a>(x:&'a u8)->u8{{*x}}"
    );
    Ok(crate::test_source::source_provider(
        &source,
        "const_function_cleanup",
    )?)
}

fn original() -> Result<(Declarations, Template, Contract)> {
    let report = report(true)?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    for function in &facts.functions {
        let name = report
            .package
            .names
            .get(report.resolution.definitions[function.definition.0 as usize].name);
        if matches!(name, "plain" | "borrowed") {
            let contract = crate::body::regions::Collector::default()
                .function(&facts, function)?
                .clone();
            assert!(
                contract
                    .builtins
                    .as_deref()
                    .is_none_or(|b| !b.has_defining())
            );
        }
    }
    let function = *facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "consume"
        })
        .ok_or("actual const function")?;
    let template = prepared.templates.function(&report, &facts, &function)?;
    assert!(template.body.const_context);
    let mut contract = crate::body::regions::Collector::default()
        .function(&facts, &function)?
        .clone();
    crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
    Ok((facts, template, contract))
}

#[test]
fn const_function_cleanup_requires_conditional_bound_and_sealed_original_carrier() -> Result {
    let (facts, template, contract) = original()?;
    let builtins = contract.builtins.as_deref().ok_or("builtins")?;
    let state = builtins.defining().ok_or("actual const function carrier")?;
    let mut work = Work::new(1_000_000);
    state.checked(&template.body, builtins.environment.as_ref(), &mut work)?;
    let foreign = Arc::new(template.environment.as_ref().clone());
    assert!(
        state
            .checked(&template.body, Some(&foreign), &mut work)
            .is_err()
    );
    assert!(state.checked(&template.body, None, &mut work).is_err());
    let mut changed = template.body.clone();
    changed.const_context = false;
    assert!(
        state
            .checked(&changed, builtins.environment.as_ref(), &mut work)
            .is_err()
    );
    let Owner::Function { definition, .. } = &mut changed.owner else {
        return Err("function".into());
    };
    *definition = DefId(u32::MAX);
    assert!(
        state
            .checked(&changed, builtins.environment.as_ref(), &mut work)
            .is_err()
    );
    let Owner::Function {
        definition: actual, ..
    } = template.body.owner
    else {
        return Err("function".into());
    };
    assert!(facts.is_const_function(actual));
    let ordinary = report(false)?;
    let error = crate::pipeline::analyze_source(&ordinary)
        .err()
        .ok_or("ordinary Destruct paid a const cleanup")?;
    assert!(
        error
            .to_string()
            .contains("conditional Destruct obligation"),
        "{error}"
    );
    Ok(())
}

#[test]
fn symbolic_const_cleanup_cannot_authorize_concrete_glue_or_an_instance() -> Result {
    let (mut facts, template, contract) = original()?;
    let proof = projections::regions(&mut facts, &template, &contract)?;
    let body = &template.body;
    let mut copies = Copies::in_environment(Arc::clone(&template.environment));
    let mut view = CopyView::checked(&mut copies, body, &proof, facts.uses_source_copy())?;
    let init = initialization::analyze_ordered(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut view,
        body,
        None,
    )
    .map_err(|e| e.message)?;
    let mut needed = NeedsDrop::in_environment([], Arc::clone(&template.environment));
    let symbolic = destruction::symbolic::select(
        &facts.types,
        &facts.aggregates,
        &mut needed,
        &init,
        |_| false,
        |_, _| Err("no nominal field projection in this source"),
    )
    .map_err(|e| e.message)?;
    let active = symbolic
        .schedule()
        .cleanups()
        .find(|c| c.access)
        .ok_or("real active generic cleanup")?;
    let mut live = crate::types::outlives::LiveRegions::default();
    let mut drops = crate::types::destruction::regions::Regions::new([])?;
    loans::check_schedule(
        &facts.types,
        &facts.aggregates,
        &mut live,
        &mut drops,
        &init,
        &proof,
        symbolic.schedule(),
    )?;
    // Deliberately forged executable annotation on the real active candidate.
    // A copied defining carrier must not authorize even its first concrete call.
    let concrete = destruction::Plan {
        cleanups: vec![destruction::Cleanup {
            candidate: active.candidate,
            node: Some(destruction::NodeId(0)),
            live_type: active.live_type,
        }],
        nodes: vec![destruction::Node {
            condition: destruction::Condition::Always,
            operation: destruction::Operation::Empty,
        }],
        ..Default::default()
    };
    let error = loans::check_destruction(
        &facts.types,
        &facts.aggregates,
        &mut live,
        &mut drops,
        &init,
        &proof,
        &concrete,
    )
    .err()
    .ok_or("defining carrier authorized concrete CTFE cleanup")?;
    assert_eq!(error.message, "constant evaluation cannot run destructors");
    reject_instance(&facts, &template, &proof, symbolic.schedule())?;
    assert!(size_of::<destruction::schedule::Schedule<'_>>() <= 48);
    Ok(())
}

fn reject_instance(
    facts: &Declarations,
    template: &Template,
    proof: &crate::borrow::regions::Body,
    symbolic: destruction::schedule::Schedule<'_>,
) -> Result {
    let body = &template.body;
    let mut live = crate::types::outlives::LiveRegions::default();
    let mut drops = crate::types::destruction::regions::Regions::new([])?;
    let mut instance = body.clone();
    let Owner::Function { definition, .. } = body.owner else {
        return Err("function".into());
    };
    instance.instance = Some(crate::types::instance::Instance::new(
        &facts.types,
        facts.definitions[definition.0 as usize].ok_or("actual prototype item")?,
    )?);
    let mut copies = Copies::in_environment(Arc::clone(&template.environment));
    let mut view = CopyView::ordinary(&mut copies);
    let init = initialization::analyze_ordered(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut view,
        &instance,
        None,
    )
    .map_err(|e| e.message)?;
    let error = loans::check_schedule(
        &facts.types,
        &facts.aggregates,
        &mut live,
        &mut drops,
        &init,
        proof,
        symbolic,
    )
    .err()
    .ok_or("symbolic const instance reused original defining cleanup")?;
    assert_eq!(error.message, "constant evaluation cannot run destructors");
    Ok(())
}
