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

mod projections {
    use super::semantic;
    use rustgm::{
        body::{
            self,
            regions::{Contract, Contracts},
            templates::Template,
        },
        borrow,
        sema::model::Declarations,
        types::{
            ItemId, Kind, Primitive, RegionId, TypeId, arguments::Argument, instance::Instance,
        },
    };
    type TestResult = Result<(), Box<dyn std::error::Error>>;
    const SOURCE: &str = "trait A{type Out;}impl<'a:'static>A for &'a u8{type Out=u16;}fn f<T:A>(p:fn(<T as A>::Out))->fn(<T as A>::Out){p}fn take(_:u16){}fn main(){let p=f::<&'static u8>(take);p(73);}";
    const QUANTIFIED: &str = "trait A<'a>{type Out;}impl<'a,'x:'a>A<'a>for &'x u8{type Out=u16;}fn f<T:for<'a>A<'a>>(p:fn(<T as A<'static>>::Out))->fn(<T as A<'static>>::Out){p}fn take(_:u16){}fn main(){let p=f::<&'static u8>(take);p(73);}";
    struct Selected {
        facts: Declarations,
        template: Template,
        contract: Contract,
        item: ItemId,
        signature: TypeId,
        instance: Instance,
    }
    fn selected() -> Result<Selected, Box<dyn std::error::Error>> {
        selected_from(SOURCE)
    }
    fn selected_from(source: &str) -> Result<Selected, Box<dyn std::error::Error>> {
        let (report, facts) = semantic::analyze(source)?;
        let mut templates = body::templates::build(&report, &facts)?;
        let template = templates.bodies.pop().ok_or("missing generic template")?;
        assert!(templates.bodies.is_empty());
        let body::ir::Owner::Function { definition, .. } = template.body.owner else {
            return Err("missing function owner".into());
        };
        let item = ItemId(definition.0);
        let contract = body::regions::collect(&facts)?
            .bodies
            .into_iter()
            .find(|c| c.source == template.body.source)
            .ok_or("missing template contract")?;
        let ordinary = body::build_concrete(&report, &facts)?;
        let ty = ordinary.bodies.iter().flat_map(|body| &body.values).map(|v| v.ty)
            .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::FunctionItem {item: selected, ..}) if selected==item))
            .ok_or("missing concrete selection")?;
        let Some(Kind::FunctionItem { signature, .. }) = facts.types.kind(ty) else {
            return Err("missing selected signature".into());
        };
        let instance = Instance::new(&facts.types, ty)?;
        Ok(Selected {
            facts,
            template,
            contract,
            item,
            signature,
            instance,
        })
    }
    fn check(facts: &Declarations, contract: Contract) -> Result<(), String> {
        borrow::regions::check(
            &facts.types,
            &facts.lifetimes.variances,
            &Contracts {
                bodies: vec![contract],
                callables: &facts.lifetimes.callables,
                nominal_requirements: &facts.lifetimes.nominal_requirements,
            },
        )
        .map(|_| ())
        .map_err(|e| e.to_string())
    }

    #[test]
    fn concrete_selection_normalizes_the_body_and_its_contract() -> TestResult {
        let mut selected = selected()?;
        let materialized = body::instantiate::with_contract(
            &mut selected.facts,
            &selected.template,
            selected.instance,
            &selected.contract,
        )?;
        let body = materialized.body;
        assert_eq!(body.blocks.len(), selected.template.body.blocks.len());
        assert_eq!(
            body.statements.len(),
            selected.template.body.statements.len()
        );
        for ty in std::iter::once(body.result)
            .chain(body.values.iter().map(|v| v.ty))
            .chain(body.locals.iter().map(|v| v.ty))
            .chain(body.projections.iter().map(|v| v.ty))
        {
            assert_eq!(selected.facts.types.contains_projections(ty), Some(false));
        }
        check(&selected.facts, materialized.contract)?;
        Ok(())
    }

    #[test]
    fn normalized_signature_comparison_rejects_a_forged_callable() -> TestResult {
        let mut selected = selected()?;
        let Some(Kind::FunctionItem { arguments, .. }) =
            selected.facts.types.kind(selected.instance.ty())
        else {
            return Err("missing selected item".into());
        };
        let parameters = selected.facts.types.intern_list(&[])?;
        let signature = selected.facts.types.intern(Kind::Function {
            bound_regions: 0,
            parameters,
            result: TypeId::UNIT,
            unsafe_: false,
            convention: rustgm::types::callable::Convention::RUST,
        })?;
        let ty = selected.facts.types.intern(Kind::FunctionItem {
            item: selected.item,
            signature,
            arguments,
        })?;
        let instance = Instance::new(&selected.facts.types, ty)?;
        let result = body::instantiate::with_contract(
            &mut selected.facts,
            &selected.template,
            instance,
            &selected.contract,
        );
        assert!(matches!(
            result,
            Err("instance signature differs from normalized template")
        ));
        Ok(())
    }

    #[test]
    fn erased_projection_inputs_keep_the_selected_implementations_lifetime_debt() -> TestResult {
        super::rustc::qualify("manual-quantified-instance", QUANTIFIED, true)?;
        for source in [SOURCE, QUANTIFIED] {
            let mut selected = selected_from(source)?;
            let argument = selected.facts.types.intern(Kind::Reference {
                region: RegionId(700),
                mutable: false,
                referent: TypeId::primitive(Primitive::U8),
            })?;
            let arguments = selected
                .facts
                .types
                .intern_arguments(&[Argument::Type(argument)])?;
            let ty = selected.facts.types.intern(Kind::FunctionItem {
                item: selected.item,
                signature: selected.signature,
                arguments,
            })?;
            let instance = Instance::new(&selected.facts.types, ty)?;
            let materialized = body::instantiate::with_contract(
                &mut selected.facts,
                &selected.template,
                instance,
                &selected.contract,
            )?;
            assert_eq!(
                materialized.contract.universals,
                selected.contract.universals + 1
            );
            let result = check(&selected.facts, materialized.contract);
            assert!(
                result
                    .as_ref()
                    .is_err_and(|e| e.contains("does not outlive")),
                "{result:?}"
            );
        }
        Ok(())
    }
}
