//! Opaque-owner families retain source seals and actual-image formation debts.
use crate::{
    body::{
        self,
        regions::{Contract, Contracts},
    },
    borrow,
    sema::model::Declarations,
    types::{self, Kind, RegionId, TypeId, instance::Instance},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn source() -> Result<String> {
    Ok(std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa24/tests/spec/captures/shared-reference-truncation.rs"),
    )?)
}
struct Fixture {
    facts: Declarations,
    templates: body::templates::Templates,
    inputs: Vec<Contract>,
    identity: types::closure::ClosureId,
    ty: TypeId,
}
fn fixture() -> Result<Fixture> {
    let report = crate::test_source::resolve(&source()?, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let closure = *facts.closures.all().first().ok_or("closure")?;
    let ty = facts
        .node(closure.source)
        .and_then(|n| n.ty)
        .ok_or("closure type")?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let inputs = body::regions::collect(&facts)?.bodies;
    for input in &inputs {
        input.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    // The plain builder keeps the whole original family; specialized lowering
    // uses the complementary family selection instead of duplicating the child.
    let plain = body::build_concrete(&report, &facts)?;
    let owner = closure.owner.ok_or("owner")?;
    for key in [
        body::ir::BodyKey::Declaration(owner),
        body::ir::BodyKey::Closure(closure.identity),
    ] {
        assert!(templates.bodies.iter().any(|t| t.body.key() == key));
        assert!(plain.bodies.iter().any(|b| b.key() == key));
    }
    Ok(Fixture {
        facts,
        templates,
        inputs,
        identity: closure.identity,
        ty,
    })
}
fn checked<'a>(
    facts: &Declarations,
    templates: &'a body::templates::Templates,
    inputs: &[Contract],
) -> Result<borrow::templates::Checked<'a>> {
    let mut copies = types::copy::Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    Ok(borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &Contracts {
            bodies: inputs.to_vec(),
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        },
        templates,
        &[],
        |ty, field| body::drops::nominal_position(facts, ty, field),
    )?)
}
fn publish(facts: &mut Declarations, checked: &borrow::templates::Checked<'_>) -> Result<usize> {
    Ok(checked.visit_closure_conditions(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
        |identity, source, pairs| facts.closures.retain_conditions(identity, source, pairs),
    )?)
}
fn verify_instance(facts: &mut Declarations, result: &body::instantiate::Materialized) -> Result {
    result
        .contract
        .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut queries = types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        &result.contract,
        &result.body,
    )?;
    Ok(())
}
#[test]
fn source_seal_precedes_declared_specialization_and_required_stays_unpaid() -> Result {
    let Fixture {
        mut facts,
        templates,
        inputs,
        identity,
        ty,
    } = fixture()?;
    let checked = checked(&facts, &templates, &inputs)?;
    let (template, input) = checked.closure(identity).ok_or("checked closure")?;
    let instance = Instance::closure(&facts.types, ty)?;
    let mut cleanup = body::instantiate::Cleanup::new(&facts);
    let missing =
        body::instantiate::from_checked_caller(&mut facts, template, instance, input, &mut cleanup)
            .err()
            .ok_or("missing seal accepted")?;
    assert!(missing.contains("checked source execution conditions"));
    assert!(publish(&mut facts, &checked)? > 0);
    assert_eq!(
        facts.closures.conditions(identity).ok_or("catalog")?,
        &[(RegionId::bound(0, 1)?, RegionId::bound(0, 0)?)]
    );
    let required = body::instantiate::with_contract(&mut facts, template, instance, input)?;
    let original_wf = input
        .obligations
        .iter()
        .filter(|o| matches!(o.kind, body::regions::Kind::WellFormed(_)))
        .count();
    let required_wf = required
        .contract
        .obligations
        .iter()
        .filter(|o| matches!(o.kind, body::regions::Kind::WellFormed(_)))
        .count();
    assert!(
        required_wf > original_wf,
        "source and formation duties must both remain"
    );
    let error = verify_instance(&mut facts, &required)
        .err()
        .ok_or("manual instance inherited a catalog premise")?;
    assert!(
        error
            .to_string()
            .contains("required lifetime does not outlive its target"),
        "{error}"
    );
    let declared = body::instantiate::from_checked_caller(
        &mut facts,
        template,
        instance,
        input,
        &mut cleanup,
    )?;
    verify_instance(&mut facts, &declared)?;
    Ok(())
}
fn caller(
    facts: &mut Declarations,
    ty: TypeId,
    source: body::ir::BodyOrigin,
    swapped: bool,
    paid: bool,
) -> Result<Contract> {
    let Some(Kind::Closure {
        identity,
        signature,
        environment,
        ..
    }) = facts.types.kind(ty)
    else {
        return Err("closure kind".into());
    };
    let a = RegionId::bound(0, u16::from(swapped))?;
    let b = RegionId::bound(0, u16::from(!swapped))?;
    let arguments = facts.types.intern_regions(&[a, b])?;
    let ty = facts.types.intern(Kind::Closure {
        identity,
        signature,
        environment,
        arguments,
    })?;
    let receiver = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: ty,
    })?;
    let parameters = facts.types.intern_list(&[receiver])?;
    let selected = facts.types.intern(Kind::Function {
        bound_regions: 0,
        unsafe_: false,
        convention: types::callable::Convention::RUST,
        parameters,
        result: TypeId::primitive(types::Primitive::U8),
    })?;
    let mut input = Contract {
        key: body::ir::BodyKey::Anonymous(crate::test_sites::node(source)?),
        source,
        signature: None,
        universals: 2,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: Vec::new(),
        proofs: Vec::new(),
        builtins: None,
        output: None,
        require_result_formation: false,
    };
    if paid {
        input
            .assumptions
            .push((RegionId::bound(0, 1)?, RegionId::bound(0, 0)?));
    }
    let mut proofs = body::instantiate::ProofCursor::default();
    assert!(
        body::invocation_entries::select_execution(
            facts,
            &mut input,
            source,
            crate::support::call_traits::CallTrait::Fn,
            receiver,
            selected,
            &mut proofs
        )?
        .is_some()
    );
    Ok(input)
}
fn verify_caller(facts: &Declarations, input: Contract) -> Result {
    borrow::regions::check(
        &facts.types,
        &facts.lifetimes.variances,
        &Contracts {
            bodies: vec![input],
            callables: &facts.lifetimes.callables,
            nominal_requirements: &facts.lifetimes.nominal_requirements,
        },
    )?;
    Ok(())
}
#[test]
fn selected_callers_pay_the_exact_actual_image_not_the_formal_pair() -> Result {
    let Fixture {
        mut facts,
        templates,
        inputs,
        ty,
        ..
    } = fixture()?;
    let source = inputs.first().ok_or("source")?.source;
    let missing = caller(&mut facts, ty, source, false, true)
        .err()
        .ok_or("selection lacks its source seal")?;
    assert!(
        missing
            .to_string()
            .contains("checked source execution conditions"),
        "{missing}"
    );
    let checked = checked(&facts, &templates, &inputs)?;
    publish(&mut facts, &checked)?;
    let unpaid = caller(&mut facts, ty, source, false, false)?;
    assert!(unpaid.assumptions.is_empty());
    let actual_pair = (RegionId::bound(0, 1)?, RegionId::bound(0, 0)?);
    assert!(
        unpaid
            .obligations
            .iter()
            .any(|o| matches!(o.kind, body::regions::Kind::Outlives(pair) if pair == actual_pair))
    );
    assert!(verify_caller(&facts, unpaid).is_err());
    let valid = caller(&mut facts, ty, source, false, true)?;
    verify_caller(&facts, valid)?;
    let wrong = caller(&mut facts, ty, source, true, true)?;
    let swapped_pair = (RegionId::bound(0, 0)?, RegionId::bound(0, 1)?);
    assert!(
        wrong
            .obligations
            .iter()
            .any(|o| matches!(o.kind, body::regions::Kind::Outlives(pair) if pair == swapped_pair))
    );
    assert!(
        verify_caller(&facts, wrong).is_err(),
        "swapped actual images reused the formal premise"
    );
    Ok(())
}
