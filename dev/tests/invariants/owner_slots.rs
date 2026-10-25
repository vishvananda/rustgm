//! Impl regions seal before registry publication and remain exact independent slots.
use super::*;
use crate::{
    sema::model::Declarations,
    types::{Kind as Type, arguments::Argument, parameters::predicates::Bound},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const _: [(); 32] = [(); size_of::<Owner>()];
const _: [(); 20] = [(); size_of::<OwnerKind>()];
const _: [(); 32] = [(); size_of::<SourceOwner>()];
const _: [(); 96] = [(); size_of::<crate::sema::traits::implementations::Implementation>()];

#[test]
fn source_owner_requires_real_provenance_and_absence_is_not_import_authority() -> Result {
    use crate::sema::traits::implementations::{Implementation, Implementations};
    let mut facts = fixture("trait K{}struct S;impl K for S{}")?;
    let actual = *facts
        .associated
        .owners()
        .iter()
        .find(|owner| matches!(owner.kind, OwnerKind::Impl { .. }))
        .ok_or("actual impl")?;
    let id = facts
        .associated
        .owner(actual.self_definition)
        .ok_or("impl id")?;
    let source = facts.associated.source_owner(id)?;
    assert_eq!(Some(source.source), actual.source);
    assert_eq!(Owner::from(source), actual);
    let header = facts
        .generics
        .implementations
        .as_ref()
        .and_then(|r| r.header(actual.self_definition))
        .ok_or("actual header")?;
    let value = Implementation {
        owner: header.owner,
        source: None,
        predicate: header.predicate,
        methods: Vec::new(),
        types: Vec::new(),
        constants: Vec::new(),
    };
    let (application, _) = value.predicate.bound.trait_parts().ok_or("trait")?;
    let item = facts
        .types
        .trait_application(application)
        .ok_or("application")?
        .item;
    let missing = Owner {
        source: None,
        ..actual
    };
    facts.associated.owners[id.0 as usize] = missing;
    assert!(facts.associated.source_owner(id).is_err());
    assert!(facts.associated.source_at(id.0 as usize).is_err());
    let mut registry = Implementations::default();
    assert!(
        registry
            .prepare_source(
                &mut facts.types,
                &facts.associated,
                crate::sema::traits::implementations::Identity::User(item),
                missing,
                value
            )
            .is_err()
    );
    assert!(registry.headers().next().is_none());
    assert!(!registry.is_validated(actual.self_definition));
    facts.associated.owners[id.0 as usize] = actual;
    assert!(facts.associated.source_at(id.0 as usize)?.is_some());
    println!(
        "source-owner layouts owner={} kind={} source={} impl={} inventory={} cold_language={}",
        size_of::<Owner>(),
        size_of::<OwnerKind>(),
        size_of::<SourceOwner>(),
        size_of::<crate::sema::traits::implementations::Implementation>(),
        size_of::<Inventory>(),
        size_of::<language::Declarations>()
    );
    Ok(())
}

#[test]
fn projected_header_readiness_keeps_empty_completed_and_pending_family_inputs_distinct() -> Result {
    let mut facts = fixture("trait Family{type Item<M>;}")?;
    let member = *facts
        .associated
        .families
        .keys()
        .next()
        .ok_or("actual associated family")?;
    facts.associated.require_projection_header(member)?;
    let requirements = facts
        .associated
        .families
        .get_mut(&member)
        .ok_or("family")?
        .requirements
        .take()
        .ok_or("completed inputs")?;
    assert!(facts.associated.require_projection_header(member).is_err());
    facts
        .associated
        .families
        .get_mut(&member)
        .ok_or("family")?
        .requirements = Some(Vec::new());
    facts.associated.require_projection_header(member)?;
    facts
        .associated
        .families
        .get_mut(&member)
        .ok_or("family")?
        .requirements = Some(requirements);
    let contract = facts
        .associated
        .types
        .remove(&member)
        .ok_or("type contract")?;
    assert!(facts.associated.require_projection_header(member).is_err());
    facts.associated.types.insert(member, contract);
    let owner = facts.associated.owner(member).ok_or("member owner")?;
    let schema = facts
        .associated
        .schemas
        .remove(&owner)
        .ok_or("actual trait schema")?;
    assert!(facts.associated.require_projection_header(member).is_err());
    facts.associated.schemas.insert(owner, schema);
    facts.associated.require_projection_header(member)?;
    Ok(())
}
fn fixture(source: &str) -> Result<Declarations> {
    let report = crate::test_source::resolve(source, true)?;
    Ok(crate::pipeline::analyze_source(&report)?.facts)
}
fn owner(facts: &Declarations) -> Result<Owner> {
    Ok(*facts
        .associated
        .owners()
        .iter()
        .find(|o| {
            matches!(
                o.kind,
                OwnerKind::Impl {
                    trait_type: Some(_),
                    ..
                }
            )
        })
        .ok_or("missing impl owner")?)
}
#[test]
fn self_and_trait_reference_inputs_retain_independent_slots() -> Result {
    for (source, captures, expected) in [
        (
            "trait K<R>{fn yes(&self,r:R)->bool;}struct S;impl K<&S>for&S{fn yes(&self,_:&S)->bool{true}}fn main(){}",
            2,
            vec![0, 1],
        ),
        (
            "trait K<A,B>{}struct S;impl K<&S,&S>for&S{}fn main(){}",
            3,
            vec![0, 1, 2],
        ),
        (
            "trait K<R>{}struct S;impl<'unused>K<&S>for&S{}fn main(){}",
            3,
            vec![1, 2],
        ),
    ] {
        let facts = fixture(source)?;
        let owner = owner(&facts)?;
        let schema = facts
            .associated
            .schema(owner.self_definition)
            .ok_or("missing schema")?;
        assert_eq!(schema.captures, captures);
        let header = facts
            .generics
            .implementations
            .as_ref()
            .and_then(|r| r.header(owner.self_definition))
            .ok_or("missing header")?;
        let Bound::Trait(id) = header.predicate.bound else {
            return Err("missing trait application".into());
        };
        let app = facts
            .types
            .trait_application(id)
            .ok_or("missing application")?;
        let mut inputs = vec![schema.self_type];
        inputs.extend(
            facts
                .types
                .arguments(app.arguments)
                .ok_or("missing arguments")?
                .filter_map(|a| {
                    if let Argument::Type(t) = a {
                        Some(t)
                    } else {
                        None
                    }
                }),
        );
        let slots = inputs
            .into_iter()
            .map(|ty| {
                let Some(Type::Reference { region, .. }) = facts.types.kind(ty) else {
                    return Err("missing reference input");
                };
                let Some((0, index)) = region.bound_parts() else {
                    return Err("input outside impl binder");
                };
                Ok(index)
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        assert_eq!(slots, expected);
        for f in &facts.functions {
            if facts.associated.owner(f.definition) == facts.associated.owner(owner.self_definition)
            {
                assert_eq!(
                    facts
                        .lifetimes
                        .functions
                        .get(&f.definition)
                        .ok_or("missing function regions")?
                        .captures,
                    captures
                );
            }
        }
    }
    Ok(())
}
#[test]
fn region_sealing_rejects_published_stale_and_wrong_owners() -> Result {
    let mut facts = fixture("trait K<R>{}struct S;impl<'a,'b>K<&'b S>for&'a S{}fn main(){}")?;
    let owner = owner(&facts)?;
    let definition = owner.self_definition;
    assert!(facts.finish_impl_regions(definition, 2).is_err());
    assert!(
        facts
            .finish_impl_regions(crate::resolve::model::DefId(u32::MAX), 2)
            .is_err()
    );
    let trait_owner = facts
        .associated
        .owners()
        .iter()
        .find(|o| matches!(o.kind, OwnerKind::Trait { .. }))
        .ok_or("missing trait owner")?
        .self_definition;
    assert!(facts.finish_impl_regions(trait_owner, 2).is_err());
    // Model the pre-publication metadata seam without forging selection evidence.
    let registry = facts.generics.implementations.take();
    assert!(facts.finish_impl_regions(definition, 1).is_err());
    let ty = facts.definitions[definition.0 as usize];
    facts.definitions[definition.0 as usize] = Some(TypeId::UNIT);
    assert!(facts.finish_impl_regions(definition, 2).is_err());
    facts.definitions[definition.0 as usize] = ty;
    facts.finish_impl_regions(definition, 2)?;
    assert_eq!(
        facts
            .associated
            .schema(definition)
            .ok_or("missing restored schema")?
            .captures,
        2
    );
    facts.generics.implementations = registry;
    assert!(facts.finish_impl_regions(definition, 3).is_err());
    assert_eq!(
        facts
            .associated
            .schema(definition)
            .ok_or("missing schema")?
            .captures,
        2
    );
    Ok(())
}
#[test]
fn nested_callable_binders_do_not_become_impl_slots() -> Result {
    let facts = fixture("trait K<R>{}struct S;impl K<fn(&S)->&S>for&S{}fn main(){}")?;
    let owner = owner(&facts)?;
    assert_eq!(
        facts
            .associated
            .schema(owner.self_definition)
            .ok_or("missing schema")?
            .captures,
        1
    );
    let header = facts
        .generics
        .implementations
        .as_ref()
        .and_then(|r| r.header(owner.self_definition))
        .ok_or("missing header")?;
    let Bound::Trait(id) = header.predicate.bound else {
        return Err("missing application".into());
    };
    let app = facts
        .types
        .trait_application(id)
        .ok_or("missing application")?;
    assert!(facts.types.arguments(app.arguments).ok_or("missing arguments")?.any(|a|matches!(a,Argument::Type(t) if matches!(facts.types.kind(t),Some(Type::Function{bound_regions:1,..})))));
    Ok(())
}

#[test]
fn prepared_unvalidated_headers_already_seal_region_metadata() -> Result {
    use crate::sema::traits::implementations::{Implementation, Implementations};
    let mut facts = fixture("trait K<R>{}struct S;impl<'a,'b>K<&'b S>for&'a S{}fn main(){}")?;
    let definition = owner(&facts)?.self_definition;
    let original = facts
        .generics
        .implementations
        .take()
        .ok_or("missing registry")?;
    let header = original.header(definition).ok_or("missing header")?;
    let Bound::Trait(application) = header.predicate.bound else {
        return Err("missing trait application".into());
    };
    let item = facts
        .types
        .trait_application(application)
        .ok_or("missing application")?
        .item;
    let value = Implementation {
        owner: definition,
        source: header.source,
        predicate: header.predicate,
        methods: Vec::new(),
        types: Vec::new(),
        constants: Vec::new(),
    };
    let mut prepared = Implementations::default();
    prepared.prepare(&mut facts.types, item, value)?;
    assert!(!prepared.is_validated(definition));
    facts.generics.implementations = Some(Box::new(prepared));
    assert!(facts.finish_impl_regions(definition, 2).is_err());
    assert!(facts.finish_impl_regions(definition, 3).is_err());
    assert_eq!(
        facts
            .associated
            .schema(definition)
            .ok_or("missing schema")?
            .captures,
        2
    );
    Ok(())
}
