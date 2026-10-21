//! Default validation classifies original clauses and proves eligible whole requirements.
use super::{Clause, Proof, check_requirement};
use crate::rustc;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
        model::DefId,
    },
    sema::{declarations, model::Declarations},
    support::source::SourceFile,
    types::{
        ItemId, Kind, RegionId, Store, TypeId,
        arguments::Argument,
        outlives::Queries,
        parameters::{
            ParameterId,
            clauses::Quantified,
            predicates::{Bound, Predicate},
        },
        relations::Constraints,
        substitute, traits,
    },
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
struct Fixture {
    facts: Declarations,
    parameter: ParameterId,
    default: Argument,
    predicate: Predicate,
}
impl Fixture {
    fn new(source: &str) -> Result<Self> {
        let report = resolve::analyze_expanded(resolve::expansion::analyze(
            InputFile {
                identity: "default.rs".into(),
                path: "default.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
        )?)?;
        let owner = |name| -> Result<DefId> {
            let index = report
                .resolution
                .definitions
                .iter()
                .position(|d| report.package.names.get(d.name) == name)
                .ok_or("missing default fixture owner")?;
            Ok(DefId(u32::try_from(index)?))
        };
        let mut facts = declarations::analyze(&report)?;
        let parameter = facts.generics.parameters[&owner("S")?][0];
        let Argument::Type(ty) = parameter.argument else {
            return Err("missing type parameter".into());
        };
        let Some(Kind::Parameter(id)) = facts.types.kind(ty) else {
            return Err("invalid parameter type".into());
        };
        let application = facts.types.intern_trait_application(traits::Application {
            item: ItemId(owner("A")?.0),
            arguments: crate::types::GenericArgs::EMPTY,
        })?;
        Ok(Self {
            facts,
            parameter: id,
            default: parameter.default.ok_or("missing default")?,
            predicate: Predicate {
                ty,
                bound: Bound::Trait(application),
            },
        })
    }
}

fn quantified(store: &mut Store, predicate: Predicate, count: u32) -> Result<Clause> {
    Ok(Clause::ForAll(store.intern_quantified(Quantified {
        predicate,
        bound_regions: count,
    })?))
}

#[test]
fn default_clauses_keep_unused_quantifiers_and_validate_substituted_types() -> Result {
    for (name, default, implementation, accepted) in [
        ("missing", "u8", "", false),
        ("present", "u8", "impl A for u8{}", true),
        ("reference-missing", "&'static u8", "", false),
        (
            "reference-present",
            "&'static u8",
            "impl A for &'static u8{}",
            true,
        ),
    ] {
        let prefix = format!("trait A{{}}{implementation}struct S<T={default}>");
        rustc::qualify(
            &format!("default-unused-{name}"),
            &format!("{prefix}where for<'a>T:A{{v:T}}fn main(){{}}"),
            accepted,
        )?;
        for count in [0, 1, 8] {
            let mut fixture = Fixture::new(&format!("{prefix}{{v:T}}"))?;
            let clause = quantified(&mut fixture.facts.types, fixture.predicate, count)?;
            assert!(clause.checks_parameter_default(&fixture.facts.types)?);
            let mut substitute = substitute::Bound::parameter_only(
                &fixture.facts.types,
                fixture.parameter,
                fixture.default,
            )?;
            let bound = substitute.clause(&mut fixture.facts.types, clause)?;
            let mut proof = Proof::for_declaration(None, 0, 1);
            assert_eq!(
                check_requirement(&mut fixture.facts, &mut proof, bound)?,
                accepted,
                "{name}"
            );
            if accepted {
                let evidence = proof
                    .take_trait_proof()
                    .ok_or("missing default scope proof")?;
                assert_eq!(evidence.scope_count(), 1);
                let facts = &fixture.facts;
                let mut constraints =
                    Constraints::new(&facts.types, &facts.lifetimes.variances, 0, 100_000)?;
                let mut queries = Queries::new(
                    &facts.types,
                    &facts.lifetimes.nominal_requirements,
                    &facts.lifetimes.callables,
                );
                evidence.require(&mut constraints, &mut queries)?;
                constraints.prove()?;
            }
        }
    }
    Ok(())
}

#[test]
fn default_clauses_defer_every_written_lifetime_occurrence() -> Result {
    let mut fixture = Fixture::new("trait A{}struct S<T=u8>{v:T}")?;
    let store = &mut fixture.facts.types;
    let local = RegionId::bound(0, 0)?;
    for region in [RegionId::STATIC, local, RegionId::bound(1, 0)?] {
        let reference = store.intern(Kind::Reference {
            mutable: false,
            region,
            referent: fixture.predicate.ty,
        })?;
        let arguments = store.intern_arguments(&[Argument::Lifetime(region)])?;
        let application = store.intern_trait_application(traits::Application {
            item: ItemId(999),
            arguments,
        })?;
        for predicate in [
            Predicate {
                ty: reference,
                bound: Bound::Copy,
            },
            Predicate {
                ty: fixture.predicate.ty,
                bound: Bound::Trait(application),
            },
            Predicate {
                ty: fixture.predicate.ty,
                bound: Bound::ProjectionEqual(reference),
            },
        ] {
            assert!(!quantified(store, predicate, 1)?.checks_parameter_default(store)?);
        }
    }
    let reference = store.intern(Kind::Reference {
        mutable: false,
        region: local,
        referent: TypeId::UNIT,
    })?;
    let parameters = store.intern_list(&[reference])?;
    let callable = store.intern(Kind::Function {
        bound_regions: 1,
        parameters,
        result: TypeId::UNIT,
        unsafe_: false,
        convention: crate::types::callable::Convention::RUST,
    })?;
    let arguments = store.intern_arguments(&[Argument::Type(callable)])?;
    let application = store.intern_trait_application(traits::Application {
        item: ItemId(1000),
        arguments,
    })?;
    let predicate = Predicate {
        ty: fixture.predicate.ty,
        bound: Bound::Trait(application),
    };
    assert!(!quantified(store, predicate, 1)?.checks_parameter_default(store)?);
    Ok(())
}
