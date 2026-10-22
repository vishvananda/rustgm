//! Library iteration originals require exact declarations and ordinary body proofs.
use super::*;
use crate::{body, borrow, sema, types};
use std::fmt::Write;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const SOURCE: &str = "struct I;impl core::iter::Iterator for I{type Item=u8;fn next(&mut self)->Option<u8>{None}}fn main(){for _ in I{}}";
fn fixture(source: &str) -> Result<(crate::resolve::expansion::Report, Declarations)> {
    let report = crate::test_source::resolve(source, true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
fn verify(facts: &mut Declarations, templates: &Templates, contracts: Vec<Contract>) -> Result {
    let contracts = super::super::regions::Contracts {
        bodies: contracts,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut facts.copies,
    };
    let types = &facts.types;
    let members = &facts.members;
    let checked = borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        templates,
        &[],
        |ty, field| {
            let (Some(types::Kind::Adt(owner, _)), body::ir::Selection::Nominal(field)) =
                (types.kind(ty), field)
            else {
                return Err("invalid iterator field selection");
            };
            let identity = members.identity(field).ok_or("missing iterator field")?;
            if owner.0 != identity.owner.0 {
                return Err("iterator field belongs to another owner");
            }
            Ok((identity.variant, identity.field))
        },
    )?;
    for template in &templates.bodies {
        let body::ir::Owner::Function { definition, .. } = template.body.owner else {
            return Err("iterator original owner".into());
        };
        assert!(checked.get(types::ItemId(definition.0)).is_some());
    }
    Ok(())
}
#[test]
fn iterator_originals_enter_both_contract_collectors_and_ordinary_proof() -> Result {
    let (report, mut facts) = fixture(SOURCE)?;
    for incremental in [false, true] {
        let mut contracts = if incremental {
            body::regions::Collector::default().finish(&facts)?.bodies
        } else {
            body::regions::collect(&facts)?.bodies
        };
        let mut templates = Templates { bodies: Vec::new() };
        append(&report, &mut facts, &mut templates, &mut contracts)?;
        assert_eq!(templates.bodies.len(), 2);
        for template in &templates.bodies {
            assert!(template.body.instance.is_none());
            assert!(matches!(
                template.body.key(),
                body::ir::BodyKey::Declaration(_)
            ));
        }
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        verify(&mut facts, &templates, contracts.clone())?;
        templates.bodies[0].body.locals[0].kind = body::ir::LocalKind::Binding;
        assert!(verify(&mut facts, &templates, contracts).is_err());
    }
    Ok(())
}
#[test]
fn collection_originals_enter_both_collectors_and_ordinary_ownership_proof() -> Result {
    let (report, mut facts) = fixture("fn main(){let a=[1u8,2];for _ in &a{}}")?;
    for incremental in [false, true] {
        let mut contracts = if incremental {
            body::regions::Collector::default().finish(&facts)?.bodies
        } else {
            body::regions::collect(&facts)?.bodies
        };
        let mut templates = Templates { bodies: Vec::new() };
        append(&report, &mut facts, &mut templates, &mut contracts)?;
        assert_eq!(templates.bodies.len(), 6);
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        verify(&mut facts, &templates, contracts.clone())?;
        let next = facts
            .iteration
            .as_deref()
            .and_then(|r| r.collections.as_deref())
            .ok_or("collections")?
            .originals[crate::support::iteration::Collection::SliceIterator as usize]
            .ok_or("next")?
            .declaration
            .method;
        let template=templates.bodies.iter_mut().find(|t|matches!(t.body.owner,body::ir::Owner::Function {definition,..} if definition==next)).ok_or("next original")?;
        template.body.locals[0].kind = body::ir::LocalKind::Binding;
        assert!(verify(&mut facts, &templates, contracts).is_err());
    }
    Ok(())
}
#[test]
fn iterator_factory_rejects_forged_owner_parameter_prerequisite_and_binder() -> Result {
    let (_, mut facts) = fixture(SOURCE)?;
    let originals = facts.iteration.as_deref().ok_or("registry")?.originals;
    for original in originals.into_iter().flatten() {
        original.validate(&facts)?;
        let mut forged = original;
        forged.declaration.definition = original.declaration.method;
        assert!(forged.validate(&facts).is_err());
        forged = original;
        forged.parameter = types::TypeId::UNIT;
        assert!(forged.validate(&facts).is_err());
        forged = original;
        forged.predicate.ty = types::TypeId::UNIT;
        assert!(forged.validate(&facts).is_err());
        forged = original;
        let types::Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            bound_regions,
        } = facts.types.kind(original.signature).ok_or("signature")?
        else {
            return Err("function".into());
        };
        forged.signature = facts.types.intern(types::Kind::Function {
            parameters,
            result,
            unsafe_,
            convention,
            bound_regions: bound_regions + 1,
        })?;
        assert!(forged.validate(&facts).is_err());
        let clauses = facts
            .generics
            .predicates
            .remove(&original.declaration.definition)
            .ok_or("declared prerequisite")?;
        assert!(original.validate(&facts).is_err());
        facts
            .generics
            .predicates
            .insert(original.declaration.definition, clauses);
        original.validate(&facts)?;
    }
    Ok(())
}
#[test]
fn iteration_is_sparse_and_preserves_two_distinct_call_contexts() -> Result {
    let (report, mut facts) = fixture("fn main(){let _=1u8+2u8;}")?;
    assert!(report.resolution.builtin_traits.iteration().is_none());
    assert!(facts.iteration.is_none());
    let mut templates = Templates { bodies: Vec::new() };
    let mut contracts = Vec::new();
    append(&report, &mut facts, &mut templates, &mut contracts)?;
    assert!(templates.bodies.is_empty() && contracts.is_empty());
    let (mut report, mut facts) = fixture(SOURCE)?;
    let registry = facts.iteration.as_deref().ok_or("registry")?;
    assert_eq!(registry.plans.len(), 1);
    let plan = *registry.plans.values().next().ok_or("plan")?;
    assert_ne!(plan.into_iter.callable, plan.next.callable);
    assert_ne!(plan.into_iter.signature, plan.next.signature);
    let key = *registry.plans.keys().next().ok_or("source")?;
    facts
        .iteration
        .as_deref_mut()
        .ok_or("registry")?
        .plans
        .get_mut(&key)
        .ok_or("plan")?
        .into_iter = plan.next;
    let entry = sema::entry::select(&mut report, &facts)?;
    assert!(crate::lower::instances::build(&report, &mut facts, entry).is_err());
    Ok(())
}
#[test]
fn iterator_plans_scale_with_loops_and_ignore_unrelated_source() -> Result {
    let mut observations = Vec::new();
    for (loops, unused) in [(8, 0), (32, 0), (128, 0), (1, 8), (1, 32), (1, 128)] {
        let mut source = SOURCE
            .strip_suffix("fn main(){for _ in I{}}")
            .ok_or("source")?
            .to_owned();
        for index in 0..unused {
            write!(source, "fn unused_{index}(){{let _=1u8;}}")?;
        }
        source.push_str("fn main(){");
        for _ in 0..loops {
            source.push_str("for _ in I{}");
        }
        source.push('}');
        let (_, facts) = fixture(&source)?;
        let registry = facts.iteration.as_deref().ok_or("registry")?;
        assert_eq!(registry.plans.len(), loops);
        assert_eq!(registry.originals.iter().flatten().count(), 2);
        assert_eq!(facts.body_work.trait_selections, 2 * loops);
        eprintln!(
            "iteration loops={loops} unused={unused} inference={} selection={} steps={}",
            facts.body_work.inference_steps,
            facts.body_work.trait_selections,
            facts.body_work.steps
        );
        observations.push(facts.body_work.inference_steps);
    }
    for values in observations.as_chunks::<3>().0 {
        assert_eq!(4 * (values[1] - values[0]), values[2] - values[1]);
    }

    Ok(())
}
#[test]
fn iteration_visibility_reuses_indexes_across_calls_and_unrelated_source() -> Result {
    let mut steps = Vec::new();
    for (calls, unused) in [(8, 0), (32, 0), (128, 0), (1, 8), (1, 32), (1, 128)] {
        let mut source = SOURCE
            .strip_suffix("fn main(){for _ in I{}}")
            .ok_or("source")?
            .to_owned();
        for index in 0..unused {
            write!(source, "fn unused_{index}(){{let _=1u8;}}")?;
        }
        source.push_str("fn main(){let mut i=I;");
        for _ in 0..calls {
            source.push_str("let _=i.next();");
        }
        source.push('}');
        let (_, facts) = fixture(&source)?;
        assert_eq!(facts.methods.calls.len(), calls);
        let work = facts.methods.trait_visibility_steps;
        eprintln!("iteration visibility calls={calls} unused={unused} work={work}");
        steps.push(work);
    }
    for values in steps.as_chunks::<3>().0 {
        assert_eq!(4 * (values[1] - values[0]), values[2] - values[1]);
    }
    let (_, facts) = fixture("fn main(){let _=1u8+2u8;}")?;
    assert!(facts.iteration.is_none());
    assert_eq!(facts.methods.trait_visibility_steps, 0);
    Ok(())
}
#[test]
fn iterator_layout_keeps_ordinary_records_compact() {
    eprintln!(
        "iteration sizes declarations={} registry={} plan={} original={} call={} value={}",
        size_of::<Declarations>(),
        size_of::<crate::sema::iteration::Registry>(),
        size_of::<crate::sema::iteration::ForPlan>(),
        size_of::<Original>(),
        size_of::<crate::sema::iteration::Call>(),
        size_of::<body::ir::Value>()
    );
    // The integrated 8544-byte base retains pointer methods; Clone adds one lazy pointer.
    assert_eq!(size_of::<Declarations>(), 8552);
    assert_eq!(size_of::<Original>(), 56);
    // The demand-only collection catalog adds one optional Box to this owner.
    assert_eq!(size_of::<crate::sema::iteration::Registry>(), 312);
    assert_eq!(size_of::<body::ir::Value>(), 24);
    assert_eq!(size_of::<crate::sema::iteration::Call>(), 8);
    assert_eq!(size_of::<crate::sema::iteration::ForPlan>(), 36);
}

#[path = "clone_contracts.rs"]
mod cloning;

#[path = "mutable_iterator_contracts.rs"]
mod mutable_collections;
