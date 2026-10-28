//! Exhaustive by-value matches retain cleanup only for initialized active payloads.
use crate::{
    body,
    borrow::{copy_view::CopyView, ownership::initialization},
    types::{Kind, copy::Copies, destruction::NeedsDrop},
};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn original() -> Result<(
    crate::resolve::expansion::Report,
    crate::sema::model::Declarations,
    body::templates::Template,
    crate::borrow::regions::Body,
)> {
    let source =
        crate::test_source::fixture("pa31/tests/spec/source-comparisons/native/actual-profile.rs")?;
    let report = crate::test_source::source_provider(&source, "symbolic_enum_cleanup")?;
    let crate::pipeline::Analysis {
        mut facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = *facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "is_some_and"
        })
        .ok_or("actual source is_some_and")?;
    let template = prepared.templates.function(&report, &facts, &function)?;
    let mut contract = body::regions::Collector::default()
        .function(&facts, &function)?
        .clone();
    body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
    contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    let proof = crate::borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        &contract,
        &template.body,
    )?;
    Ok((report, facts, template, proof))
}

fn select(
    facts: &crate::sema::model::Declarations,
    template: &body::templates::Template,
    proof: Option<&crate::borrow::regions::Body>,
) -> Result<super::Plan> {
    let mut copies = Copies::in_environment(Arc::clone(&template.environment));
    let mut view = if let Some(proof) = proof {
        CopyView::checked(&mut copies, &template.body, proof, facts.uses_source_copy())?
    } else {
        CopyView::ordinary(&mut copies)
    };
    let mut init = initialization::analyze_ordered(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut view,
        &template.body,
        None,
    )
    .map_err(|e| e.message)?;
    let custom = facts
        .destructors
        .implementations()
        .iter()
        .map(|i| i.item)
        .collect::<crate::support::hash::HashSet<_>>();
    let mut needed =
        NeedsDrop::in_environment(custom.iter().copied(), Arc::clone(&template.environment));
    Ok(super::select(
        &facts.types,
        &facts.aggregates,
        &mut needed,
        &mut init,
        |item| custom.contains(&item),
        |ty, query| body::drops::nominal_position(facts, ty, query),
    )
    .map_err(|e| e.message)?)
}

#[test]
fn actual_option_is_some_and_consumes_its_matched_payload() -> Result {
    let (report, facts, template, proof) = original()?;
    let input = template.body.locals.first().ok_or("self parameter")?.ty;
    let Some(Kind::Adt(item, _)) = facts.types.kind(input) else {
        return Err("source self lost its actual nominal application".into());
    };
    assert_eq!(
        report
            .package
            .names
            .get(report.resolution.definitions[item.0 as usize].name),
        "Option"
    );
    let Some(Kind::Adt(_, arguments)) = facts.types.kind(input) else {
        return Err("Option application".into());
    };
    let payload = facts
        .types
        .arguments(arguments)
        .ok_or("Option arguments")?
        .find_map(|a| {
            if let crate::types::arguments::Argument::Type(ty) = a {
                Some(ty)
            } else {
                None
            }
        })
        .ok_or("actual payload parameter")?;
    let destruct = facts
        .source_bindings
        .as_deref()
        .ok_or("source catalog")?
        .language(crate::support::bindings::LangItem::Destruct)
        .ok_or("actual Destruct")?
        .definition;
    for clause in template.environment.trait_clauses()? {
        let crate::types::parameters::clauses::Clause::Predicate(predicate) = clause else {
            continue;
        };
        if let Some((application, effect)) = predicate.bound.trait_parts() {
            let application = facts
                .types
                .trait_application(application)
                .ok_or("trait application")?;
            assert!(
                !(predicate.ty == payload
                    && application.item.0 == destruct.0
                    && effect != crate::support::constness::Constness::Ordinary),
                "an invented payload Destruct premise hid the move bug"
            );
        }
    }
    let plan = select(&facts, &template, Some(&proof))?;
    let mut retained = false;
    for cleanup in plan.schedule().cleanups() {
        assert!(
            !cleanup.access || cleanup.live_type != Some(input),
            "consumed matched Option payload remains active"
        );
        retained |= cleanup.live_type == Some(input);
    }
    assert!(retained, "original aggregate drop liveness was erased");
    println!(
        "actual {:?} self={input:?} plan_steps={}",
        template.body.key(),
        plan.steps
    );
    Ok(())
}

fn ordinary(source: &str) -> Result<(crate::sema::model::Declarations, body::templates::Template)> {
    let source = format!("enum E<T>{{A(T),B(T),C}}fn take<T>(_:T){{}}fn read<T>(_:&T){{}}{source}");
    let report = crate::test_source::resolve(&source, true)?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = *facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "probe"
        })
        .ok_or("probe")?;
    let template = prepared.templates.function(&report, &facts, &function)?;
    Ok((facts, template))
}

