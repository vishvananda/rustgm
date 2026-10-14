//! Concrete instances reuse proved template structure and select their actual cleanup needs.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body, borrow,
    sema::{model::Declarations, values::Evaluator},
    types::{Kind, instance::Instance},
};

fn instances(
    report: &rustgm::resolve::expansion::Report,
    facts: &mut Declarations,
) -> Result<Vec<Instance>, Box<dyn std::error::Error>> {
    let definitions = facts
        .initializers
        .iter()
        .map(|i| i.definition)
        .collect::<Vec<_>>();
    let mut evaluator = Evaluator::new(report, facts);
    definitions
        .into_iter()
        .map(|id| {
            evaluator
                .definition(id)?
                .as_function()
                .ok_or_else(|| "missing selected instance".into())
        })
        .collect()
}
fn prove<'a>(
    facts: &Declarations,
    templates: &'a body::templates::Templates,
) -> Result<borrow::templates::Checked<'a>, Box<dyn std::error::Error>> {
    let contracts = body::regions::collect(facts)?;
    let mut copies = rustgm::types::copy::Copies::default();
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        copies: &mut copies,
    };
    let custom = facts
        .destructors
        .implementations()
        .iter()
        .map(|i| i.item)
        .collect::<Vec<_>>();
    Ok(borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        templates,
        &custom,
        |_, _| Err("unexpected nominal field"),
    )?)
}
fn selected_template<'a>(
    facts: &Declarations,
    checked: &'a borrow::templates::Checked<'_>,
    instance: Instance,
) -> Result<&'a body::templates::Template, &'static str> {
    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty()) else {
        return Err("missing instance item");
    };
    checked.get(item).ok_or("missing checked template")
}

#[test]
fn const_instantiation_resolves_bounds_and_values_without_rebuilding_control_flow()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn f<const N:usize>(a:&[u8;N],i:usize)->usize{a[i] as usize+N}\
      const A:fn(&[u8;3],usize)->usize=f::<3>;const B:fn(&[u8;0],usize)->usize=f::<0>;";
    rustc::qualify("instance-array-operands", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let instances = instances(&report, &mut facts)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let checked = prove(&facts, &templates)?;
    for (instance, expected) in instances.into_iter().zip([3, 0]) {
        let template = selected_template(&facts, &checked, instance)?;
        let (actual, _) = body::instantiate::body(&mut facts, template, instance)?;
        assert_eq!(actual.source, template.body.source);
        assert_eq!(actual.instance, Some(instance));
        assert_eq!(actual.blocks.len(), template.body.blocks.len());
        assert_eq!(actual.statements.len(), template.body.statements.len());
        for (a, b) in actual.statements.iter().zip(&template.body.statements) {
            assert_eq!(a.source, b.source);
            assert_eq!(format!("{:?}", a.action), format!("{:?}", b.action));
        }
        assert!(actual.projections.iter().any(
            |p| matches!(p.kind, body::ir::ProjectionKind::Index {length,..} if length==expected)
        ));
        assert!(
            !actual
                .projections
                .iter()
                .any(|p| matches!(p.kind, body::ir::ProjectionKind::IndexParameter { .. }))
        );
        let constants = actual
            .values
            .iter()
            .filter_map(|v| match v.kind {
                body::ir::Rvalue::ConstArgument(id) => Some(id),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(constants.len(), 1);
        assert_eq!(
            facts
                .types
                .constant(constants[0])
                .ok_or("missing concrete operand")?
                .bits,
            u128::from(expected)
        );
    }
    Ok(())
}

#[test]
fn concrete_promotion_materializes_each_instances_values_separately()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn f<const N:usize>()->&'static[usize;N]{&[N;N]}\
      const A:fn()->&'static[usize;3]=f::<3>;const B:fn()->&'static[usize;5]=f::<5>;";
    rustc::qualify("instance-promoted-values", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let instances = instances(&report, &mut facts)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let checked = prove(&facts, &templates)?;
    for (instance, expected) in instances.into_iter().zip([3, 5]) {
        let template = selected_template(&facts, &checked, instance)?;
        assert_eq!(
            template
                .body
                .promotions
                .as_ref()
                .ok_or("missing promotion")?
                .deferred()
                .count(),
            1
        );
        let (actual, _) = body::instantiate::body(&mut facts, template, instance)?;
        assert!(actual.promotions.is_none());
        let mut program = body::ir::Program {
            bodies: vec![actual],
            drops: None,
            region_equalities: vec![],
            coercions: vec![],
            outlives: vec![],
            body_regions: vec![],
        };
        body::promote::analyze(&report, &mut facts, &mut program)?;
        let promotion = program.bodies[0]
            .promotions
            .as_ref()
            .ok_or("missing concrete promotion")?;
        assert_eq!(promotion.deferred().count(), 0);
        let entries = promotion.entries().collect::<Vec<_>>();
        assert_eq!(entries.len(), 1);
        let reader = rustgm::sema::values::Reader::new(&facts);
        assert!(
            matches!(reader.view(entries[0].1)?,rustgm::sema::values::View::Repeat {length,..} if length==expected)
        );
        assert_eq!(
            reader.field(entries[0].1, expected - 1)?.as_scalar()?,
            rustgm::sema::model::Scalar::Integer(u128::from(expected))
        );
    }
    Ok(())
}

#[test]
fn actual_cleanup_preserves_owned_scopes_and_removes_only_dropless_instances()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}fn id<T>(x:T)->T{x}\
        const A:fn(u8)->u8=id::<u8>;const B:fn(D)->D=id::<D>;";
    rustc::qualify("instance-cleanup-scopes", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let instances = instances(&report, &mut facts)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    let checked = prove(&facts, &templates)?;
    assert_eq!(checked.loans().len(), 1);
    assert_eq!(checked.regions().bodies.len(), 1);
    for (instance, destructible) in instances.into_iter().zip([false, true]) {
        let template = selected_template(&facts, &checked, instance)?;
        let original = template
            .body
            .owned
            .as_ref()
            .ok_or("template lost possible cleanup")?;
        let (actual, requirements) = body::instantiate::body(&mut facts, template, instance)?;
        assert_eq!(requirements.values.contains(&true), destructible);
        assert_eq!(actual.owned.is_some(), destructible);
        if let Some(owned) = actual.owned {
            assert_eq!(owned.created.len(), original.created.len());
            assert_eq!(owned.returned.len(), original.returned.len());
            for (a, b) in owned.created.iter().zip(&original.created) {
                assert_eq!(a.lifetime, b.lifetime);
                assert_eq!(a.value, b.value);
                assert_eq!(a.before, b.before);
            }
        }
    }
    Ok(())
}

#[test]
fn template_proof_rejects_invalid_unused_bodies_and_missing_promotion()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "fn bad<T>(x:T){let a=x;let b=x;}fn good<U>(x:U)->U{x}";
    rustc::qualify("instance-invalid-unused", source, false)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut templates = body::templates::build(&report, &facts)?;
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    assert!(prove(&facts, &templates).is_err());
    let (report, mut facts) = semantic::analyze("fn f<const N:usize>()->&'static usize{&N}")?;
    let mut templates = body::templates::build(&report, &facts)?;
    assert!(prove(&facts, &templates).is_err());
    body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
    prove(&facts, &templates)?;
    Ok(())
}
