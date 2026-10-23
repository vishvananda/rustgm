//! Source-free provided methods still require exact original body and contract proofs.
use crate::{body, borrow, sema, types};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn verify(
    facts: &mut sema::model::Declarations,
    templates: &body::templates::Templates,
    contracts: Vec<body::regions::Contract>,
) -> Result {
    let contracts = body::regions::Contracts {
        bodies: contracts,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut facts.copies,
    };
    let members = &facts.members;
    let types = &facts.types;
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
                return Err("invalid provided field selection");
            };
            let identity = members.identity(field).ok_or("missing provided field")?;
            if owner.0 != identity.owner.0 {
                return Err("provided field belongs to different owner");
            }
            Ok((identity.variant, identity.field))
        },
    )?;
    for template in &templates.bodies {
        let body = &template.body;
        let body::ir::Owner::Function { definition, .. } = body.owner else {
            return Err("default owner".into());
        };
        assert!(checked.get(types::ItemId(definition.0)).is_some());
        assert_eq!(
            checked
                .input(types::ItemId(definition.0))
                .ok_or("default contract")?
                .universals,
            if facts
                .associated
                .comparison_method(definition)
                .is_some_and(crate::support::comparisons::Method::owned)
            {
                0
            } else {
                2
            }
        );
    }
    Ok(())
}
#[test]
fn default_ne_original_is_checked_with_incremental_and_batch_contracts() -> Result {
    for source in [
        "struct S;impl PartialEq for S{fn eq(&self,_:&Self)->bool{false}}fn main(){let _=S!=S;}",
        "fn main(){let _=[1u8]!=[2u8];}",
    ] {
        let report = crate::test_source::resolve(source, true)?;
        let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
        for incremental in [false, true] {
            let mut contracts = if incremental {
                body::regions::Collector::default().finish(&facts)?.bodies
            } else {
                body::regions::collect(&facts)?.bodies
            };
            let mut templates = body::templates::Templates { bodies: Vec::new() };
            super::append(&report, &mut facts, &mut templates, &mut contracts)?;
            assert_eq!(templates.bodies.len(), 1);
            let default = &templates.bodies[0].body;
            assert!(default.instance.is_none());
            assert!(matches!(default.key(), body::ir::BodyKey::Declaration(_)));
            assert_eq!(default.locals.len(), 2);
            verify(&mut facts, &templates, contracts.clone())?;
            templates.bodies[0].body.locals[0].kind = body::ir::LocalKind::Binding;
            assert!(verify(&mut facts, &templates, contracts).is_err());
        }
    }
    Ok(())
}

#[test]
fn comparison_physical_borrows_retain_each_owning_coercion_once() -> Result {
    let report = crate::test_source::resolve(
        "struct S<'a>(&'a u8);impl<'a,'b> PartialEq<S<'b>> for S<'a>{fn eq(&self,r:&S<'b>)->bool{*self.0==*r.0}}fn equal<'a,'b>(a:S<'a>,b:S<'b>)->bool{a==b}fn main(){}",
        true,
    )?;
    let crate::pipeline::Analysis { facts, .. } = crate::pipeline::analyze_source(&report)?;
    let (&source, call) = facts
        .methods
        .calls
        .iter()
        .find(|(source, call)| {
            facts.associated.comparison_method(call.method).is_some()
                && facts.methods.comparison_borrow(**source).is_some()
        })
        .ok_or("comparison selection")?;
    let left = call.borrow.ok_or("left physical borrow")?;
    let right = facts
        .methods
        .comparison_borrow(source)
        .ok_or("right physical borrow")?;
    assert_ne!(left, right);
    let contracts = body::regions::collect(&facts)?;
    for actual in [left, right] {
        let coercions = facts
            .coercions
            .iter()
            .filter(|coercion| coercion.source == source && coercion.actual == actual)
            .collect::<Vec<_>>();
        assert_eq!(coercions.len(), 1);
        let coercion = coercions[0];
        let obligations=contracts.bodies.iter().flat_map(|contract|contract.obligations.iter()).filter(|obligation|obligation.source==source.into() && matches!(obligation.kind,body::regions::Kind::Coercion{actual:from,expected:to,..} if from==actual && to==coercion.expected)).count();
        assert_eq!(obligations, 1);
    }
    Ok(())
}

