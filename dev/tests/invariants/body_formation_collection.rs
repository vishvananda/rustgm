//! Typed clause declarations feed their actual body formation and final lifetime gate.
use crate::{
    body::regions,
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    sema,
    support::source::SourceFile,
    types::{
        Kind, RegionId, TypeId,
        arguments::Argument,
        parameters::{
            clauses::{Clause, Quantified},
            predicates::{Bound, Predicate},
        },
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
fn reference(
    types: &mut crate::types::Store,
    region: RegionId,
    referent: TypeId,
) -> std::result::Result<TypeId, &'static str> {
    types.intern(Kind::Reference {
        region,
        referent,
        mutable: false,
    })
}

fn predicate(
    facts: &mut sema::model::Declarations,
    report: &resolve::expansion::Report,
    trait_predicate: Predicate,
    mode: u8,
) -> Result<Predicate> {
    let t = trait_predicate.ty;
    let Bound::Trait(application) = trait_predicate.bound else {
        return Err("missing trait".into());
    };
    let trait_item = facts
        .types
        .trait_application(application)
        .ok_or("missing inputs")?
        .item;
    let captured = reference(&mut facts.types, RegionId::bound(1, 0)?, t)?;
    let local = reference(&mut facts.types, RegionId::bound(0, 0)?, captured)?;
    let mut value = Predicate {
        ty: local,
        bound: Bound::Outlives(RegionId::bound(0, 0)?),
    };
    if matches!(mode, 1 | 2) {
        let arguments = facts
            .types
            .intern_arguments(&[Argument::Lifetime(RegionId::bound(0, 0)?)])?;
        let application =
            facts
                .types
                .intern_trait_application(crate::types::traits::Application {
                    item: trait_item,
                    arguments,
                })?;
        value = Predicate {
            ty: if mode == 1 { local } else { captured },
            bound: Bound::Trait(application),
        };
    } else if mode == 3 {
        let member = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == "Out")
            .ok_or("missing associated result")?;
        let member = crate::types::ItemId(u32::try_from(member)?);
        let subject =
            facts
                .types
                .intern(Kind::Projection(crate::types::projection::Application {
                    item: member,
                    subject: t,
                    trait_application: application,
                    arguments: crate::types::GenericArgs::EMPTY,
                }))?;
        value = Predicate {
            ty: subject,
            bound: Bound::ProjectionEqual(local),
        };
    } else if mode == 4 {
        let a = reference(&mut facts.types, RegionId::bound(0, 0)?, TypeId::UNIT)?;
        let fields = facts.types.intern_list(&[a, t])?;
        let mixed = facts.types.intern(Kind::Tuple(fields))?;
        value.ty = reference(&mut facts.types, RegionId::bound(1, 0)?, mixed)?;
    }
    Ok(value)
}

#[test]
fn declaration_clauses_reach_body_formation_with_their_independent_debts() -> Result {
    for mode in 0..5 {
        for given in [false, true] {
            // Public HRTB source remains gated; replace a complete ordinary
            // declaration with the independently qualified canonical clause.
            let text = "trait A<'a>{type Out;}fn f<'x,T:A<'x>>()where T:'x{}";
            let report = resolve::analyze_expanded(resolve::expansion::analyze(
                InputFile {
                    identity: "formation.rs".into(),
                    path: "formation.rs".into(),
                    source: SourceFile::new(text.as_bytes().to_vec())?,
                },
                &mut NoFiles,
                &Config::default(),
            )?)?;
            let mut facts = sema::declarations::analyze(&report)?;
            let function = facts
                .functions
                .iter()
                .find(|f| f.body.is_some())
                .ok_or("missing body")?;
            let owner = function.definition;
            let source = function.body.ok_or("missing body source")?;
            let premises = &facts.generics.predicates[&owner];
            let trait_predicate = premises
                .iter()
                .find_map(|p| match p.value {
                    Clause::Predicate(p) if matches!(p.bound, Bound::Trait(_)) => Some(p),
                    _ => None,
                })
                .ok_or("missing trait premise")?;
            let t = trait_predicate.ty;
            let value = predicate(&mut facts, &report, trait_predicate, mode)?;
            let clause = Clause::ForAll(facts.types.intern_quantified(Quantified {
                bound_regions: 1,
                predicate: value,
            })?);
            if matches!(value.bound, Bound::Trait(_)) {
                facts
                    .generics
                    .traits
                    .as_mut()
                    .ok_or("missing trait contracts")?
                    .formation(&mut facts.types, clause)?;
            }
            let mut predicates = vec![sema::generics::Predicate {
                source,
                value: clause,
            }];
            if given {
                predicates.push(sema::generics::Predicate {
                    source,
                    value: Predicate {
                        ty: t,
                        bound: Bound::Outlives(RegionId::bound(0, 0)?),
                    }
                    .into(),
                });
            }
            facts.generics.predicates.insert(owner, predicates);
            let contracts = regions::collect(&facts)?;
            let result =
                crate::borrow::regions::check(&facts.types, &facts.lifetimes.variances, &contracts);
            let valid = given || matches!(mode, 1 | 4);
            assert_eq!(
                result.is_ok(),
                valid,
                "mode {mode}, given {given}: {:?}",
                result.as_ref().err()
            );
            if let Err(error) = result {
                assert!(error.message.contains("does not outlive"), "{error}");
                assert_eq!(error.source, Some(source));
            }
        }
    }
    Ok(())
}
