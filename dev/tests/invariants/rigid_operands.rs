//! Original template classification never becomes concrete caller permission.
use super::*;
use crate::types::{GenericArgs, Kind, Primitive, TypeId};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn fixture() -> Result<(Declarations, Initializer, Body, Contract)> {
    let report = crate::test_source::resolve(
        "trait Pick<'a>{type Out;const V:Self::Out;}struct Tag;impl<'a>Pick<'a>for Tag{type Out=&'a u8;const V:Self::Out=&7;}struct S<'a,T>(&'a u8,T);impl<'a,T:Pick<'a>>S<'a,T>{const C:T::Out={let n=T::V;n};}fn other<U>(){}fn main(){}",
        true,
    )?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let initializer = *facts.initializers.last().ok_or("missing initializer")?;
    let template =
        crate::body::templates::Prepared::default().initializer(&report, &facts, &initializer)?;
    let contract = crate::body::regions::Collector::default().initializer(&facts, &initializer)?;
    Ok((facts, initializer, template.body, contract))
}

fn operand(body: &Body) -> Result<Instance> {
    body.values
        .iter()
        .find_map(|value| match value.kind {
            crate::body::ir::Rvalue::AssociatedConstant(instance) => Some(instance),
            _ => None,
        })
        .ok_or_else(|| "missing dependent operand".into())
}

fn parameters(
    facts: &mut Declarations,
    initializer: Initializer,
    contract: &Contract,
) -> Result<Parameters> {
    let allowed = facts
        .generics
        .parameters_in_scope(initializer.definition)
        .filter(|parameter| !matches!(parameter.argument, Argument::Lifetime(_)))
        .map(|parameter| crate::types::parameters::argument_id(&facts.types, parameter.argument))
        .collect::<std::result::Result<HashSet<_>, _>>()?;
    Ok(Parameters {
        allowed,
        owned: owned_regions(facts, contract)?,
        uses: Uses::default(),
        regions: bound_uses::Uses::including_free(),
        universals: contract.universals,
    })
}

#[test]
fn original_dispatch_keeps_full_operands_and_concrete_dispatch_stays_strict() -> Result {
    let (mut facts, initializer, mut body, mut contract) = fixture()?;
    let expected = operand(&body)?;
    let mut concrete = body.clone();
    let mut concrete_contract = contract.clone();
    assert!(super::super::resolve(&mut facts, &mut concrete, &mut concrete_contract).is_err());
    super::initializer(&mut facts, &mut body, &mut contract, &initializer)?;
    assert_eq!(operand(&body)?, expected);
    assert!(body.instance.is_none());
    Ok(())
}

#[test]
fn template_entry_rejects_foreign_source_key_and_changed_declared_result() -> Result {
    let (mut facts, initializer, body, contract) = fixture()?;
    let mut wrong_body = body.clone();
    wrong_body.source.node.0 += 1;
    assert!(
        super::initializer(
            &mut facts,
            &mut wrong_body,
            &mut contract.clone(),
            &initializer
        )
        .is_err()
    );
    let mut wrong_contract = contract.clone();
    wrong_contract.key = BodyKey::Anonymous(body.source);
    assert!(
        super::initializer(
            &mut facts,
            &mut body.clone(),
            &mut wrong_contract,
            &initializer
        )
        .is_err()
    );
    let mut wrong_body = body.clone();
    wrong_body.result = TypeId::primitive(Primitive::U16);
    let mut forged = initializer;
    forged.ty = wrong_body.result;
    assert!(
        super::initializer(&mut facts, &mut wrong_body, &mut contract.clone(), &forged).is_err()
    );
    Ok(())
}