#[test]
fn fresh_variant_edges_keep_other_payloads_custom_drop_and_reinitialization() -> Result {
    for (source, active) in [
        (
            "fn probe<T>(x:E<T>){match x{E::A(a)=>take(a),E::B(b)=>take(b),E::C=>{}}}",
            false,
        ),
        (
            "fn probe<T>(x:E<T>){match x{E::A(a)=>take(a),E::B(ref b)=>read(b),E::C=>{}}}",
            true,
        ),
        (
            "impl<T> Drop for E<T>{fn drop(&mut self){}}fn probe<T>(x:E<T>){match x{E::C=>{},_=>{}}}",
            true,
        ),
        (
            "fn probe<T>(mut x:E<T>,y:T,flag:bool){loop{if flag{x=E::A(y);break;}match x{E::A(a)=>take(a),E::B(b)=>take(b),E::C=>{}}x=E::C;}}",
            true,
        ),
        (
            "fn probe<T>(mut x:E<T>,y:T){let b=match x{E::C=>true,_=>false};x=E::A(y);if b{}}",
            true,
        ),
    ] {
        let (facts, template) = ordinary(source)?;
        let plan = select(&facts, &template, None)?;
        let root = plan.schedule().cleanups().any(|cleanup| {
            cleanup.access
                && cleanup.candidate.place == body::ir::Place::Local(body::ir::LocalId(0))
                && matches!(cleanup.candidate.cause, initialization::Cause::Scope(_))
        });
        assert_eq!(root, active, "{source}");
    }
    Ok(())
}

#[test]
fn cleanup_variant_refinement_rejects_missing_and_foreign_members() -> Result {
    let (facts, template) = ordinary(
        "enum Other{A,B}fn probe<T>(x:E<T>){match x{E::A(a)=>take(a),E::B(b)=>take(b),E::C=>{}}}",
    )?;
    let input = template.body.locals.first().ok_or("input")?.ty;
    let Some(Kind::Adt(item, _)) = facts.types.kind(input) else {
        return Err("input nominal".into());
    };
    let foreign = facts
        .members
        .variants
        .iter()
        .enumerate()
        .find(|(_, v)| v.owner != item)
        .map(|(i, _)| u32::try_from(i).map(body::ir::VariantId))
        .transpose()?;
    let Some(foreign) = foreign else {
        return Err("foreign variant".into());
    };
    for variant in [foreign, body::ir::VariantId(u32::MAX)] {
        let mut changed = template.clone();
        let value = changed
            .body
            .values
            .iter_mut()
            .find(|v| matches!(v.kind, body::ir::Rvalue::IsVariant { .. }))
            .ok_or("actual test")?;
        let body::ir::Rvalue::IsVariant {
            variant: selected, ..
        } = &mut value.kind
        else {
            return Err("test".into());
        };
        *selected = variant;
        assert!(select(&facts, &changed, None).is_err());
    }
    Ok(())
}

#[test]
fn sparse_variant_refinement_has_bounded_payload_work_and_ignores_other_members() -> Result {
    use std::fmt::Write as _;
    let mut previous = None;
    let mut baseline = None;
    for (width, noise) in [(8, 0), (32, 0), (128, 0), (8, 128)] {
        let fields = std::iter::repeat_n("T", width)
            .collect::<Vec<_>>()
            .join(",");
        let names = (0..width)
            .map(|i| format!("v{i}"))
            .collect::<Vec<_>>()
            .join(",");
        let mut moves = String::new();
        for index in 0..width {
            write!(moves, "take(v{index});")?;
        }
        let mut source = format!(
            "enum W<T>{{Empty,Full({fields})}}fn probe<T>(x:W<T>){{match x{{W::Empty=>{{}},W::Full({names})=>{{{moves}}}}}}}"
        );
        for index in 0..noise {
            write!(source, "enum Noise{index}{{A(u8),B(u8),C}}")?;
        }
        let (facts, template) = ordinary(&source)?;
        let plan = select(&facts, &template, None)?;
        let input = template.body.locals.first().ok_or("input")?.ty;
        assert!(
            plan.schedule()
                .cleanups()
                .all(|c| !c.access || c.live_type != Some(input))
        );
        println!(
            "variant_fields={width} unrelated={noise} plan_steps={}",
            plan.steps
        );
        if noise == 0 {
            if let Some(old) = previous {
                assert!(plan.steps <= old * 6);
            }
            previous = Some(plan.steps);
            if width == 8 {
                baseline = Some(plan.steps);
            }
        } else {
            assert_eq!(Some(plan.steps), baseline);
        }
    }
    assert_eq!(size_of::<body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::const_eval::code::Code>(), 304);
    Ok(())
}

#[test]
fn untouched_wide_aggregates_keep_constant_cleanup_selection_work() -> Result {
    let mut baseline = None;
    for width in [8, 32, 128] {
        let fields = std::iter::repeat_n("T", width)
            .collect::<Vec<_>>()
            .join(",");
        let source = format!("struct Wide<T>({fields});fn probe<T>(x:Wide<T>){{}}");
        let (facts, template) = ordinary(&source)?;
        let plan = select(&facts, &template, None)?;
        let input = template.body.locals.first().ok_or("input")?.ty;
        assert!(
            plan.schedule()
                .cleanups()
                .any(|c| c.access && c.live_type == Some(input))
        );
        if let Some(steps) = baseline {
            assert_eq!(plan.steps, steps);
        }
        baseline = Some(plan.steps);
        println!("untouched_fields={width} selection_steps={}", plan.steps);
    }
    Ok(())
}
