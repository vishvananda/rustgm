//! Specialization reproves the child without promoting its parent's conclusions.
use super::{Inputs, specialize};
use crate::{
    body::{self, ir::BodyKey, regions::Obligation},
    borrow,
    types::{RegionId, outlives::Queries},
};

#[test]
fn uncaptured_specialization_retains_every_source_lifetime_debt()
-> Result<(), Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve(
        "fn main(){let f:for<'a> fn(&'a u8)->&'a u8=|x|x;let n=7;let _=f(&n);}",
        true,
    )?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let program = body::build_concrete(&report, &facts)?;
    let mut original = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|input| matches!(input.key, BodyKey::Closure(_)))
        .ok_or("missing source closure")?;
    let cfg = program
        .bodies
        .iter()
        .find(|cfg| cfg.key() == original.key)
        .ok_or("missing closure CFG")?;
    assert!(original.universals > 0);
    // This explicitly typed body has no inferred output port. Add an adversarial
    // parent bridge: a parent's static image must not prove a child's universal
    // outlives static when specialization removes the conditional interface.
    let parent = program
        .bodies
        .iter()
        .find(|body| matches!(body.key(), BodyKey::Declaration(_)))
        .ok_or("missing enclosing body")?;
    original.output = Some(std::sync::Arc::new(crate::sema::nested_outputs::Link {
        source: original.source,
        parent: parent.source,
        revision: 1,
        universals: original.universals,
        bridges: Box::new([crate::sema::nested_outputs::Bridge {
            slot: 0,
            parent: RegionId::STATIC,
            output: true,
        }]),
        formation: None,
    }));
    for unpaid in [false, true] {
        let mut source = original.clone();
        if unpaid {
            source.obligations.push(Obligation {
                source: source.source.into(),
                kind: body::regions::Kind::Outlives((RegionId::bound(0, 0)?, RegionId::STATIC)),
            });
        }
        let mut inputs = Inputs {
            caller: crate::sema::values::execution::CallerInstance::Original,
            instance: None,
            bindings: Vec::new(),
            captures: Vec::new(),
            comparison: None,
            connections: None,
        };
        let (_, contract) = specialize(
            &mut facts,
            &source,
            &mut inputs,
            super::super::Premises::Required,
        )?;
        assert!(contract.output.is_none());
        assert!(contract.require_result_formation);
        assert_eq!(contract.assumptions, source.assumptions);
        let mut queries = Queries::new(
            &facts.types,
            &facts.lifetimes.nominal_requirements,
            &facts.lifetimes.callables,
        );
        let proof = borrow::regions::check_one(
            &facts.types,
            &facts.lifetimes.variances,
            &mut queries,
            &contract,
            cfg,
        );
        if unpaid {
            assert!(
                proof.is_err(),
                "specialization discarded a source lifetime debt"
            );
        } else {
            proof?;
        }
    }
    Ok(())
}

struct PortFixture {
    facts: crate::sema::model::Declarations,
    templates: body::templates::Templates,
    original: body::regions::Contract,
    instance: crate::types::instance::Instance,
}
fn port_fixture(source: bool) -> Result<PortFixture, Box<dyn std::error::Error>> {
    use crate::types::{Kind, TypeId, arguments::Argument, instance::Instance};
    let report = if source {
        let source = crate::test_source::fixture(
            "pa31/tests/spec/inherited-input-regions/generic-native.rs",
        )?;
        crate::test_source::source_provider(&source, "port_scope")?
    } else {
        crate::test_source::resolve(
            "fn f<'b,'c,T>(x:&'c &'b T)->&'b T{(|&s|s)(x)}fn main(){let n=7u8;let p=&n;let _=f(&p);}",
            true,
        )?
    };
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let closure = *facts.closures.all().first().ok_or("source closure")?;
    assert_eq!((closure.inherited, closure.universals), (2, 3));
    let inner = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::STATIC,
        referent: TypeId::primitive(crate::types::Primitive::U8),
    })?;
    let outer = facts.types.intern(Kind::Reference {
        mutable: false,
        region: RegionId::bound(1, 11)?,
        referent: inner,
    })?;
    let parameters = facts.types.intern_list(&[outer])?;
    let signature = facts.types.intern(Kind::Function {
        parameters,
        result: inner,
        bound_regions: 0,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let arguments = facts.types.intern_arguments(&[
        Argument::Lifetime(RegionId::STATIC),
        Argument::Lifetime(RegionId::STATIC),
        Argument::Type(TypeId::primitive(crate::types::Primitive::U8)),
    ])?;
    let ty = facts.types.intern(Kind::Closure {
        identity: closure.identity,
        signature,
        arguments,
        environment: TypeId::UNIT,
    })?;
    let instance = Instance::closure(&facts.types, ty)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let original = body::regions::collect(&facts)?
        .bodies
        .into_iter()
        .find(|input| input.key == BodyKey::Closure(closure.identity))
        .ok_or("closure contract")?;
    Ok(PortFixture {
        facts,
        templates,
        original,
        instance,
    })
}

#[test]
fn private_input_instances_reject_changed_call_binders_and_inherited_regions()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::{Kind, arguments::Argument, instance::Instance};
    let PortFixture {
        mut facts,
        instance,
        ..
    } = port_fixture(false)?;
    super::super::inputs::select(&mut facts, instance)?;
    let Some(Kind::Closure {
        identity,
        signature,
        environment,
        arguments,
    }) = facts.types.kind(instance.ty())
    else {
        return Err("actual closure".into());
    };
    let Some(Kind::Function {
        parameters,
        result,
        unsafe_,
        convention,
        ..
    }) = facts.types.kind(signature)
    else {
        return Err("actual call signature".into());
    };
    let signature = facts.types.intern(Kind::Function {
        parameters,
        result,
        unsafe_,
        convention,
        bound_regions: 1,
    })?;
    let wrong_binder = facts.types.intern(Kind::Closure {
        identity,
        signature,
        environment,
        arguments,
    })?;
    let wrong_binder = Instance::closure(&facts.types, wrong_binder)?;
    let error = super::super::inputs::select(&mut facts, wrong_binder)
        .err()
        .ok_or("changed call binder accepted")?;
    assert!(error.contains("changed its source call binder"), "{error}");
    let Some(Kind::Closure { signature, .. }) = facts.types.kind(instance.ty()) else {
        return Err("original call signature".into());
    };
    let mut values = facts
        .types
        .arguments(arguments)
        .ok_or("actual arguments")?
        .collect::<Vec<_>>();
    values[1] = Argument::Lifetime(RegionId::bound(0, 12)?);
    let arguments = facts.types.intern_arguments(&values)?;
    let wrong_capture = facts.types.intern(Kind::Closure {
        identity,
        signature,
        environment,
        arguments,
    })?;
    let wrong_capture = Instance::closure(&facts.types, wrong_capture)?;
    let error = super::super::inputs::select(&mut facts, wrong_capture)
        .err()
        .ok_or("changed inherited region accepted")?;
    assert!(
        error.contains("differs from its inherited arguments"),
        "{error}"
    );
    Ok(())
}