#[test]
fn only_actual_own_parameters_defer_and_lifetime_only_operands_do_not() -> Result {
    let (mut facts, initializer, body, contract) = fixture()?;
    let input = operand(&body)?;
    let mut parameters = parameters(&mut facts, initializer, &contract)?;
    assert!(parameters.deferred(&mut facts, input)?);
    let arguments = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 0)?),
        Argument::Type(TypeId::primitive(Primitive::U8)),
    ])?;
    assert!(!parameters.deferred(&mut facts, Instance { arguments, ..input })?);
    let foreign = facts
        .functions
        .iter()
        .flat_map(|function| facts.generics.parameters_in_scope(function.definition))
        .filter_map(|parameter| {
            crate::types::parameters::argument_id(&facts.types, parameter.argument).ok()
        })
        .find(|id| !parameters.allowed.contains(id))
        .ok_or("missing unrelated parameter")?;
    let ty = facts.types.intern(Kind::Parameter(foreign))?;
    let arguments = facts.types.intern_arguments(&[Argument::Type(ty)])?;
    assert!(parameters.deferred(&mut facts, Instance { arguments, ..input })?);
    assert!(parameters.validate(&mut facts).is_err());
    let mut foreign_body = body.clone();
    for value in &mut foreign_body.values {
        if let crate::body::ir::Rvalue::AssociatedConstant(instance) = &mut value.kind {
            instance.arguments = arguments;
        }
    }
    assert!(
        super::initializer(
            &mut facts,
            &mut foreign_body,
            &mut contract.clone(),
            &initializer
        )
        .is_err()
    );
    assert_ne!(arguments, GenericArgs::EMPTY);
    Ok(())
}

#[test]
fn declared_source_aliases_do_not_admit_missing_or_foreign_ownership() -> Result {
    let (mut facts, initializer, body, contract) = fixture()?;
    let input = operand(&body)?;
    let mut parameters = parameters(&mut facts, initializer, &contract)?;
    assert!(parameters.deferred(&mut facts, input)?);
    let region = facts
        .types
        .arguments(input.arguments)
        .and_then(|mut args| {
            args.find_map(|argument| match argument {
                Argument::Lifetime(region) if region.bound_parts().is_none() => Some(region),
                _ => None,
            })
        })
        .ok_or("missing source-local alias")?;
    assert!(parameters.owned.remove(&region));
    assert!(parameters.deferred(&mut facts, input).is_err());
    let referent = facts
        .types
        .arguments(input.arguments)
        .and_then(|mut args| {
            args.find_map(|a| match a {
                Argument::Type(ty) => Some(ty),
                _ => None,
            })
        })
        .ok_or("missing original type parameter")?;
    let nested = facts.types.intern(Kind::Reference {
        region,
        mutable: false,
        referent,
    })?;
    let arguments = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::bound(0, 0)?),
        Argument::Type(nested),
    ])?;
    assert!(parameters.deferred(&mut facts, Instance { arguments, ..input })?);
    let before = facts.body_work.canonical_steps;
    assert!(parameters.validate(&mut facts).is_err());
    assert!(facts.body_work.canonical_steps > before);
    let mut record = facts.body_regions[region.0 as usize - 1];
    record.body.node.0 += 1;
    facts.body_regions[region.0 as usize - 1] = record;
    assert!(owned_regions(&mut facts, &contract).is_err());
    Ok(())
}

