//! Ordinary source requests retain their caller's complete, freshly selected debts.
use super::*;
use crate::{
    body::{self, templates::Template},
    types::{Primitive, arguments::Argument, traits::Application},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const INSTANCE_SOURCE: &str = "pub fn debt<T>(){} pub fn callers(){debt::<u8>();debt::<u16>();}";
fn prepare(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
) -> std::result::Result<(), &'static str> {
    super::prepare(facts, body, contract, None)
}

fn debt_template(
    report: &crate::resolve::expansion::Report,
    facts: &Declarations,
) -> Result<Template> {
    let definition = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "debt")
        .ok_or("actual debt declaration")?;
    let definition = crate::resolve::model::DefId(u32::try_from(definition)?);
    let function = facts
        .functions
        .iter()
        .find(|function| function.definition == definition)
        .ok_or("actual debt function")?;
    Ok(body::templates::Prepared::default().function(report, facts, function)?)
}
fn fixture(extra: &str) -> Result<(Declarations, Template, Contract)> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )? + extra;
    let report = crate::test_source::source_provider(&source, "body_trait_debts")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let template = debt_template(&report, &facts)?;
    let contract = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|input| input.key == template.body.key())
        .ok_or("debt input")?;
    contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    Ok((facts, template, contract))
}
fn sized(facts: &mut Declarations, ty: crate::types::TypeId) -> Result<Predicate> {
    let catalog = facts.source_bindings.as_deref().ok_or("source catalog")?;
    let role = catalog
        .language(crate::support::bindings::LangItem::Sized)
        .ok_or("actual Sized")?;
    assert_eq!(
        catalog.sizing_kind(role.definition)?,
        Some(crate::support::bindings::LangItem::Sized)
    );
    let application = facts.types.intern_trait_application(Application {
        item: crate::types::ItemId(role.definition.0),
        arguments: crate::types::GenericArgs::EMPTY,
    })?;
    Ok(Predicate {
        ty,
        bound: Bound::Trait(application),
    })
}
fn pending(
    contract: &mut Contract,
    environment: Arc<crate::types::parameters::environment::Environment>,
    goals: &[Predicate],
) {
    contract.builtins = Some(Arc::new(
        crate::body::regions::BuiltinRequirements::imported(
            environment,
            goals
                .iter()
                .map(|&goal| (contract.source.into(), goal))
                .collect(),
        ),
    ));
}
fn parameter(facts: &mut Declarations, contract: &Contract) -> Result<crate::types::TypeId> {
    let id = *contract.parameters.first().ok_or("actual type formal")?;
    Ok(facts.types.intern(Kind::Parameter(id))?)
}
fn instance_fixture() -> Result<(Declarations, Template, Contract)> {
    let (mut facts, template, mut original) = fixture(INSTANCE_SOURCE)?;
    let ty = parameter(&mut facts, &original)?;
    let goal = sized(&mut facts, ty)?;
    pending(&mut original, Arc::clone(&template.environment), &[goal]);
    Ok((facts, template, original))
}
fn instance(
    facts: &mut Declarations,
    template: &Template,
    actual: Primitive,
) -> Result<crate::types::instance::Instance> {
    let body::ir::Owner::Function {
        definition,
        signature,
        ..
    } = template.body.owner
    else {
        return Err("actual function".into());
    };
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Type(crate::types::TypeId::primitive(actual))])?;
    let ty = facts.types.intern(Kind::FunctionItem {
        item: crate::types::ItemId(definition.0),
        arguments,
        signature,
    })?;
    if actual != Primitive::Str {
        assert!(
            facts.nodes.iter().flatten().any(|node| node.ty == Some(ty)),
            "positive instance is not an actual typed call"
        );
    }
    Ok(crate::types::instance::Instance::new(&facts.types, ty)?)
}
fn opposed_builtin_row(
    facts: &mut Declarations,
    materialized: &body::instantiate::Materialized,
    original: &Arc<crate::types::parameters::environment::Environment>,
) -> Result {
    let scope = materialized
        .instance_copy_scope_for_test()
        .ok_or("actual trait instance scope")?;
    let environment = materialized
        .contract
        .builtins
        .as_ref()
        .and_then(|input| input.environment.as_ref())
        .ok_or("fresh instance environment")?;
    let source = materialized
        .body
        .source
        .source()
        .ok_or("actual source owner")?;
    let saved = facts.generics.builtin_requirements.insert(
        source,
        crate::sema::model::BuiltinRequirements {
            environment: Some(Arc::new(original.as_ref().clone())),
            predicates: Vec::new(),
            source_copy: None,
        },
    );
    assert!(
        scope
            .check(
                facts,
                &materialized.body,
                &materialized.contract,
                Some(environment)
            )
            .is_err(),
        "mismatched builtin row fell back to header"
    );
    facts
        .generics
        .builtin_requirements
        .get_mut(&source)
        .ok_or("opposed row")?
        .environment = None;
    assert!(
        scope
            .check(
                facts,
                &materialized.body,
                &materialized.contract,
                Some(environment)
            )
            .is_err(),
        "incomplete builtin row fell back to header"
    );
    facts.generics.builtin_requirements.remove(&source);
    if let Some(saved) = saved {
        facts.generics.builtin_requirements.insert(source, saved);
    }
    Ok(())
}

