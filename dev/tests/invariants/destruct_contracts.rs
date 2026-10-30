//! Exact original carriers are immutable defining requirements, not execution grants.
use super::*;
use crate::body::templates::Template;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
#[path = "parametric_cleanups.rs"]
mod functions;
#[path = "destruct_projections.rs"]
mod projections;

fn fixture(
    path: &str,
) -> Result<(
    crate::resolve::expansion::Report,
    Declarations,
    Template,
    Contract,
)> {
    let source = crate::test_source::fixture(path)?;
    let report = crate::test_source::source_provider(&source, "cleanup_contracts")?;
    let crate::pipeline::Analysis {
        facts,
        mut prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "consume"
        })
        .copied()
        .ok_or("actual consume default")?;
    let template = prepared.templates.function(&report, &facts, &function)?;
    let contract = crate::body::regions::Collector::default()
        .function(&facts, &function)?
        .clone();
    Ok((report, facts, template, contract))
}

#[test]
fn required_default_carriers_reject_missing_foreign_and_unprepared_inputs() -> Result {
    let (_report, mut facts, template, mut contract) =
        fixture("pa31/tests/spec/trait-default-effects/cleanup/runtime.rs")?;
    let body = &template.body;
    let state = contract
        .builtins
        .as_deref()
        .and_then(|r| r.defining())
        .ok_or("required state")?;
    let mut work = Work::new(1_000_000);
    assert!(state.checked(body, None, &mut work).is_err());
    assert!(work.steps > 0);
    let mut missing = contract.clone();
    missing.builtins = None;
    assert!(crate::body::copy_requirements::prepare(&mut facts, body, &mut missing).is_err());
    crate::body::copy_requirements::prepare(&mut facts, body, &mut contract)?;
    let builtins = contract.builtins.as_deref().ok_or("prepared carrier")?;
    let state = builtins.defining().ok_or("prepared state")?;
    state.checked(body, builtins.environment.as_ref(), &mut work)?;
    let foreign = Arc::new(template.environment.as_ref().clone());
    assert!(state.check_environment(&foreign, &mut work).is_err());
    assert!(state.checked(body, Some(&foreign), &mut work).is_err());
    let mut changed = body.clone();
    let Owner::Function { definition, .. } = &mut changed.owner else {
        return Err("owner".into());
    };
    *definition = DefId(u32::MAX);
    assert!(
        state
            .checked(&changed, builtins.environment.as_ref(), &mut work)
            .is_err()
    );
    let mut tiny = Work::new(1);
    assert!(
        state
            .checked(body, builtins.environment.as_ref(), &mut tiny)
            .is_err()
    );
    assert!(tiny.steps > 1);
    Ok(())
}

#[test]
fn defining_preparation_records_failed_work_and_invalidates_old_completion() -> Result {
    let (_report, mut facts, template, mut contract) =
        fixture("pa31/tests/spec/trait-default-effects/cleanup/runtime.rs")?;
    crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
    let Owner::Function { definition, .. } = template.body.owner else {
        return Err("function".into());
    };
    facts
        .function_indices
        .remove(&definition)
        .ok_or("actual function index")?;
    let before = facts.body_work.canonical_steps;
    assert!(
        crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract).is_err()
    );
    assert!(facts.body_work.canonical_steps > before);
    assert!(matches!(
        contract.builtins.as_deref().and_then(|r| r.defining()),
        Some(State::Required(_))
    ));
    Ok(())
}

#[test]
fn exact_definition_claims_have_bounded_linear_reads_and_cold_storage() -> Result {
    let (_report, mut facts, template, mut contract) =
        fixture("pa31/tests/spec/trait-default-effects/cleanup/runtime.rs")?;
    crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
    let builtins = contract.builtins.as_deref().ok_or("builtins")?;
    let state = builtins.defining().ok_or("defining")?;
    let Owner::Function { definition, .. } = template.body.owner else {
        return Err("function".into());
    };
    let owner = facts.associated.trait_owner(definition).ok_or("trait")?;
    let ty = facts.associated.schema(owner).ok_or("schema")?.self_type;
    for count in [8, 32, 128] {
        let mut work = Work::new(1_000_000);
        let prepared = state.checked(&template.body, builtins.environment.as_ref(), &mut work)?;
        let setup = work.steps;
        for _ in 0..count {
            prepared.check(&facts.types, ty, &mut work)?;
        }
        assert_eq!(work.steps - setup, count);
        println!(
            "defining_drop_queries={count} setup={setup} visits={} retained={}",
            work.steps - setup,
            state.retained_bytes()
        );
    }
    assert_eq!(size_of::<crate::body::regions::BuiltinRequirements>(), 56);
    println!(
        "BuiltinRequirements={} State={} Work={} Contract={} Body={} Value={} BorrowEnvironment={}",
        size_of::<crate::body::regions::BuiltinRequirements>(),
        size_of::<State>(),
        size_of::<Work>(),
        size_of::<Contract>(),
        size_of::<Body>(),
        size_of::<crate::body::ir::Value>(),
        size_of::<crate::borrow::Environment<'_, crate::support::hash::BuildFxHasher>>()
    );
    Ok(())
}

