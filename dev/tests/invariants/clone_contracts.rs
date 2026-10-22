//! Canonical Clone bodies and marker premises keep separate proof authority.
use super::*;
use crate::{body, sema, types};
use std::{collections::HashMap, fmt::Write};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn fixture(source: &str) -> Result<(crate::resolve::expansion::Report, Declarations)> {
    let report = crate::test_source::resolve(source, true)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
#[test]
fn clone_original_keeps_registered_schema_and_two_independent_lifetime_slots() -> Result {
    let (_, mut facts) = fixture("struct V;fn main(){let a=[V];let _=a.iter().clone();}")?;
    let original = facts
        .cloning
        .as_deref()
        .and_then(|r| r.original)
        .ok_or("original")?;
    original.validate(&facts)?;
    for index in 0..4 {
        let mut forged = original;
        match index {
            0 => forged.declaration.definition = original.declaration.method,
            1 => forged.parameter = types::TypeId::UNIT,
            2 => forged.subject = types::TypeId::UNIT,
            _ => forged.predicate.ty = types::TypeId::UNIT,
        }
        assert!(forged.validate(&facts).is_err());
    }
    let scheme = facts
        .lifetimes
        .functions
        .get_mut(&original.declaration.method)
        .ok_or("scheme")?;
    assert_eq!(
        (scheme.total, scheme.captures, scheme.late.as_slice()),
        (2, 1, &[1][..])
    );
    scheme.late[0] = 0;
    assert!(original.validate(&facts).is_err());
    facts
        .lifetimes
        .functions
        .get_mut(&original.declaration.method)
        .ok_or("scheme")?
        .late[0] = 1;
    original.validate(&facts)?;
    facts.cloning.as_deref_mut().ok_or("registry")?.complete = false;
    assert!(original.validate(&facts).is_err());
    Ok(())
}
#[test]
fn clone_factory_enters_batch_and_incremental_original_proof() -> Result {
    let (report, mut facts) = fixture("struct V;fn main(){let a=[V];let _=a.iter().clone();}")?;
    for incremental in [false, true] {
        let mut contracts = if incremental {
            body::regions::Collector::default().finish(&facts)?.bodies
        } else {
            body::regions::collect(&facts)?.bodies
        };
        let mut templates = Templates { bodies: Vec::new() };
        append(&report, &mut facts, &mut templates, &mut contracts)?;
        assert_eq!(templates.bodies.len(), 7);
        body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
        verify(&mut facts, &templates, contracts.clone())?;
        let method = facts
            .cloning
            .as_deref()
            .and_then(|r| r.original)
            .ok_or("original")?
            .declaration
            .method;
        let template = templates.bodies.iter_mut().find(|t| matches!(t.body.owner, Owner::Function { definition, .. } if definition == method)).ok_or("template")?;
        assert_eq!(template.body.locals[0].kind, LocalKind::Parameter(0));
        template.body.locals[0].kind = LocalKind::Binding;
        assert!(verify(&mut facts, &templates, contracts).is_err());
    }
    Ok(())
}
#[test]
fn typed_member_demand_keeps_marker_and_source_winners_cold() -> Result {
    for source in [
        "fn main(){let _=1u8;}",
        "struct V;fn main(){let a=[V];let _=a.iter();}",
        "fn need<T:Clone>(){}struct V;fn main(){let a=[V];need::<core::slice::Iter<'_,V>>();let _=a.iter();}",
        "struct X;trait Local{fn clone(&self)->u8;}impl Local for X{fn clone(&self)->u8{9}}fn main(){let a=[1u8];let _i=a.iter();let _=X.clone();}",
        "struct X;impl X{fn clone(&self)->u8{9}}fn main(){let a=[1u8];let _i=a.iter();let _=X.clone();}",
    ] {
        let (report, mut facts) = fixture(source)?;
        assert!(
            facts
                .cloning
                .as_deref()
                .is_none_or(|r| !r.requested && r.original.is_none() && !r.complete)
        );
        let mut templates = Templates { bodies: Vec::new() };
        let mut contracts = Vec::new();
        append(&report, &mut facts, &mut templates, &mut contracts)?;
        assert!(templates.bodies.iter().all(|t| {
            let Owner::Function { definition, .. } = t.body.owner else {
                return false;
            };
            facts.cloning.as_deref().is_none_or(|r| {
                r.canonical
                    .implementation
                    .is_none_or(|i| definition != i.method)
            })
        }));
    }
    let (_, facts) = fixture(
        "fn duplicate<T:Clone>(x:&T)->T{x.clone()}struct V;fn main(){let a=[V];let _=duplicate(&a.iter());}",
    )?;
    assert!(
        facts
            .cloning
            .as_deref()
            .is_some_and(|r| r.requested && r.complete && r.original.is_some())
    );
    Ok(())
}
#[test]
fn fields_only_candidate_possibility_does_not_grant_actual_clone_permission() -> Result {
    use types::{
        aggregate::{Definitions, Kind as Aggregate, Repr, Variant},
        copy::{Copies, Declaration},
        infer::{Constructor, Context, properties::Query},
        parameters::environment::Environment,
    };
    let mut store = types::Store::default();
    let mut definitions = Definitions::default();
    let item = types::ItemId(912);
    let field = store.intern(Kind::Reference {
        referent: types::TypeId::primitive(types::Primitive::U8),
        region: types::RegionId::STATIC,
        mutable: true,
    })?;
    let fields = store.intern_list(&[field])?;
    definitions.insert(
        item,
        Aggregate::Struct,
        Repr::default(),
        &[Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let nominal = store.intern(Kind::Adt(item, types::GenericArgs::EMPTY))?;
    let selected = HashMap::from([(item, Declaration::fields_only_clone())]);
    let environment = Arc::new(Environment::new(&store, &[])?);
    for known in [false, true] {
        let mut context = Context::default();
        let term = if known {
            context.known(&store, nominal)?
        } else {
            context.construct(Constructor::Adt(item, types::GenericArgs::EMPTY), &[])?
        };
        let mut query = Query::new(Arc::clone(&environment));
        let possible = query.demand(&mut context, &mut store, &definitions, &selected, term)?;
        assert!(possible.sized && possible.abilities.clone_);
        let actual = Copies::default().demand(&store, &definitions, &selected, nominal)?;
        assert!(!actual.copy && !actual.clone_);
    }
    Ok(())
}
#[test]
fn iterator_clone_demand_is_cached_under_calls_and_unrelated_source_growth() -> Result {
    let mut observations = Vec::new();
    for (calls, unused) in [(8, 0), (32, 0), (128, 0), (1, 8), (1, 32), (1, 128)] {
        let mut source = "fn duplicate<T:Clone>(x:&T)->T{x.clone()}struct V;".to_owned();
        for index in 0..unused {
            write!(source, "struct U{index};fn unused{index}(_:U{index}){{}}")?;
        }
        source.push_str("fn main(){let a=[V];");
        for _ in 0..calls {
            source.push_str("let _=duplicate(&a.iter());");
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let facts = sema::declarations::analyze(&report)?;
        let before = (
            facts.lifetimes.nominal_requirements.steps,
            facts.lifetimes.variances.steps,
        );
        let facts = sema::bodies::analyze(&report, facts)?;
        let registry = facts.cloning.as_deref().ok_or("registry")?;
        assert!(registry.complete && registry.original.is_some());
        let original = registry.original.ok_or("original")?;
        original.validate(&facts)?;
        assert_eq!(
            facts
                .generics
                .implementations()
                .filter(|i| i.owner == original.declaration.definition)
                .count(),
            1
        );
        eprintln!(
            "clone calls={calls} unrelated={unused} inference={} selections={} metadata={} variances={}",
            facts.body_work.inference_steps,
            facts.body_work.trait_selections,
            facts.lifetimes.nominal_requirements.steps - before.0,
            facts.lifetimes.variances.steps - before.1
        );
        observations.push((
            facts.lifetimes.nominal_requirements.steps - before.0,
            facts.lifetimes.variances.steps - before.1,
        ));
    }
    assert!(observations.iter().all(|o| *o == observations[0]));
    Ok(())
}
#[test]
fn iterator_clone_preserves_ordinary_record_sizes_and_lazy_owner_cost() {
    eprintln!(
        "clone sizes declarations={} resolution={} semantic_registry={} resolver_declaration={} kind={} context={} store={}",
        size_of::<Declarations>(),
        size_of::<crate::resolve::model::Resolution>(),
        size_of::<sema::cloning::Registry>(),
        size_of::<crate::resolve::model::BuiltinCloning>(),
        size_of::<Kind>(),
        size_of::<types::infer::Context>(),
        size_of::<types::Store>()
    );
    // Nullable policies and layout caches add two cold pointers to the 8552-byte base.
    assert_eq!(size_of::<Declarations>(), 8568);
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2152);
    assert_eq!(size_of::<Kind>(), 24);
    assert_eq!(size_of::<types::infer::Context>(), 448);
    assert_eq!(size_of::<types::Store>(), 448);
}