fn routing_work(width: usize, repeated: bool) -> Result<usize> {
    let names = (0..width).map(|i| format!("P{i}")).collect::<Vec<_>>();
    let parameters = names.join(",");
    let bounds = names
        .iter()
        .map(|name| format!("{name}:Pick<'a>"))
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        "trait Pick<'a>{{type Out;const V:Self::Out;}}\
         struct S<'a,T,{parameters}>(&'a u8,T,{parameters});\
         impl<'a,T:Pick<'a>,{bounds}>S<'a,T,{parameters}>\
         {{const C:T::Out={{let n=T::V;n}};}}fn main(){{}}"
    );
    let report = crate::test_source::resolve(&source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let initializer = *facts.initializers.last().ok_or("missing initializer")?;
    let mut body = crate::body::templates::Prepared::default()
        .initializer(&report, &facts, &initializer)?
        .body;
    let mut contract =
        crate::body::regions::Collector::default().initializer(&facts, &initializer)?;
    let input = operand(&body)?;
    let value = body
        .values
        .iter()
        .find(|value| matches!(value.kind, crate::body::ir::Rvalue::AssociatedConstant(_)))
        .copied()
        .ok_or("missing operand value")?;
    let parameters = facts
        .generics
        .parameters_in_scope(initializer.definition)
        .filter_map(|parameter| match parameter.argument {
            Argument::Type(ty) => Some(ty),
            _ => None,
        })
        .skip(1)
        .collect::<Vec<_>>();
    assert_eq!(parameters.len(), width);
    for index in 0..width {
        let ty = parameters[if repeated { 0 } else { index }];
        let arguments = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Type(ty),
        ])?;
        let mut copy = value;
        copy.kind = crate::body::ir::Rvalue::AssociatedConstant(Instance { arguments, ..input });
        body.values.push(copy);
    }
    let before = facts.body_work.canonical_steps;
    super::initializer(&mut facts, &mut body, &mut contract, &initializer)?;
    Ok(facts.body_work.canonical_steps - before)
}

#[test]
fn distinct_original_operands_validate_once_with_linear_counted_work() -> Result {
    let mut measurements = Vec::new();
    for width in [8, 32, 128] {
        let work = routing_work(width, false)?;
        println!("distinct original operands {width}: {work}");
        assert_eq!(work, 7 * width + 12);
        measurements.push(work);
    }
    assert!(measurements[2] <= 4 * measurements[1] + 32);
    Ok(())
}

#[test]
fn repeated_original_operands_reuse_the_same_occurrence_graph() -> Result {
    let mut measurements = Vec::new();
    for width in [8, 32, 128] {
        let work = routing_work(width, true)?;
        println!("repeated original operands {width}: {work}");
        assert_eq!(work, 6 * width + 13);
        measurements.push(work);
    }
    assert!(measurements[2] <= 4 * measurements[1] + 32);
    Ok(())
}

fn anonymous_fixture() -> Result<(Declarations, Body, Contract)> {
    let report = crate::test_source::resolve(
        "trait A{const V:u8;}fn get<T:A>()->u8{const{T::V}}fn main(){}",
        true,
    )?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    let source = facts
        .inline_constants
        .as_deref()
        .and_then(|r| r.bodies().next())
        .ok_or("missing anonymous body")?
        .source;
    let template =
        crate::body::templates::Prepared::default().anonymous(&report, &facts, source)?;
    let contract = crate::body::regions::Collector::default().expression(&facts, source)?;
    Ok((facts, template.body, contract))
}
#[test]
fn original_anonymous_dispatch_keeps_the_exact_deferred_operand() -> Result {
    let (mut facts, mut body, mut contract) = anonymous_fixture()?;
    let expected = operand(&body)?;
    assert!(super::super::resolve(&mut facts, &mut body.clone(), &mut contract.clone()).is_err());
    super::anonymous(&mut facts, &mut body, &mut contract)?;
    assert_eq!(operand(&body)?, expected);
    Ok(())
}
#[test]
fn original_anonymous_entry_rejects_foreign_contract_and_result() -> Result {
    let (mut facts, body, contract) = anonymous_fixture()?;
    let mut wrong = contract.clone();
    wrong.source.node.0 += 1;
    assert!(super::anonymous(&mut facts, &mut body.clone(), &mut wrong).is_err());
    let mut wrong = contract.clone();
    wrong.universals += 1;
    assert!(super::anonymous(&mut facts, &mut body.clone(), &mut wrong).is_err());
    let mut wrong = body.clone();
    wrong.result = TypeId::primitive(Primitive::U16);
    assert!(super::anonymous(&mut facts, &mut wrong, &mut contract.clone()).is_err());
    let mut wrong = body;
    wrong.source.node.0 += 1;
    assert!(super::anonymous(&mut facts, &mut wrong, &mut contract.clone()).is_err());
    Ok(())
}