#[test]
fn no_drop_requires_actual_ready_copy_identity_without_destruct() -> Result {
    let (report, mut facts, template, _contract) =
        fixture("pa31/tests/spec/trait-default-effects/destruction/provider-copy-dropless.rs")?;
    let mut work = Work::new(1_000_000);
    let copy = claims::Role::current(&facts, crate::support::bindings::LangItem::Copy, &mut work)?;
    let claims = claims::Claims::new(
        &facts,
        &template.environment,
        claims::Role::Absent,
        copy,
        &mut work,
    )?;
    let Owner::Function { definition, .. } = template.body.owner else {
        return Err("function".into());
    };
    let owner = facts.associated.trait_owner(definition).ok_or("trait")?;
    let ty = facts.associated.schema(owner).ok_or("schema")?.self_type;
    assert_eq!(claims.get(ty), Some(claims::Claim::NoDrop));
    let pending_catalog =
        crate::sema::compiler_items::Catalog::reserve(&report)?.ok_or("reserved source catalog")?;
    let completed = facts
        .source_bindings
        .replace(pending_catalog)
        .ok_or("completed source catalog")?;
    let pending =
        claims::Role::current(&facts, crate::support::bindings::LangItem::Copy, &mut work)?;
    let pending_claims = claims::Claims::new(
        &facts,
        &template.environment,
        claims::Role::Absent,
        pending,
        &mut work,
    )?;
    assert_eq!(pending_claims.get(ty), Some(claims::Claim::PendingCopy));
    assert!(pending.require().is_err());
    facts.source_bindings = Some(completed);
    let unrelated = claims::Role::Validated(owner);
    assert!(
        claims::Claims::new(
            &facts,
            &template.environment,
            claims::Role::Absent,
            unrelated,
            &mut work
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn non_source_defaults_require_preparation_while_ordinary_bodies_allocate_no_carrier() -> Result {
    for (source, name, default) in [
        (
            "#![feature(const_trait_impl)] const trait Q{fn provided()->u8{7}}",
            "provided",
            true,
        ),
        ("fn ordinary()->u8{7}", "ordinary", false),
    ] {
        let report = crate::test_source::resolve(source, true)?;
        let crate::pipeline::Analysis {
            mut facts,
            mut prepared,
        } = crate::pipeline::analyze_source(&report)?;
        assert!(facts.source_bindings.is_none());
        let function = facts
            .functions
            .iter()
            .find(|f| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[f.definition.0 as usize].name)
                    == name
            })
            .copied()
            .ok_or("function")?;
        let template = prepared.templates.function(&report, &facts, &function)?;
        let mut contract = crate::body::regions::Collector::default()
            .function(&facts, &function)?
            .clone();
        assert_eq!(contract.requires_defining_cleanup_preparation(), default);
        let before = facts.body_work.canonical_steps;
        crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
        if default {
            let builtins = contract
                .builtins
                .as_deref()
                .ok_or("actual default carrier")?;
            builtins.defining().ok_or("actual default")?.checked(
                &template.body,
                builtins.environment.as_ref(),
                &mut Work::new(1_000_000),
            )?;
        } else {
            assert!(contract.builtins.is_none());
            assert_eq!(facts.body_work.canonical_steps, before);
        }
        let before = facts.body_work.canonical_steps;
        let scratch =
            crate::body::copy_requirements::OriginalIndexBudget::new(1_000_000, 64 * 1024 * 1024);
        crate::body::copy_requirements::originals_with_budget(
            &mut facts,
            [&template.body],
            [&mut contract],
            &scratch,
        )?;
        if default {
            assert!(scratch.costs().0 > 0 && scratch.costs().1 > 0);
            assert!(facts.body_work.canonical_steps > before + 1);
            let mut required = crate::body::regions::Collector::default()
                .function(&facts, &function)?
                .clone();
            let payment = crate::body::copy_requirements::OriginalIndexBudget::new(1_000_000, 0);
            let mut previous = (0, 0);
            for _ in 0..2 {
                let before = facts.body_work.canonical_steps;
                assert!(
                    crate::body::copy_requirements::originals_with_budget(
                        &mut facts,
                        [&template.body],
                        [&mut required],
                        &payment,
                    )
                    .is_err()
                );
                assert!(facts.body_work.canonical_steps > before);
                let charged = payment.costs();
                assert!(charged.0 > previous.0 && charged.1 > 0);
                previous = charged;
                assert!(matches!(
                    required.builtins.as_deref().and_then(|r| r.defining()),
                    Some(State::Required(_))
                ));
            }
        } else {
            assert_eq!(facts.body_work.canonical_steps, before + 1);
            assert_eq!(scratch.costs(), (0, 0));
            assert!(contract.builtins.is_none());
        }
    }
    Ok(())
}

fn verify(facts: &mut Declarations, template: Template, contract: Contract) -> Result {
    contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let contracts = crate::body::regions::Contracts {
        bodies: vec![contract],
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let templates = crate::body::templates::Templates::source(vec![template]);
    let custom = facts
        .destructors
        .implementations()
        .iter()
        .map(|i| i.item)
        .collect::<Vec<_>>();
    let mut copies = crate::types::copy::Copies::default();
    crate::borrow::templates::verify(
        &mut crate::borrow::Environment {
            types: &facts.types,
            definitions: &facts.aggregates,
            derives: &facts.derives,
            copies: &mut copies,
            source_copy: facts.uses_source_copy(),
        },
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &custom,
        |ty, field| crate::body::drops::nominal_position(facts, ty, field),
    )?;
    Ok(())
}

#[test]
fn actual_loan_proof_rejects_a_self_consistent_foreign_environment_carrier() -> Result {
    let path = "pa31/tests/spec/trait-default-effects/cleanup/runtime.rs";
    let (_report, mut facts, template, mut contract) = fixture(path)?;
    crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
    verify(&mut facts, template.clone(), contract.clone())?;
    let (_foreign_report, mut foreign_facts, foreign_template, mut foreign) = fixture(path)?;
    crate::body::copy_requirements::prepare(
        &mut foreign_facts,
        &foreign_template.body,
        &mut foreign,
    )?;
    assert_eq!(foreign.key, contract.key);
    assert_eq!(foreign.source, contract.source);
    contract.builtins = foreign.builtins;
    let error = verify(&mut facts, template, contract)
        .err()
        .ok_or("foreign arena environment accepted")?;
    assert!(
        error
            .to_string()
            .contains("another actual template environment"),
        "{error}"
    );
    Ok(())
}

#[test]
fn whole_source_claim_width_and_noise_keep_scoped_work_and_storage_bounded() -> Result {
    let mut baseline = None;
    for (width, noise) in [(8, 0), (32, 0), (128, 0), (8, 128)] {
        let path = format!(
            "pa31/tests/spec/trait-default-effects/destruction/work/claims-width-{width}-noise-{noise}.rs"
        );
        let (_report, mut facts, template, mut contract) = fixture(&path)?;
        let before = facts.body_work.canonical_steps;
        crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
        let steps = facts.body_work.canonical_steps - before;
        let state = contract
            .builtins
            .as_deref()
            .and_then(|r| r.defining())
            .ok_or("prepared defining requirements")?;
        let State::Prepared(prepared) = state else {
            return Err("unprepared defining requirements".into());
        };
        assert_eq!(prepared.claims.len(), width);
        let retained = state.retained_bytes();
        println!(
            "defining_drop_width={width} noise={noise} preparation_steps={steps} retained={retained}"
        );
        if let Some((base_steps, base_bytes)) = baseline {
            if noise == 128 {
                assert_eq!((steps, retained), (base_steps, base_bytes));
            } else {
                assert!(steps <= base_steps * (width / 8) * 2);
                assert!(retained <= base_bytes * (width / 8) * 2);
            }
        } else {
            baseline = Some((steps, retained));
        }
        verify(&mut facts, template, contract)?;
    }
    Ok(())
}
