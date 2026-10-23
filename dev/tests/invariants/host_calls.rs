//! Exact use effects and declarations separate signature equations from permission.
use super::*;
use crate::{
    sema::traits::{assumptions::Assumptions, selection},
    types::{
        ItemId, TypeId,
        infer::Instantiation,
        parameters::predicates::{Bound, Predicate as Canonical},
        traits::Application,
    },
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn provider() -> TestResult<(crate::resolve::expansion::Report, Declarations)> {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-payment/callable/const-function-item.rs"),
    )? + "fn plain()->u8{2}const fn conditional<T:[const]Clone>()->u8{7}fn materialize(){conditional::<u8>();}";
    let report = crate::test_source::source_provider(&source, "host_effect_checks")?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
fn item(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
    name: &str,
) -> TestResult<TypeId> {
    let definition = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == name
        })
        .ok_or("function")?
        .definition;
    Ok(facts.definitions[definition.0 as usize].ok_or("item type")?)
}
fn predicate(facts: &mut Declarations, ty: TypeId, effect: Constness) -> TestResult<Canonical> {
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::FnOnce)?
        .ok_or("FnOnce")?;
    let list = facts.types.intern_list(&[])?;
    let unit = facts.types.intern(Kind::Tuple(list))?;
    let args = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(unit)])?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.definition.0),
        arguments: args,
    })?;
    Ok(Canonical {
        ty,
        bound: Bound::trait_application(application, effect),
    })
}
fn select(facts: &mut Declarations, predicate: Canonical) -> TestResult<selection::Selection> {
    let mut context = Context::default();
    let mut fold = Instantiation::new(&mut context, &facts.types, &[])?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut fold)?;
    Ok(selection::select(
        facts,
        &mut context,
        &goal,
        selection::Caller {
            assumptions: &Assumptions::default(),
            environment: None,
            first_region: 1,
            retain_sized: false,
        },
    )?)
}
#[test]
fn const_callable_cold_warm_orders_preserve_each_exact_target() -> TestResult {
    for order in [
        [Constness::Ordinary, Constness::Const],
        [Constness::Const, Constness::Ordinary],
    ] {
        let (report, mut facts) = provider()?;
        let constant = item(&report, &facts, "item")?;
        let plain = item(&report, &facts, "plain")?;
        let Kind::FunctionItem {
            signature: pointer, ..
        } = facts.types.kind(constant).ok_or("kind")?
        else {
            return Err("item".into());
        };
        for effect in order
            .into_iter()
            .chain(order)
            .chain([Constness::Conditional])
        {
            for (ty, accepted) in [
                (constant, true),
                (plain, effect == Constness::Ordinary),
                (pointer, effect == Constness::Ordinary),
            ] {
                let goal = predicate(&mut facts, ty, effect)?;
                assert_eq!(
                    matches!(select(&mut facts, goal)?, selection::Selection::Proven(_)),
                    accepted
                );
            }
        }
    }
    Ok(())
}
#[test]
fn const_callable_rejects_forged_signature_and_retains_unpaid_instance() -> TestResult {
    let (report, mut facts) = provider()?;
    let actual = item(&report, &facts, "item")?;
    let Kind::FunctionItem {
        item: id,
        arguments,
        ..
    } = facts.types.kind(actual).ok_or("kind")?
    else {
        return Err("item".into());
    };
    let parameters = facts.types.intern_list(&[])?;
    let signature = facts.types.intern(Kind::Function {
        bound_regions: 0,
        parameters,
        result: TypeId::primitive(crate::types::Primitive::Bool),
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let forged = facts.types.intern(Kind::FunctionItem {
        item: id,
        arguments,
        signature,
    })?;
    let goal = predicate(&mut facts, forged, Constness::Const)?;
    assert!(select(&mut facts, goal).is_err());
    let generic = item(&report, &facts, "conditional")?;
    let Kind::FunctionItem {
        item: id,
        signature,
        ..
    } = facts.types.kind(generic).ok_or("kind")?
    else {
        return Err("item".into());
    };
    let args = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(TypeId::primitive(
            crate::types::Primitive::U8,
        ))])?;
    let selected = facts.types.intern(Kind::FunctionItem {
        item: id,
        arguments: args,
        signature,
    })?;
    let goal = predicate(&mut facts, selected, Constness::Const)?;
    let error = select(&mut facts, goal)
        .err()
        .ok_or("unpaid conditional callable accepted")?
        .to_string();
    assert!(error.contains("scoped continuation"), "{error}");
    facts.lifetimes.callables = crate::types::callable::Templates::default();
    assert!(matches!(
        select(&mut facts, goal)?,
        selection::Selection::Deferred
    ));
    Ok(())
}
#[test]
fn ordinary_callable_skips_declaration_reconstruction_and_pending_const_is_not_absence()
-> TestResult {
    let (report, mut facts) = provider()?;
    let actual = item(&report, &facts, "item")?;
    let ordinary = predicate(&mut facts, actual, Constness::Ordinary)?;
    let required = predicate(&mut facts, actual, Constness::Const)?;
    facts.function_indices.clear();
    assert!(matches!(
        select(&mut facts, ordinary)?,
        selection::Selection::Proven(_)
    ));
    assert!(matches!(
        select(&mut facts, required)?,
        selection::Selection::Deferred
    ));
    assert_eq!(size_of::<super::super::Matched>(), 20);
    assert_eq!(selection::callable_layouts(), (1312, 80));
    println!(
        "Matched={} Eligibility={}",
        size_of::<super::super::Matched>(),
        size_of::<Eligibility>()
    );
    Ok(())
}

#[path = "host_work.rs"]
mod work;
