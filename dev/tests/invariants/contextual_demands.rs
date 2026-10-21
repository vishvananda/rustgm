//! Reusable target clauses retain complete binders without caller proof authority.
use super::*;
use crate::sema::{bodies, declarations};
use std::fmt::Write;

fn facts(source: &str) -> Result<Declarations, Box<dyn std::error::Error>> {
    let report = crate::test_source::resolve(source, true)?;
    Ok(declarations::analyze(&report)?)
}

#[test]
fn complete_targets_share_payload_and_preserve_unused_binders()
-> Result<(), Box<dyn std::error::Error>> {
    let mut facts = facts("trait Apply:Fn(u8)->u8{}type A=dyn Apply;type B=dyn for<'a>Apply;")?;
    let targets: Vec<_> = (0..u32::try_from(facts.types.len())?)
        .map(TypeId)
        .filter(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
        .collect();
    let plain = *targets
        .iter()
        .find(|&&ty| {
            matches!(
                facts.types.kind(ty),
                Some(Kind::Dynamic {
                    bound_regions: 0,
                    ..
                })
            )
        })
        .ok_or("missing empty object binder")?;
    let quantified = *targets
        .iter()
        .find(|&&ty| {
            matches!(
                facts.types.kind(ty),
                Some(Kind::Dynamic {
                    bound_regions: 1,
                    ..
                })
            )
        })
        .ok_or("missing unused object binder")?;
    let (a, _) = get(&mut facts, plain)?;
    let (b, _) = get(&mut facts, quantified)?;
    let a = a.ok_or("missing plain template")?;
    let b = b.ok_or("missing quantified template")?;
    let Some(Kind::Dynamic {
        shape,
        arguments,
        bound_regions,
        ..
    }) = facts.types.kind(plain)
    else {
        return Err("missing object shape".into());
    };
    let borrowed = facts.types.intern(Kind::Dynamic {
        shape,
        arguments,
        bound_regions,
        region: RegionId::bound(0, 0)?,
    })?;
    let (shared, _) = get(&mut facts, borrowed)?;
    assert!(Arc::ptr_eq(
        &a.clauses,
        &shared.ok_or("missing borrowed template")?.clauses
    ));
    assert!(!Arc::ptr_eq(&a.clauses, &b.clauses));
    for clause in b.clauses.iter() {
        let Clause::ForAll(id) = clause else {
            return Err("lost object quantifier".into());
        };
        assert!(
            facts
                .types
                .quantified(*id)
                .ok_or("missing clause")?
                .bound_regions
                >= 1
        );
    }
    let count = facts.types.len();
    for _ in 0..128 {
        let (reused, steps) = get(&mut facts, plain)?;
        assert!(Arc::ptr_eq(
            &a.clauses,
            &reused.ok_or("missing reused template")?.clauses
        ));
        assert_eq!(steps, 1);
    }
    assert_eq!(facts.types.len(), count);
    assert_eq!(
        facts
            .generics
            .traits
            .as_ref()
            .and_then(|c| c.object_bounds.demands.as_ref())
            .ok_or("missing cache")?
            .entries
            .len(),
        2
    );
    Ok(())
}

#[test]
fn failed_and_unfinished_contracts_never_publish_cache_success()
-> Result<(), Box<dyn std::error::Error>> {
    let mut facts = facts("trait Apply:Fn(u8)->u8{}type A=dyn Apply;")?;
    assert!(get(&mut facts, TypeId(u32::MAX)).is_err());
    assert!(
        facts
            .generics
            .traits
            .as_ref()
            .is_some_and(|c| c.object_bounds.demands.is_none())
    );
    let target = (0..u32::try_from(facts.types.len())?)
        .map(TypeId)
        .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
        .ok_or("missing object")?;
    facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing contracts")?
        .object_bounds
        .enabled = false;
    assert!(get(&mut facts, target)?.0.is_some());
    assert!(
        facts
            .generics
            .traits
            .as_ref()
            .is_some_and(|c| c.object_bounds.demands.is_none())
    );
    let bounds = &mut facts
        .generics
        .traits
        .as_mut()
        .ok_or("missing contracts")?
        .object_bounds;
    bounds.enable();
    get(&mut facts, target)?;
    let cache = facts
        .generics
        .traits
        .as_mut()
        .and_then(|c| c.object_bounds.demands.as_mut())
        .ok_or("missing cache")?;
    cache.steps = 1_000_000;
    assert!(get(&mut facts, target).is_err());
    Ok(())
}

#[test]
fn ordinary_and_nonclosure_conversions_allocate_no_demand_cache()
-> Result<(), Box<dyn std::error::Error>> {
    for source in [
        "trait Apply:Fn(u8)->u8{}fn main(){let x=7u8;let _=x;}",
        "trait Apply:Fn(u8)->u8{}impl<F:Fn(u8)->u8>Apply for F{}fn id(x:u8)->u8{x}fn main(){let f=id as fn(u8)->u8;let _: &dyn Apply=&f;}",
    ] {
        let report = crate::test_source::resolve(source, true)?;
        let facts = bodies::analyze(&report, declarations::analyze(&report)?)?;
        assert!(
            facts
                .generics
                .traits
                .as_ref()
                .is_some_and(|c| c.object_bounds.demands.is_none())
        );
    }
    Ok(())
}

#[test]
fn repeated_and_distinct_target_graphs_charge_linear_work() -> Result<(), Box<dyn std::error::Error>>
{
    for distinct in [false, true] {
        for width in [8, 32, 128] {
            let mut source = "trait Apply<T>:Fn(T)->T{}".to_owned();
            for index in 0..width {
                let count = if distinct { index + 1 } else { 1 };
                write!(source, "type A{index}=dyn Apply<[u8;{count}]>;")?;
            }
            let mut facts = facts(&source)?;
            let targets: Vec<_> = (0..u32::try_from(facts.types.len())?)
                .map(TypeId)
                .filter(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
                .collect();
            let mut work = 0;
            for ty in targets.iter().cycle().take(width) {
                work += get(&mut facts, *ty)?.1;
            }
            let cache = facts
                .generics
                .traits
                .as_ref()
                .and_then(|c| c.object_bounds.demands.as_ref())
                .ok_or("missing demand cache")?;
            assert_eq!(cache.entries.len(), if distinct { width } else { 1 });
            assert!(
                work <= 400 * width,
                "distinct={distinct} width={width} work={work}"
            );
            eprintln!(
                "object input distinct={distinct} width={width} steps={work} entries={} payload={}",
                cache.entries.len(),
                cache.payload
            );
        }
    }
    Ok(())
}

#[test]
fn declared_templates_reuse_no_pending_caller_terms_and_keep_exact_binders()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::infer::{Context, Domain, arguments::Argument};
    let mut facts = facts("trait Apply<'a,T>:Fn(T)->&'a u8{}type A=dyn Apply<'static,u8>;")?;
    let target = (0..u32::try_from(facts.types.len())?)
        .map(TypeId)
        .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
        .ok_or("missing object")?;
    let mut context = Context::default();
    let root = context.known(&facts.types, target)?;
    let mut principal = context
        .object_principal(&facts.types, root)?
        .ok_or("missing principal")?;
    let first = context.variable(Domain::Any)?;
    principal.arguments[1] = Argument::Type(first);
    let a = pending::get(&mut facts, &principal)?;
    let other = context.variable(Domain::Any)?;
    principal.arguments[1] = Argument::Type(other);
    let b = pending::get(&mut facts, &principal)?;
    assert!(Arc::ptr_eq(&a.template.clauses, &b.template.clauses));
    assert_eq!(a.bindings[0].1, Argument::Type(first));
    assert_eq!(b.bindings[0].1, Argument::Type(other));
    assert_eq!(b.steps, 3);
    principal.bound_regions = 1;
    let c = pending::get(&mut facts, &principal)?;
    assert!(!Arc::ptr_eq(&a.template.clauses, &c.template.clauses));
    principal.arguments[0] = Argument::Lifetime(RegionId::bound(0, 0)?);
    let d = pending::get(&mut facts, &principal)?;
    assert!(!Arc::ptr_eq(&c.template.clauses, &d.template.clauses));
    for clause in d.template.clauses.iter() {
        let Clause::ForAll(id) = clause else {
            return Err("lost quantifier".into());
        };
        assert!(
            facts
                .types
                .quantified(*id)
                .ok_or("missing clause")?
                .bound_regions
                >= 1
        );
    }
    assert_eq!(
        facts
            .generics
            .traits
            .as_ref()
            .and_then(|c| c.object_bounds.demands.as_ref())
            .ok_or("missing cache")?
            .entries
            .len(),
        3
    );
    Ok(())
}

#[test]
fn malformed_pending_principals_do_not_publish_completed_templates()
-> Result<(), Box<dyn std::error::Error>> {
    use crate::types::infer::{Context, arguments::Argument};
    let mut facts = facts("trait Apply<T>:Fn(T)->T{}type A=dyn Apply<u8>;")?;
    let target = (0..u32::try_from(facts.types.len())?)
        .map(TypeId)
        .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
        .ok_or("missing object")?;
    let mut context = Context::default();
    let root = context.known(&facts.types, target)?;
    let mut principal = context
        .object_principal(&facts.types, root)?
        .ok_or("missing principal")?;
    principal.arguments.clear();
    assert!(pending::get(&mut facts, &principal).is_err());
    principal
        .arguments
        .push(Argument::Lifetime(RegionId::STATIC));
    assert!(pending::get(&mut facts, &principal).is_err());
    assert!(
        facts
            .generics
            .traits
            .as_ref()
            .is_some_and(|c| c.object_bounds.demands.is_none())
    );
    Ok(())
}
