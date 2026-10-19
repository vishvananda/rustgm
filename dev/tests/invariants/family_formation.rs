//! A parent assumption never discharges the member's own conditions or lifetime debts.
#[path = "member_normalization.rs"]
mod search;
#[path = "member_templates.rs"]
mod templates;
use super::{Checker, Kind, TestResult, report, reserve};
use crate::{
    resolve::model::DefId,
    sema::{
        associated::OwnerKind,
        model::Declarations,
        nominals::traits::{Projected, normal_form},
        traits::{
            heads::Head,
            implementations::{Availability, Implementations, PendingHeader},
        },
    },
    types::{
        GenericArgs, ItemId, Primitive, RegionId, TypeId,
        arguments::Argument,
        lifetime_proof::Obligations,
        outlives::Queries,
        parameters::{
            ParameterId,
            environment::{Abilities, Environment},
            predicates::{Bound, Predicate},
        },
        projection::Application,
        relations::Constraints,
        substitute::{Normalizer, Step},
        traits,
    },
};
use std::sync::Arc;
const BYTE: TypeId = TypeId::primitive(Primitive::U8);
const WORD: TypeId = TypeId::primitive(Primitive::U16);

struct Fixture {
    facts: Declarations,
    template: Application,
    parameter: ParameterId,
    mark: Option<DefId>,
}
fn fixture(source: &str) -> TestResult<Fixture> {
    let report = report(source)?;
    let mut checker = Checker::new(&report)?;
    reserve(&mut checker)?;
    let member = *checker
        .facts
        .associated
        .members()
        .iter()
        .find(|m| {
            report
                .package
                .names
                .get(report.resolution.definitions[m.definition.0 as usize].name)
                == "Out"
        })
        .ok_or("missing family")?;
    let owner = checker.facts.associated.owners()[member.owner.0 as usize];
    let OwnerKind::Trait { definition, .. } = owner.kind else {
        return Err("missing family trait".into());
    };
    let key = report.resolution.definitions[member.definition.0 as usize]
        .node
        .ok_or("missing family source")?;
    checker.declare_associated_type(member.definition, key, owner, definition)?;
    checker.complete_associated_result_bounds(member.definition, key, &mut 0, 100_000)?;
    for member in checker.facts.associated.members().to_vec() {
        let owner = checker.facts.associated.owners()[member.owner.0 as usize];
        if matches!(owner.kind, OwnerKind::Impl { .. })
            && report.resolution.definitions[member.definition.0 as usize].kind
                == crate::resolve::model::DefKind::TypeAlias
        {
            checker
                .facts
                .generics
                .parents
                .insert(member.definition, owner.self_definition);
            checker.definition(member.definition, false)?;
            checker.drain(&mut 0, 100_000)?;
        }
    }
    let ty = checker
        .facts
        .associated
        .type_contract(member.definition)
        .ok_or("missing contract")?
        .projection;
    let Some(Kind::Projection(template)) = checker.facts.types.kind(ty) else {
        return Err("missing template".into());
    };
    let Some(Kind::Parameter(parameter)) = checker.facts.types.kind(template.subject) else {
        return Err("missing trait Self".into());
    };
    let mark = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == "Mark")
        .map(|(index, _)| u32::try_from(index).map(DefId))
        .transpose()?;
    Ok(Fixture {
        facts: checker.facts,
        template,
        parameter,
        mark,
    })
}
impl Fixture {
    fn environment(&mut self, extra: Option<TypeId>) -> TestResult<Arc<Environment>> {
        let parent = Predicate {
            ty: self.template.subject,
            bound: Bound::Trait(self.template.trait_application),
        };
        let mut clauses = vec![parent.into()];
        if let Some(ty) = extra {
            let trait_application =
                self.facts
                    .types
                    .intern_trait_application(traits::Application {
                        item: ItemId(self.mark.ok_or("missing Mark")?.0),
                        arguments: GenericArgs::EMPTY,
                    })?;
            clauses.push(
                Predicate {
                    ty,
                    bound: Bound::Trait(trait_application),
                }
                .into(),
            );
        }
        Ok(Arc::new(
            Environment::new(&self.facts.types, &[(self.parameter, Abilities::default())])?
                .with_clauses(&self.facts.types, &clauses)?,
        ))
    }
    fn begin(
        &mut self,
        own: &[Argument],
        environment: Arc<Environment>,
    ) -> TestResult<normal_form::Progress> {
        let arguments = self.facts.types.intern_arguments(own)?;
        let root = self.facts.types.intern(Kind::Projection(Application {
            arguments,
            ..self.template
        }))?;
        let mut fold = Normalizer::default();
        let Step::Resolve(request) = fold.start(&mut self.facts.types, root)? else {
            return Err("missing projection request".into());
        };
        Ok(normal_form::begin(
            &mut self.facts,
            Some(environment),
            request,
            100,
            false,
            Availability::Declarations,
        )?)
    }
    fn ready(&mut self, own: &[Argument], environment: Arc<Environment>) -> TestResult<Projected> {
        let normal_form::Progress::Ready(selected) = self.begin(own, environment)? else {
            return Err("unexpected header wait".into());
        };
        Ok(selected)
    }
    fn prove(&self, obligations: &Obligations, count: u32, assume: bool) -> TestResult<bool> {
        let mut proof = Constraints::new(
            &self.facts.types,
            &self.facts.lifetimes.variances,
            count,
            100_000,
        )?;
        if assume {
            proof.assume(RegionId::bound(0, 0)?, RegionId::bound(0, 1)?)?;
        }
        let mut queries = Queries::new(
            &self.facts.types,
            &self.facts.lifetimes.nominal_requirements,
            &self.facts.lifetimes.callables,
        );
        obligations.require(&mut proof, &mut queries)?;
        Ok(proof.prove().is_ok())
    }
}