#[test]
fn body_trait_transfer_keeps_original_and_complete_scoped_proof() -> Result {
    let (mut facts, template, mut contract) = fixture("pub fn debt<'a,'b>(x:&'a u8,y:&'b u8){}")?;
    let goal = sized(&mut facts, crate::types::TypeId::primitive(Primitive::U8))?;
    let legacy = Predicate {
        bound: Bound::Sized,
        ..goal
    };
    pending(
        &mut contract,
        Arc::clone(&template.environment),
        &[goal, legacy, goal],
    );
    let original = contract.clone();
    let before = facts.body_work.canonical_steps;
    prepare(&mut facts, &template.body, &mut contract)?;
    assert!(facts.body_work.canonical_steps > before);
    assert_eq!(
        contract
            .builtins
            .as_ref()
            .ok_or("prepared")?
            .predicates
            .len(),
        1
    );
    assert_eq!(
        original
            .builtins
            .as_ref()
            .ok_or("original")?
            .predicates
            .len(),
        3
    );
    assert_eq!(contract.proofs.len(), original.proofs.len() + 1);
    let proof = &contract.proofs.last().ok_or("complete proof")?.proof;
    assert_eq!(proof.regions.len(), contract.universals as usize);
    assert!(proof.equalities.len() >= contract.universals as usize);
    let unchanged = Arc::clone(contract.builtins.as_ref().ok_or("legacy")?);
    let proofs = contract.proofs.len();
    prepare(&mut facts, &template.body, &mut contract)?;
    assert!(Arc::ptr_eq(
        &unchanged,
        contract.builtins.as_ref().ok_or("legacy")?
    ));
    assert_eq!(contract.proofs.len(), proofs);
    Ok(())
}

#[test]
fn body_trait_transfer_failure_preserves_successful_prefix_and_charges_work() -> Result {
    let (mut facts, template, mut contract) = fixture("pub fn debt(){}")?;
    let good = sized(&mut facts, crate::types::TypeId::primitive(Primitive::U8))?;
    let bad = sized(&mut facts, crate::types::TypeId::primitive(Primitive::Str))?;
    pending(
        &mut contract,
        Arc::clone(&template.environment),
        &[good, bad],
    );
    let unchanged = Arc::clone(contract.builtins.as_ref().ok_or("pending")?);
    let proofs = contract.proofs.len();
    for _ in 0..2 {
        let before = facts.body_work.canonical_steps;
        assert_eq!(
            prepare(&mut facts, &template.body, &mut contract),
            Err("body trait requirement is not proved")
        );
        assert!(facts.body_work.canonical_steps > before);
        assert!(Arc::ptr_eq(
            &unchanged,
            contract.builtins.as_ref().ok_or("pending")?
        ));
        assert_eq!(contract.proofs.len(), proofs);
    }
    Ok(())
}

#[test]
fn body_trait_proof_cannot_reuse_another_callers_environment() -> Result {
    let (mut facts, template, contract) = fixture("pub fn debt<T:?Sized>(){}")?;
    let ty = parameter(&mut facts, &contract)?;
    let goal = sized(&mut facts, ty)?;
    let paid = Arc::new(
        template
            .environment
            .as_ref()
            .clone()
            .with_clauses(&facts.types, &[goal.into()])?,
    );
    for environment in [&paid, &template.environment, &paid, &template.environment] {
        let before = facts.body_work.canonical_steps;
        let selected = crate::sema::compiler_items::prove_body_requirements(
            &mut facts,
            Some(Arc::clone(environment)),
            contract.universals,
            1,
            std::iter::once(goal),
        );
        assert_eq!(selected.is_ok(), Arc::ptr_eq(environment, &paid));
        assert!(facts.body_work.canonical_steps > before);
    }
    Ok(())
}