#[test]
fn ordering_defaults_use_original_proof_with_both_collectors() -> Result {
    let report = crate::test_source::resolve(
        "struct S;impl PartialEq for S{fn eq(&self,_:&Self)->bool{false}}impl PartialOrd for S{fn partial_cmp(&self,_:&Self)->Option<core::cmp::Ordering>{None}}fn main(){let _=S<S;}",
        true,
    )?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    for incremental in [false, true] {
        let mut contracts = if incremental {
            body::regions::Collector::default().finish(&facts)?.bodies
        } else {
            body::regions::collect(&facts)?.bodies
        };
        let mut templates = body::templates::Templates { bodies: Vec::new() };
        super::append(&report, &mut facts, &mut templates, &mut contracts)?;
        // The requested canonical Ordering result also exposes its Ord contract:
        // ne, four partial relations and three owned helpers are checked originals.
        assert_eq!(templates.bodies.len(), 8);
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        verify(&mut facts, &templates, contracts.clone())?;
        let selected=templates.bodies.iter().position(|template| matches!(template.body.owner,body::ir::Owner::Function{definition,..} if facts.associated.comparison_method(definition)==Some(crate::support::comparisons::Method::Lt))).ok_or("provided lt original")?;
        assert_eq!(templates.bodies[selected].body.blocks.len(), 4);
        templates.bodies[selected].body.locals[0].kind = body::ir::LocalKind::Binding;
        assert!(verify(&mut facts, &templates, contracts).is_err());
    }
    Ok(())
}

#[test]
fn owned_helpers_require_original_contracts_with_method_only_sized() -> Result {
    use crate::support::comparisons::{Comparison, Method};
    let report = crate::test_source::resolve(
        "fn main(){let _=u8::min(1,2);let _=u8::clamp(2,1,3);}",
        true,
    )?;
    let crate::pipeline::Analysis { mut facts, .. } = crate::pipeline::analyze_source(&report)?;
    for incremental in [false, true] {
        let mut contracts = if incremental {
            body::regions::Collector::default().finish(&facts)?.bodies
        } else {
            body::regions::collect(&facts)?.bodies
        };
        let mut templates = body::templates::Templates { bodies: Vec::new() };
        super::append(&report, &mut facts, &mut templates, &mut contracts)?;
        assert_eq!(templates.bodies.len(), 8);
        let error = verify(&mut facts, &templates, contracts.clone())
            .err()
            .ok_or("missing promotion gate")?;
        assert!(
            error
                .to_string()
                .contains("requires completed promotion analysis")
        );
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        verify(&mut facts, &templates, contracts.clone())?;
        let helper=templates.bodies.iter().position(|template|matches!(template.body.owner,body::ir::Owner::Function{definition,..} if facts.associated.comparison_method(definition)==Some(Method::Min))).ok_or("provided min")?;
        let body::ir::Owner::Function {
            definition,
            signature,
            ..
        } = templates.bodies[helper].body.owner
        else {
            return Err("owned helper owner".into());
        };
        assert!(matches!(
            facts.types.kind(signature),
            Some(types::Kind::Function {
                bound_regions: 0,
                ..
            })
        ));
        assert!(
            facts.generics.predicates[&definition]
                .iter()
                .any(|p| matches!(
                    p.value,
                    types::parameters::clauses::Clause::Predicate(
                        types::parameters::predicates::Predicate {
                            bound: types::parameters::predicates::Bound::Sized,
                            ..
                        }
                    )
                ))
        );
        let owner = facts
            .associated
            .trait_owner(definition)
            .ok_or("Ord owner")?;
        assert_eq!(facts.associated.comparison(owner), Some(Comparison::Ord));
        assert!(!facts.generics.predicates[&owner].iter().any(|p| matches!(
            p.value,
            types::parameters::clauses::Clause::Predicate(
                types::parameters::predicates::Predicate {
                    bound: types::parameters::predicates::Bound::Sized,
                    ..
                }
            )
        )));
        templates.bodies[helper].body.locals[0].kind = body::ir::LocalKind::Binding;
        assert!(verify(&mut facts, &templates, contracts).is_err());
    }
    Ok(())
}