#[test]
fn assumed_parent_still_requires_own_trait_conditions_and_caller_permissions() -> TestResult {
    let mut fixture =
        fixture("trait Mark{} impl Mark for u8{} trait Q{type Out<T> where T:Mark;}")?;
    for (ty, extra, accepted) in [
        (BYTE, None, true),
        (WORD, None, false),
        (WORD, Some(WORD), true),
        (WORD, None, false),
    ] {
        let environment = fixture.environment(extra)?;
        let selected = fixture.ready(&[Argument::Type(ty)], environment)?;
        assert_eq!(selected.source.is_some(), accepted);
    }
    Ok(())
}

#[test]
fn implicit_sized_is_a_member_input_requirement_unless_explicitly_relaxed() -> TestResult {
    for (source, accepted) in [
        ("trait Q{type Out<T>;}", false),
        ("trait Q{type Out<T:?Sized>;}", true),
    ] {
        let mut fixture = fixture(source)?;
        let slice = fixture.facts.types.intern(Kind::Slice(BYTE))?;
        let environment = fixture.environment(None)?;
        let selected = fixture.ready(&[Argument::Type(slice)], environment)?;
        assert_eq!(selected.source.is_some(), accepted);
    }
    Ok(())
}

#[test]
fn member_lifetime_relations_remain_debts_of_the_requesting_proof() -> TestResult {
    let mut fixture = fixture("trait Q{type Out<'a,'b> where 'a:'b;}")?;
    let environment = fixture.environment(None)?;
    let selected = fixture.ready(
        &[
            Argument::Lifetime(RegionId::bound(0, 0)?),
            Argument::Lifetime(RegionId::bound(0, 1)?),
        ],
        environment,
    )?;
    assert!(selected.source.is_some());
    assert!(!selected.proof.outlives.is_empty());
    assert!(!fixture.prove(&selected.proof, 2, false)?);
    assert!(fixture.prove(&selected.proof, 2, true)?);
    Ok(())
}

#[test]
fn quantified_member_conditions_keep_universal_scope_and_reject_static_only_evidence() -> TestResult
{
    for (implementation, accepted) in [
        ("impl<'a>Mark<'a> for u8{}", true),
        ("impl Mark<'static> for u8{}", false),
    ] {
        let mut fixture = fixture(&format!(
            "trait Mark<'a>{{}}{implementation}trait Q{{type Out<T> where for<'a>T:Mark<'a>;}}"
        ))?;
        let environment = fixture.environment(None)?;
        let selected = fixture.ready(&[Argument::Type(BYTE)], environment)?;
        assert!(selected.source.is_some());
        assert!(selected.proof.scope_count() > 0);
        assert_eq!(fixture.prove(&selected.proof, 0, false)?, accepted);
    }
    Ok(())
}

#[test]
fn family_conditions_retain_universal_proof_across_header_waits_and_foreign_allocations()
-> TestResult {
    let mut fixture = fixture(
        "trait Mark<'a>{}impl<'a>Mark<'a> for u8{}trait Q{type Out<T> where for<'a>T:Mark<'a>;}",
    )?;
    let registry = fixture
        .facts
        .generics
        .implementations
        .take()
        .ok_or("missing implementation inventory")?;
    let owner = {
        let mut headers = registry.headers();
        let owner = headers.next().ok_or("missing Mark implementation")?.owner;
        assert!(headers.next().is_none());
        owner
    };
    let mut pending = Implementations::default();
    pending.reserve(vec![PendingHeader {
        owner,
        item: ItemId(fixture.mark.ok_or("missing Mark")?.0),
        head: Some(Head::canonical(Kind::Primitive(Primitive::U8))),
    }])?;
    fixture.facts.generics.implementations = Some(Box::new(pending));
    let environment = fixture.environment(None)?;
    let normal_form::Progress::Pending(waiting, mut query) =
        fixture.begin(&[Argument::Type(BYTE)], environment)?
    else {
        return Err("family condition bypassed its pending header".into());
    };
    assert_eq!(waiting, owner);
    let end = query.allocation_end();
    for _ in 0..3 {
        let normal_form::Progress::Pending(waiting, retained) =
            query.resume(&mut fixture.facts, end)?
        else {
            return Err("pending family condition lost its continuation".into());
        };
        assert_eq!(waiting, owner);
        assert_eq!(retained.allocation_end(), end);
        query = retained;
    }
    fixture.facts.generics.implementations = Some(registry);
    let normal_form::Progress::Ready(selected) = query.resume(&mut fixture.facts, end + 7)? else {
        return Err("prepared family condition did not complete".into());
    };
    assert!(selected.source.is_some());
    assert!(selected.proof.scope_count() > 0);
    assert!(fixture.prove(&selected.proof, 0, false)?);
    for id in end..end + 7 {
        assert!(!selected.proof.owns_region(RegionId(id)));
    }
    Ok(())
}

#[test]
fn unprepared_family_contract_is_not_an_empty_successful_contract() -> TestResult {
    let mut fixture = fixture("trait Q{type Out<T>;}")?;
    fixture
        .facts
        .associated
        .families
        .get_mut(&DefId(fixture.template.item.0))
        .ok_or("missing family")?
        .requirements = None;
    let environment = fixture.environment(None)?;
    assert!(fixture.begin(&[Argument::Type(BYTE)], environment).is_err());
    Ok(())
}