fn check_port_instance(
    facts: &mut crate::sema::model::Declarations,
    value: &body::instantiate::Materialized,
) -> Result<(), Box<dyn std::error::Error>> {
    value
        .contract
        .prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        &value.contract,
        &value.body,
    )?;
    Ok(())
}

#[test]
fn private_input_connections_never_erase_the_specialized_bodys_lifetime_debt()
-> Result<(), Box<dyn std::error::Error>> {
    let PortFixture {
        mut facts,
        templates,
        original,
        instance,
    } = port_fixture(false)?;
    let template = templates
        .bodies
        .iter()
        .find(|body| body.body.key() == original.key)
        .ok_or("closure template")?;
    for unpaid in [false, true] {
        let mut contract = original.clone();
        if unpaid {
            contract.obligations.push(Obligation {
                source: contract.source.into(),
                kind: body::regions::Kind::Outlives((RegionId::bound(0, 2)?, RegionId::STATIC)),
            });
        }
        let value = body::instantiate::with_contract(&mut facts, template, instance, &contract)?;
        let result = check_port_instance(&mut facts, &value);
        if unpaid {
            let error = result
                .err()
                .ok_or("private input debt was assumed or discarded")?;
            assert!(
                error
                    .to_string()
                    .contains("required lifetime does not outlive its target"),
                "{error}"
            );
        } else {
            result?;
        }
    }
    Ok(())
}

#[test]
fn source_closure_copy_scopes_reject_changed_instances_and_body_signatures()
-> Result<(), Box<dyn std::error::Error>> {
    let PortFixture {
        mut facts,
        templates,
        original,
        instance,
    } = port_fixture(true)?;
    let template = templates
        .bodies
        .iter()
        .find(|body| body.body.key() == original.key)
        .ok_or("source closure template")?;
    let value = body::instantiate::with_contract(&mut facts, template, instance, &original)?;
    assert!(value.has_instance_copy_scope_for_test());
    assert!(matches!(
        value.body.owner,
        body::ir::Owner::Closure {
            execution: Some(_),
            ..
        }
    ));
    for change in 0..4 {
        let mut cfg = value.body.clone();
        let mut input = value.contract.clone();
        match change {
            1 => cfg.instance = None,
            2 => {
                let body::ir::Owner::Closure { execution, .. } = &mut cfg.owner else {
                    return Err("source closure owner changed".into());
                };
                *execution = None;
            }
            3 => {
                let pending = input.builtins.as_mut().ok_or("source closure builtins")?;
                std::sync::Arc::make_mut(pending).environment = Some(std::sync::Arc::new(
                    crate::types::parameters::environment::Environment::new(&facts.types, &[])?,
                ));
            }
            _ => {}
        }
        let checked = value.prepare_source_copy(&mut facts, &cfg, &mut input);
        if change == 0 {
            checked?;
        } else {
            assert!(
                checked.is_err(),
                "changed instance scope {change} was accepted"
            );
        }
    }
    Ok(())
}