#[test]
fn body_trait_transfer_rejects_wrong_owner_signature_and_instance() -> Result {
    let (mut facts, template, mut contract) =
        fixture("pub fn debt(){} pub fn other(){} pub fn use_other(){other();}")?;
    let goal = sized(&mut facts, crate::types::TypeId::UNIT)?;
    pending(&mut contract, Arc::clone(&template.environment), &[goal]);
    let foreign = facts.nodes.iter().flatten().filter_map(|node| node.ty).find(|&ty| {
        matches!(facts.types.kind(ty), Some(Kind::FunctionItem { item, .. }) if contract.key != crate::body::ir::BodyKey::Declaration(crate::resolve::model::DefId(item.0)))
            && facts.types.contains_parameters(ty) == Some(false)
    }).ok_or("actual foreign callable")?;
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(foreign) else {
        return Err("actual function item".into());
    };
    let foreign_source = facts
        .functions
        .iter()
        .find(|function| function.definition.0 == item.0)
        .and_then(|function| function.body)
        .ok_or("actual foreign source")?;
    for change in 0..3 {
        let mut body = template.body.clone();
        contract.signature = body.owner.signatures().map(|(_, owned)| owned);
        match change {
            0 => contract.signature = None,
            1 => {
                body.source = foreign_source.into();
            }
            _ => {
                contract.signature = body.owner.signatures().map(|(_, owned)| owned);
                body.instance = Some(crate::types::instance::Instance::new(
                    &facts.types,
                    foreign,
                )?);
            }
        }
        assert!(prepare(&mut facts, &body, &mut contract).is_err());
        assert_eq!(
            contract
                .builtins
                .as_ref()
                .ok_or("pending")?
                .predicates
                .len(),
            1
        );
    }
    Ok(())
}

#[test]
fn body_trait_original_premise_does_not_pay_unused_str_instance() -> Result {
    let (mut facts, template, original) = instance_fixture()?;
    let mut prepared = original.clone();
    prepare(&mut facts, &template.body, &mut prepared)?;
    for (actual, expected) in [
        (Primitive::U8, true),
        (Primitive::Str, false),
        (Primitive::U16, true),
    ] {
        let instance = instance(&mut facts, &template, actual)?;
        let specialized =
            body::instantiate::with_contract(&mut facts, &template, instance, &original);
        let selected = specialized
            .map_err(String::from)
            .and_then(|mut materialized| materialized.resolve(&mut facts));
        assert_eq!(
            selected.is_ok(),
            expected,
            "actual={actual:?} result={selected:?}"
        );
    }
    assert_eq!(
        original
            .builtins
            .as_ref()
            .ok_or("original debts")?
            .predicates
            .len(),
        1
    );
    Ok(())
}

#[test]
fn materialized_trait_scope_rejects_unused_instance_env_and_foreign_arena() -> Result {
    let (mut facts, template, original) = instance_fixture()?;
    let valid = instance(&mut facts, &template, Primitive::U8)?;
    let wrong = instance(&mut facts, &template, Primitive::Str)?;
    let signature = |facts: &Declarations, instance: crate::types::instance::Instance| match facts
        .types
        .kind(instance.ty())
    {
        Some(Kind::FunctionItem { signature, .. }) => Some(signature),
        _ => None,
    };
    assert_eq!(
        signature(&facts, valid),
        signature(&facts, wrong),
        "unused type changes the public signature"
    );
    let materialized = body::instantiate::with_contract(&mut facts, &template, valid, &original)?;
    let scope = materialized
        .instance_copy_scope_for_test()
        .ok_or("actual trait instance scope")?;
    let environment = materialized
        .contract
        .builtins
        .as_ref()
        .and_then(|input| input.environment.as_ref())
        .ok_or("fresh instance environment")?;
    scope.check(
        &mut facts,
        &materialized.body,
        &materialized.contract,
        Some(environment),
    )?;
    let mut forged = materialized.body.clone();
    forged.instance = Some(wrong);
    let mut contract = materialized.contract.clone();
    assert!(
        materialized
            .prepare_source_copy(&mut facts, &forged, &mut contract)
            .is_err()
    );
    let mut raw = materialized.contract.clone();
    let error = crate::body::copy_requirements::prepare(&mut facts, &materialized.body, &mut raw)
        .err()
        .ok_or("raw materialized debt got permission")?;
    assert_eq!(
        error,
        "materialized trait requirements need their exact instance scope"
    );
    let replacement = Arc::new(environment.as_ref().clone());
    assert!(
        scope
            .check(
                &mut facts,
                &materialized.body,
                &materialized.contract,
                Some(&replacement)
            )
            .is_err()
    );
    opposed_builtin_row(&mut facts, &materialized, &template.environment)?;
    let (mut foreign, _, _) = instance_fixture()?;
    let before = foreign.body_work.canonical_steps;
    assert!(
        scope
            .check(
                &mut foreign,
                &materialized.body,
                &materialized.contract,
                Some(environment)
            )
            .is_err()
    );
    assert!(foreign.body_work.canonical_steps > before);
    scope.check(
        &mut facts,
        &materialized.body,
        &materialized.contract,
        Some(environment),
    )?;
    println!(
        "ordinary_trait_instance_scope_bytes={}",
        scope.retained_bytes()
    );
    Ok(())
}

