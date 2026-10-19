//! Family reservation precedes result bounds without replacing parent identities.
#[path = "unused_families.rs"]
mod conformance;
#[path = "family_formation.rs"]
mod formation;
#[path = "family_paths.rs"]
mod paths;
use super::Checker;
use crate::{
    resolve::{
        self,
        load::{InputFile, Sources},
        model::DefKind,
    },
    sema::associated::OwnerKind,
    support::source::SourceFile,
    types::{GenericArgs, Kind, RegionId, arguments::Argument},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
#[expect(
    clippy::default_trait_access,
    reason = "resolution fixture uses standard expansion defaults"
)]
fn report(source: &str) -> TestResult<resolve::expansion::Report> {
    let input = InputFile {
        identity: "family_parameters.rs".into(),
        path: "family_parameters.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    let expanded = resolve::expansion::analyze(input, &mut NoFiles, &Default::default())?;
    resolve::analyze_expanded(expanded).map_err(Into::into)
}

fn reserve(checker: &mut Checker<'_>) -> TestResult {
    let mut steps = 0;
    checker.reserve_member_parents();
    checker.reserve_nominals()?;
    checker.reserve_implementation_candidates()?;
    checker.reserve_inherent_headers()?;
    checker.complete_nominal_headers(&mut steps, 100_000)?;
    checker.prepare_trait_owners(&mut steps, 100_000)?;
    checker.prepare_trait_implementations(&mut steps, 100_000)?;
    checker.prepare_inherent_owners(&mut steps, 100_000)?;
    checker.reserve_family_parameters(false, &mut steps, 100_000)?;
    Ok(())
}

#[test]
fn trait_family_reservation_retains_unused_captures_and_separate_bound_syntax() -> TestResult {
    let report = report(
        "trait Q<'p,'unused,T>{type Plain;type Out<'b,'a,U,const N:usize>:'b where 'b:'a,T:'a;type Life<'z>;}",
    )?;
    let mut checker = Checker::new(&report)?;
    reserve(&mut checker)?;
    let members = checker.facts.associated.members().to_vec();
    for member in members {
        let id = member.definition;
        let definition = &report.resolution.definitions[id.0 as usize];
        if definition.kind != DefKind::TypeAlias {
            continue;
        }
        let key = definition.node.ok_or("missing source")?;
        let name = report.package.names.get(definition.name);
        let args = checker.template_arguments(id, key)?;
        if name == "Plain" {
            assert_eq!(args, GenericArgs::EMPTY);
            assert!(!checker.facts.associated.families.contains_key(&id));
            continue;
        }
        let family = &checker.facts.associated.families[&id];
        assert_eq!(family.captures, 2);
        assert_eq!(family.total, if name == "Out" { 4 } else { 3 });
        let own: Vec<_> = checker
            .facts
            .types
            .arguments(args)
            .ok_or("missing arguments")?
            .collect();
        assert_eq!(own[0], Argument::Lifetime(RegionId::bound(0, 2)?));
        if name == "Out" {
            assert_eq!(own[1], Argument::Lifetime(RegionId::bound(0, 3)?));
            assert_eq!(
                family.regions,
                [(RegionId::bound(0, 2)?, RegionId::bound(0, 3)?)]
            );
            let Argument::Type(ty) = own[2] else {
                return Err("missing type parameter".into());
            };
            let Some(Kind::Parameter(parameter)) = checker.facts.types.kind(ty) else {
                return Err("missing identity".into());
            };
            let parameter = checker
                .facts
                .types
                .parameter(parameter)
                .ok_or("invalid parameter")?;
            assert_eq!(parameter.owner.0, id.0);
            assert_eq!(parameter.index, 2);
            assert!(matches!(own[3], Argument::Const(_)));
            assert_eq!(checker.bound_syntax[&id].len(), 1);
            assert_eq!(checker.associated_type_bounds[&id].syntax.len(), 1);
        } else {
            assert_eq!(own.len(), 1);
        }
        let before = checker.owner_binders[&id];
        assert!(checker.template_context(id, key)?.binder == Some(before));
        assert_eq!(checker.template_arguments(id, key)?, args);
    }
    Ok(())
}

#[test]
fn impl_family_parameters_capture_the_prepared_owner_and_keep_own_order() -> TestResult {
    let report = report(
        "trait Q<T>{type Out<'a,U,const N:usize>;}struct S<'p,T>(&'p T);impl<'p,T>Q<T> for S<'p,T>{type Out<'a,U,const N:usize>=(T,U,[u8;N]);}",
    )?;
    let mut checker = Checker::new(&report)?;
    reserve(&mut checker)?;
    let member = checker
        .facts
        .associated
        .members()
        .iter()
        .find(|member| {
            matches!(
                checker.facts.associated.owners()[member.owner.0 as usize].kind,
                OwnerKind::Impl { .. }
            )
        })
        .copied()
        .ok_or("missing impl member")?;
    let family = &checker.facts.associated.families[&member.definition];
    assert_eq!((family.captures, family.total), (1, 2));
    let parameters = &checker.facts.generics.parameters[&member.definition];
    assert_eq!(parameters.len(), 3);
    assert_eq!(
        parameters[0].argument,
        Argument::Lifetime(RegionId::bound(0, 1)?)
    );
    let Argument::Type(ty) = parameters[1].argument else {
        return Err("missing type parameter".into());
    };
    let Some(Kind::Parameter(id)) = checker.facts.types.kind(ty) else {
        return Err("missing identity".into());
    };
    let parameter = checker
        .facts
        .types
        .parameter(id)
        .ok_or("invalid parameter")?;
    assert_eq!(parameter.owner.0, member.definition.0);
    assert_eq!(parameter.index, 1);
    Ok(())
}

#[test]
fn associated_parameter_defaults_are_rejected_during_reservation() -> TestResult {
    for source in [
        "trait Q {type Out<T=u8>;}",
        "trait Q {type Out<const N:usize=3>;}",
    ] {
        let report = report(source)?;
        let mut checker = Checker::new(&report)?;
        let error = reserve(&mut checker)
            .err()
            .ok_or("accepted family default")?;
        assert!(
            error
                .to_string()
                .contains("associated type parameters cannot have defaults")
        );
    }
    Ok(())
}

#[test]
fn input_requirements_and_projected_result_promises_remain_separate() -> TestResult {
    use crate::types::parameters::{clauses::Clause, predicates::Bound};
    let report = report("trait Q<T>{type Out<'a>:'a where T:'a;}")?;
    let mut checker = Checker::new(&report)?;
    reserve(&mut checker)?;
    let member = checker.facts.associated.members()[0];
    let owner = checker.facts.associated.owners()[member.owner.0 as usize];
    let OwnerKind::Trait { definition, .. } = owner.kind else {
        return Err("missing trait owner".into());
    };
    let id = member.definition;
    let key = report.resolution.definitions[id.0 as usize]
        .node
        .ok_or("missing source")?;
    let mut steps = 0;
    checker.declare_associated_type(id, key, owner, definition)?;
    checker.complete_associated_result_bounds(id, key, &mut steps, 100_000)?;
    let contract = checker
        .facts
        .associated
        .type_contract(id)
        .ok_or("missing result")?;
    let Some(Kind::Projection(projection)) = checker.facts.types.kind(contract.projection) else {
        return Err("missing family projection".into());
    };
    let own: Vec<_> = checker
        .facts
        .types
        .arguments(projection.arguments)
        .ok_or("missing own args")?
        .collect();
    assert_eq!(own, [Argument::Lifetime(RegionId::bound(0, 0)?)]);
    let requirements = checker.facts.associated.families[&id]
        .requirements
        .as_ref()
        .ok_or("unfinished family input contract")?;
    assert_eq!(requirements.len(), 1);
    let Clause::Predicate(input) = requirements[0] else {
        return Err("unexpected quantified input".into());
    };
    assert!(matches!(
        checker.facts.types.kind(input.ty),
        Some(Kind::Parameter(_))
    ));
    assert_eq!(input.bound, Bound::Outlives(RegionId::bound(0, 0)?));
    assert!(!contract.bounds.contains(&requirements[0]));
    assert!(contract.bounds.contains(&Clause::Predicate(
        crate::types::parameters::predicates::Predicate {
            ty: contract.projection,
            bound: input.bound,
        }
    )));
    crate::sema::traits::results::prepare_outlives(&mut checker.facts)?;
    let promises = checker
        .facts
        .types
        .projection_outlives(projection, 0)?
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(promises, [Some(RegionId::bound(0, 0)?)]);
    Ok(())
}
