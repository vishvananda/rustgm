//! Scoped formation survives real template specialization, including contract-only projections.
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body::{
        self,
        ir::Owner,
        regions::{Contract, Contracts, Kind as Debt, Obligation},
        templates::Template,
    },
    sema::model::Declarations,
    types::{
        GenericArgs, ItemId, Kind, RegionId, TypeId, arguments::Argument, instance::Instance,
        parameters::predicates::Bound, projection, traits,
    },
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    facts: Declarations,
    template: Template,
    instance: Instance,
    contract: Contract,
    parameter: TypeId,
}
impl Fixture {
    fn new(source: &str) -> Result<Self> {
        let (report, facts) = semantic::analyze(source)?;
        let mut templates = body::templates::build(&report, &facts)?;
        assert_eq!(templates.bodies.len(), 1);
        let template = templates.bodies.pop().ok_or("missing template")?;
        let Owner::Function { definition, .. } = template.body.owner else {
            return Err("template is not a function".into());
        };
        let parameter = facts
            .generics
            .parameters_in_scope(definition)
            .find_map(|p| {
                if let Argument::Type(ty) = p.argument {
                    Some(ty)
                } else {
                    None
                }
            })
            .ok_or("missing generic parameter")?;
        let ordinary = body::build_concrete(&report, &facts)?;
        let selected = ordinary.bodies.iter().flat_map(|body| &body.values)
            .find(|value| matches!(facts.types.kind(value.ty), Some(Kind::FunctionItem { item, .. }) if item.0 == definition.0))
            .ok_or("missing selected instance")?.ty;
        let instance = Instance::new(&facts.types, selected)?;
        let contract = body::regions::collect(&facts)?
            .bodies
            .into_iter()
            .find(|c| c.source == template.body.source)
            .ok_or("missing template contract")?;
        Ok(Self {
            facts,
            template,
            instance,
            contract,
            parameter,
        })
    }
    fn add(&mut self, ty: TypeId, count: u32) {
        self.contract.obligations.push(Obligation {
            source: self.contract.source,
            kind: Debt::ScopedWellFormed {
                ty,
                bound_regions: count,
            },
        });
    }
    fn materialize(&mut self) -> Result<body::instantiate::Materialized> {
        Ok(body::instantiate::with_contract(
            &mut self.facts,
            &self.template,
            self.instance,
            &self.contract,
        )?)
    }
    fn check(&self, contract: Contract, valid: bool) {
        let contracts = Contracts {
            bodies: vec![contract],
            callables: &self.facts.lifetimes.callables,
            nominal_requirements: &self.facts.lifetimes.nominal_requirements,
        };
        let result = rustgm::borrow::regions::check(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            &contracts,
        );
        assert_eq!(result.is_ok(), valid, "{:?}", result.as_ref().err());
        if let Err(error) = result {
            assert!(error.message.contains("does not outlive"), "{error}");
            assert_eq!(error.source, Some(self.contract.source));
        }
    }
}

#[test]
fn substitution_preserves_local_binders_and_lifts_actual_argument_captures() -> Result {
    let mut f = Fixture::new("fn f<T>(x:T){}fn use_<'x>(x:&'x u8){f(x)}")?;
    let ty = f.facts.types.intern(Kind::Reference {
        region: RegionId::bound(0, 0)?,
        referent: f.parameter,
        mutable: false,
    })?;
    f.add(ty, 1);
    let materialized = f.materialize()?;
    let specialized = materialized
        .contract
        .obligations
        .iter()
        .find_map(|o| {
            if let Debt::ScopedWellFormed {
                ty,
                bound_regions: 1,
            } = o.kind
            {
                Some(ty)
            } else {
                None
            }
        })
        .ok_or("specialization lost the scoped formation term")?;
    let Some(Kind::Reference {
        region, referent, ..
    }) = f.facts.types.kind(specialized)
    else {
        return Err("missing specialized reference".into());
    };
    assert_eq!(region, RegionId::bound(0, 0)?);
    let Some(Kind::Reference { region, .. }) = f.facts.types.kind(referent) else {
        return Err("missing concrete argument".into());
    };
    assert_eq!(region.bound_parts(), Some((1, 0)));
    f.check(materialized.contract, true);
    Ok(())
}

#[test]
fn a_projection_only_in_the_formation_contract_still_requires_its_selection_debts() -> Result {
    for restricted in [false, true] {
        for count in [0, 1] {
            let bound = if restricted { ":'static" } else { "" };
            let mut f = Fixture::new(&format!(
                "trait Q<'a>{{type Out;}}struct S;impl<'a{bound}>Q<'a>for S{{type Out=&'a u8;}}fn f<T>(){{}}fn main(){{f::<S>()}}"
            ))?;
            let implementation = f
                .facts
                .generics
                .implementations()
                .next()
                .ok_or("missing impl")?;
            let Bound::Trait(application) = implementation.predicate.bound else {
                return Err("missing trait application".into());
            };
            let trait_item = f
                .facts
                .types
                .trait_application(application)
                .ok_or("missing trait")?
                .item;
            let member = ItemId(implementation.types.first().ok_or("missing member")?.0.0);
            let local = if count == 0 {
                RegionId::STATIC
            } else {
                RegionId::bound(0, 0)?
            };
            let arguments = f
                .facts
                .types
                .intern_arguments(&[Argument::Lifetime(local)])?;
            let trait_application =
                f.facts
                    .types
                    .intern_trait_application(traits::Application {
                        item: trait_item,
                        arguments,
                    })?;
            let projected = f
                .facts
                .types
                .intern(Kind::Projection(projection::Application {
                    item: member,
                    subject: f.parameter,
                    trait_application,
                    arguments: GenericArgs::EMPTY,
                }))?;
            f.add(projected, count);
            let materialized = f.materialize()?;
            let ty = materialized
                .contract
                .obligations
                .iter()
                .find_map(|o| {
                    if let Debt::ScopedWellFormed { ty, .. } = o.kind {
                        Some(ty)
                    } else {
                        None
                    }
                })
                .ok_or("missing formation obligation")?;
            assert_eq!(f.facts.types.contains_projections(ty), Some(false));
            assert!(materialized.contract.type_assumptions.is_empty());
            assert!(materialized.contract.assumptions.is_empty());
            assert!(!materialized.contract.proofs.is_empty());
            f.check(materialized.contract, !restricted || count == 0);
        }
    }
    Ok(())
}