#[test]
fn body_trait_request_budget_rejects_before_selection_and_accounts_failure() -> Result {
    let (mut facts, _, _) = fixture("pub fn debt(){}")?;
    let goal = sized(&mut facts, crate::types::TypeId::UNIT)?;
    let before = facts.body_work.canonical_steps;
    let selected = crate::sema::compiler_items::prove_body_requirements(
        &mut facts,
        None,
        0,
        1,
        std::iter::repeat_n(goal, crate::support::crates::MAX_COMPONENTS + 1),
    );
    assert_eq!(
        selected.err(),
        Some("body trait requirement work budget exceeded")
    );
    assert!(facts.body_work.canonical_steps > before);
    Ok(())
}

#[test]
fn body_trait_repeated_requests_have_bounded_work_and_separate_proofs() -> Result {
    let (mut facts, template, contract) = fixture("pub fn debt<'a>(x:&'a u8){}")?;
    let goal = sized(&mut facts, crate::types::TypeId::UNIT)?;
    let mut prior = None;
    for count in [8, 32, 128] {
        let before = facts.body_work.canonical_steps;
        let first = crate::sema::compiler_items::prove_body_requirements(
            &mut facts,
            Some(Arc::clone(&template.environment)),
            contract.universals,
            100,
            std::iter::repeat_n(goal, count),
        )?;
        let steps = facts.body_work.canonical_steps - before;
        let second = crate::sema::compiler_items::prove_body_requirements(
            &mut facts,
            Some(Arc::clone(&template.environment)),
            contract.universals,
            200,
            std::iter::repeat_n(goal, count),
        )?;
        assert_eq!(first.regions, 100..100 + contract.universals);
        assert_eq!(second.regions, 200..200 + contract.universals);
        assert_eq!(first.equalities.len(), second.equalities.len());
        if let Some(previous) = prior {
            assert!(steps <= previous * 4, "count={count} steps={steps}");
        }
        prior = Some(steps);
        println!(
            "body_trait_requests={count} canonical_steps={steps} proof_regions={}",
            first.regions.len()
        );
    }
    Ok(())
}

#[test]
fn body_trait_selected_lifetime_debts_reach_ordinary_region_proof() -> Result {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-functions-transport/minimal-provider.rs",
    )? + "pub trait Gate{} impl<'a,'b> Gate for &'a &'b u8 where 'a:'b {} pub fn debt<'a,'b>(x:&'a &'b u8){}";
    let report = crate::test_source::source_provider(&source, "body_trait_debts")?;
    let gate = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "Gate")
        .ok_or("actual trait")?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let template = debt_template(&report, &facts)?;
    let mut contract = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|input| input.key == template.body.key())
        .ok_or("actual contract")?;
    let ty = template
        .body
        .locals
        .iter()
        .find(|local| matches!(local.kind, body::ir::LocalKind::Parameter(0)))
        .ok_or("actual parameter local")?
        .ty;
    let application = facts.types.intern_trait_application(Application {
        item: crate::types::ItemId(u32::try_from(gate)?),
        arguments: crate::types::GenericArgs::EMPTY,
    })?;
    pending(
        &mut contract,
        Arc::clone(&template.environment),
        &[Predicate {
            ty,
            bound: Bound::Trait(application),
        }],
    );
    let proofs = contract.proofs.len();
    prepare(&mut facts, &template.body, &mut contract)?;
    assert_eq!(contract.proofs.len(), proofs + 1);
    assert!(
        !contract
            .proofs
            .last()
            .ok_or("selected proof")?
            .proof
            .outlives
            .is_empty()
    );
    contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let check = |facts: &Declarations, contract: &Contract| {
        let mut queries = crate::types::outlives::Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        crate::borrow::regions::check_one(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            contract,
            &template.body,
        )
    };
    assert!(
        check(&facts, &contract).is_err(),
        "selected debt was silently discarded"
    );
    contract.assumptions.push((
        crate::types::RegionId::bound(0, 0)?,
        crate::types::RegionId::bound(0, 1)?,
    ));
    check(&facts, &contract)?;
    Ok(())
}
