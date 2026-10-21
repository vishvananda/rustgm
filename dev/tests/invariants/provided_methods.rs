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
        copies: &mut facts.copies,
    };
    let checked = borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        templates,
        &[],
        |_, _| Err("unexpected equality field"),
    )?;
    let body = &templates.bodies[0].body;
    let body::ir::Owner::Function { definition, .. } = body.owner else {
        return Err("default owner".into());
    };
    assert!(checked.get(types::ItemId(definition.0)).is_some());
    assert_eq!(
        checked
            .input(types::ItemId(definition.0))
            .ok_or("default contract")?
            .universals,
        2
    );
    Ok(())
}
#[test]
fn default_ne_original_is_checked_with_incremental_and_batch_contracts() -> Result {
    let report = crate::test_source::resolve(
        "struct S;impl PartialEq for S{fn eq(&self,_:&Self)->bool{false}}fn main(){let _=S!=S;}",
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
        assert_eq!(templates.bodies.len(), 1);
        let default = &templates.bodies[0].body;
        assert!(default.instance.is_none());
        assert!(matches!(default.key(), body::ir::BodyKey::Declaration(_)));
        assert_eq!(default.locals.len(), 2);
        verify(&mut facts, &templates, contracts.clone())?;
        templates.bodies[0].body.locals[0].kind = body::ir::LocalKind::Binding;
        assert!(verify(&mut facts, &templates, contracts).is_err());
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
        let obligations=contracts.bodies.iter().flat_map(|contract|contract.obligations.iter()).filter(|obligation|obligation.source==source && matches!(obligation.kind,body::regions::Kind::Coercion{actual:from,expected:to,..} if from==actual && to==coercion.expected)).count();
        assert_eq!(obligations, 1);
    }
    Ok(())
}
