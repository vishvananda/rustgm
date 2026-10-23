//! Only completed allocations from the exact normalization operation gain diagnostic rows.
use super::*;
use crate::{
    body::ir::Owner,
    sema::values::execution::with_namespace,
    types::{
        Kind as Type, Primitive, arguments::Argument, instance::Instance,
        lifetime_proof::Obligations,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture() -> Result<(Declarations, Body, Contract)> {
    let report = crate::test_source::resolve(
        "struct S<'a>(&'a u8);impl<'a>S<'a>{const C:u8=7;}fn f<'a,T>(_:&'a u8)->u8{S::<'a>::C}fn g<'a,T>(_:&'a u8)->u8{S::<'a>::C}fn main(){}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let function = facts
        .functions
        .iter()
        .find(|f| {
            f.body
                .is_some_and(|s| crate::sema::values::execution::caller_published(&facts, s))
        })
        .copied()
        .ok_or("missing published source")?;
    let source = function.body.ok_or("missing function body")?;
    let ty = facts.definitions[function.definition.0 as usize].ok_or("missing function type")?;
    let Some(Type::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(ty)
    else {
        return Err("missing callable".into());
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("missing arguments")?
        .map(|a| match a {
            Argument::Type(_) => Argument::Type(TypeId::primitive(Primitive::U8)),
            other => other,
        })
        .collect::<Vec<_>>();
    let arguments = facts.types.intern_arguments(&arguments)?;
    let ty = facts.types.intern(Type::FunctionItem {
        item,
        arguments,
        signature,
    })?;
    let instance = Instance::new(&facts.types, ty)?;
    let mut body = Body::new(
        Owner::Function {
            definition: function.definition,
            signature,
            execution: None,
        },
        source,
        TypeId::primitive(Primitive::U8),
    );
    body.instance = Some(instance);
    with_namespace(
        &mut facts,
        source,
        CallerInstance::Callable(instance),
        1,
        &[],
        0,
        |_, _| Ok(()),
    )?;
    let contract = Contract {
        key: body.key(),
        source: source.into(),
        signature: None,
        universals: 1,
        assumptions: Vec::new(),
        parameters: Vec::new(),
        type_assumptions: Vec::new(),
        obligations: Vec::new(),
        proofs: Vec::new(),
        builtins: None,
        output: None,
        require_result_formation: false,
    };
    Ok((facts, body, contract))
}
fn frontier(facts: &Declarations, source: crate::body::ir::BodyOrigin) -> Result<u32> {
    Ok(super::frontier(facts, crate::test_sites::node(source)?)?)
}

fn proof(source: impl Into<ProofSite>, range: Range<u32>) -> Proof {
    let mut proof = Obligations::new(range.start);
    proof.regions.end = range.end;
    Proof::new(source, proof)
}

#[test]
fn completed_and_resumed_proofs_reserve_only_their_actual_owned_ranges() -> Result {
    let (mut facts, body, mut contract) = fixture()?;
    let first = frontier(&facts, body.source)?;
    contract.proofs.push(proof(body.source, first..first));
    let mut owner = Operation::begin(&mut facts, &body, &contract)?.ok_or("missing owner")?;
    contract.proofs.push(proof(body.source, first..first + 2));
    owner.record(1, &contract.proofs[1], first..first + 2)?;
    contract.proofs[0].proof.resume_at(first + 2)?;
    contract.proofs[0].proof.regions.end = first + 3;
    owner.record(0, &contract.proofs[0], first + 2..first + 3)?;
    owner.finish(&mut facts, &body, &contract, first + 3)?;
    assert_eq!(frontier(&facts, body.source)?, first + 3);
    assert_eq!(owner.rows.len(), 2);
    for (_, region) in facts
        .body_regions
        .regions_after(crate::test_sites::node(body.source)?, first as usize - 1)
    {
        assert_eq!(region.body, crate::test_sites::node(body.source)?);
        assert_eq!(region.source, crate::test_sites::node(body.source)?);
    }
    eprintln!(
        "retained_rows=3 recorder_capacity={} work={}",
        owner.rows.capacity() * size_of::<Allocation>(),
        owner.work
    );
    Ok(())
}

#[test]
fn gaps_foreign_sources_stale_proofs_and_incomplete_records_do_not_publish() -> Result {
    let (mut facts, body, mut contract) = fixture()?;
    let mut owner = Operation::begin(&mut facts, &body, &contract)?.ok_or("missing owner")?;
    let first = owner.first;
    let evidence = proof(body.source, first..first + 2);
    assert!(owner.record(0, &evidence, first + 1..first + 2).is_err());
    let mut foreign = evidence.clone();
    foreign.source = crate::test_sites::next_node(foreign.source)?.into();
    assert!(owner.record(0, &foreign, first..first + 2).is_err());
    let mut gapped = evidence.clone();
    gapped.proof = Obligations::new(first);
    gapped.proof.regions.end = first + 1;
    gapped.proof.resume_at(first + 2)?;
    gapped.proof.regions.end = first + 3;
    assert!(owner.record(0, &gapped, first..first + 3).is_err());
    assert!(
        owner
            .finish(&mut facts, &body, &contract, first + 2)
            .is_err()
    );
    owner.record(0, &evidence, first..first + 2)?;
    assert!(
        owner
            .finish(&mut facts, &body, &contract, first + 2)
            .is_err()
    );
    contract.proofs.push(evidence);
    contract.proofs[0].source = crate::test_sites::next_node(contract.proofs[0].source)?.into();
    assert!(
        owner
            .finish(&mut facts, &body, &contract, first + 2)
            .is_err()
    );
    contract.proofs[0].source = body.source.into();
    contract.proofs[0].proof.regions.end -= 1;
    assert!(
        owner
            .finish(&mut facts, &body, &contract, first + 2)
            .is_err()
    );
    assert_eq!(frontier(&facts, body.source)?, first);
    Ok(())
}

#[test]
fn recorder_budget_and_empty_operations_preserve_the_frontier() -> Result {
    let (mut facts, body, contract) = fixture()?;
    let mut owner = Operation::begin(&mut facts, &body, &contract)?.ok_or("missing owner")?;
    let first = owner.first;
    owner.bytes = 0;
    assert!(
        owner
            .record(0, &proof(body.source, first..first + 1), first..first + 1)
            .is_err()
    );
    assert!(owner.rows.is_empty());
    owner.finish(&mut facts, &body, &contract, first)?;
    assert_eq!(frontier(&facts, body.source)?, first);
    assert!(owner.work > 0);
    Ok(())
}

#[test]
fn distinct_instances_and_interleaved_normalization_cannot_reuse_operation_ownership() -> Result {
    let (mut facts, body, mut contract) = fixture()?;
    let mut first = Operation::begin(&mut facts, &body, &contract)?.ok_or("missing owner")?;
    let Some(Type::FunctionItem {
        item, signature, ..
    }) = facts
        .types
        .kind(body.instance.ok_or("missing instance")?.ty())
    else {
        return Err("missing callable".into());
    };
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Type(TypeId::primitive(Primitive::U16))])?;
    let ty = facts.types.intern(Type::FunctionItem {
        item,
        arguments,
        signature,
    })?;
    let instance = Instance::new(&facts.types, ty)?;
    let mut other = body.clone();
    other.instance = Some(instance);
    with_namespace(
        &mut facts,
        crate::test_sites::node(other.source)?,
        CallerInstance::Callable(instance),
        1,
        &[],
        0,
        |_, _| Ok(()),
    )?;
    assert!(
        first
            .finish(&mut facts, &other, &contract, first.first)
            .is_err()
    );
    let mut second =
        Operation::begin(&mut facts, &other, &contract)?.ok_or("missing second owner")?;
    contract
        .proofs
        .push(proof(body.source, second.first..second.first + 1));
    second.record(0, &contract.proofs[0], second.first..second.first + 1)?;
    second.finish(&mut facts, &other, &contract, second.first + 1)?;
    assert!(
        first
            .finish(&mut facts, &body, &contract, first.first)
            .is_err()
    );
    let after = frontier(&facts, body.source)?;
    contract.proofs.clear();
    let mut resumed =
        Operation::begin(&mut facts, &body, &contract)?.ok_or("missing fresh owner")?;
    contract.proofs.push(proof(body.source, after..after + 1));
    resumed.record(0, &contract.proofs[0], after..after + 1)?;
    resumed.finish(&mut facts, &body, &contract, after + 1)?;
    assert_eq!(frontier(&facts, body.source)?, after + 1);
    Ok(())
}

#[test]
fn allocation_rows_and_validation_work_grow_with_actual_resumed_proofs() -> Result {
    let mut previous = None;
    for count in [8, 32, 128] {
        let (mut facts, body, mut contract) = fixture()?;
        let first = frontier(&facts, body.source)?;
        for _ in 0..count {
            contract.proofs.push(proof(body.source, first..first));
        }
        let mut owner = Operation::begin(&mut facts, &body, &contract)?.ok_or("missing owner")?;
        contract.proofs.push(proof(body.source, first..first + 1));
        owner.record(count, &contract.proofs[count], first..first + 1)?;
        let mut next = first + 1;
        for index in 0..count {
            contract.proofs[index].proof.resume_at(next)?;
            contract.proofs[index].proof.regions.end = next + 1;
            owner.record(index, &contract.proofs[index], next..next + 1)?;
            next += 1;
        }
        owner.finish(&mut facts, &body, &contract, next)?;
        let rows = usize::try_from(next - first)?;
        assert_eq!(rows, count + 1);
        assert_eq!(owner.rows.len(), rows);
        if let Some((old_count, old_work)) = previous {
            assert_eq!((owner.work - old_work) / (count - old_count), 5);
        }
        eprintln!(
            "resumed_proofs={count} diagnostic_rows={rows} payload_bytes={} recorder_bytes={} work={}",
            rows * size_of::<crate::sema::model::BodyRegion>(),
            owner.rows.capacity() * size_of::<Allocation>(),
            owner.work
        );
        previous = Some((count, owner.work));
    }
    Ok(())
}

#[test]
fn unrelated_source_normalization_keeps_a_parked_operation_and_claims_only_its_rows() -> Result {
    let (mut facts, body, contract) = fixture()?;
    let mut parked = Operation::begin(&mut facts, &body, &contract)?.ok_or("missing caller")?;
    let other = facts
        .functions
        .iter()
        .find(|f| {
            f.body.is_some_and(|s| {
                crate::body::ir::BodyOrigin::from(s) != body.source
                    && crate::sema::values::execution::caller_published(&facts, s)
            })
        })
        .copied()
        .ok_or("missing unrelated caller")?;
    let source = other.body.ok_or("missing other source")?;
    let ty = facts.definitions[other.definition.0 as usize].ok_or("missing other type")?;
    let Some(Type::FunctionItem {
        item,
        arguments,
        signature,
    }) = facts.types.kind(ty)
    else {
        return Err("missing other callable".into());
    };
    let arguments = facts
        .types
        .arguments(arguments)
        .ok_or("missing other arguments")?
        .map(|a| match a {
            Argument::Type(_) => Argument::Type(TypeId::primitive(Primitive::U8)),
            other => other,
        })
        .collect::<Vec<_>>();
    let arguments = facts.types.intern_arguments(&arguments)?;
    let ty = facts.types.intern(Type::FunctionItem {
        item,
        arguments,
        signature,
    })?;
    let instance = Instance::new(&facts.types, ty)?;
    let mut other_body = Body::new(
        Owner::Function {
            definition: other.definition,
            signature,
            execution: None,
        },
        source,
        body.result,
    );
    other_body.instance = Some(instance);
    with_namespace(
        &mut facts,
        source,
        CallerInstance::Callable(instance),
        1,
        &[],
        0,
        |_, _| Ok(()),
    )?;
    let mut other_contract = contract.clone();
    other_contract.source = source.into();
    other_contract.key = other_body.key();
    let mut operation = Operation::begin(&mut facts, &other_body, &other_contract)?
        .ok_or("missing unrelated owner")?;
    let first = operation.first;
    other_contract.proofs.push(proof(source, first..first + 1));
    operation.record(0, &other_contract.proofs[0], first..first + 1)?;
    operation.finish(&mut facts, &other_body, &other_contract, first + 1)?;
    // The unrelated source numbers its own unit; only the parked unit's
    // frontier can invalidate the parked operation.
    assert_eq!(frontier(&facts, body.source)?, parked.first);
    facts
        .body_regions
        .allocate(crate::sema::model::BodyRegion {
            body: crate::test_sites::node(body.source)?,
            source: crate::test_sites::node(body.source)?,
            origin: crate::sema::model::BodyRegionOrigin::Inference,
        })?;
    assert!(
        parked
            .finish(&mut facts, &body, &contract, parked.first)
            .is_err()
    );
    let published = facts
        .body_regions
        .get(source, crate::types::RegionId(first))
        .ok_or("missing published region")?;
    assert_eq!(published.body, source);
    assert_eq!(published.source, source);
    assert!(Operation::begin(&mut facts, &body, &contract)?.is_some());
    Ok(())
}
