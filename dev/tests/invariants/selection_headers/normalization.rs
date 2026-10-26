//! Header-dependent projections keep one provider, opened scope and authorizing proof.
use super::{
    Availability, Declarations, DefId, Implementation, ItemId, Primitive, TypeId, delay, publish,
    tests,
};
use crate::{
    resolve::unit::NodeKey,
    sema::{
        model::{BodyRegion, BodyRegionOrigin},
        nominals::traits::normal_form,
        projections::{Cursor, Definition, Progress},
        traits::obligations::Reservation,
    },
    types::{
        GenericArgs, Kind, RegionId,
        arguments::Argument,
        lifetime_proof::{Obligations, ScopeId},
        outlives::Queries,
        parameters::{
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        relations::Constraints,
        substitute::{Normalizer, Request, Resolution, Step},
    },
};
use std::{fmt::Write as _, sync::Arc};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

struct Fixture {
    facts: Declarations,
    pending: Vec<Implementation>,
    root: TypeId,
    expected: TypeId,
    source: NodeKey,
    base: DefId,
}

fn fixture(depth: usize, restricted: bool, base: &str, region: RegionId) -> TestResult<Fixture> {
    let restriction = if restricted { ":'static" } else { "" };
    let mut source = format!("trait B0<'a>{{}}impl<'a{restriction}>B0<'a> for {base}{{}}");
    for index in 1..=depth {
        write!(
            source,
            "trait B{index}<'a>{{}}impl<'a,T:B{}<'a>>B{index}<'a> for T{{}}",
            index - 1
        )?;
    }
    write!(
        source,
        "trait A<'a>{{type Out;}}impl<'a,T:B{depth}<'a>>A<'a> for T{{type Out=&'a u8;}}"
    )?;
    let (report, mut facts) = tests::declarations(&source)?;
    let owner = tests::definition(&report, "A")?;
    let item = tests::definition(&report, "Out")?;
    let mut traits = vec![owner];
    for index in 0..=depth {
        traits.push(tests::definition(&report, &format!("B{index}"))?);
    }
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(region)])?;
    let predicate = tests::predicate(
        &mut facts,
        owner,
        TypeId::primitive(Primitive::U8),
        arguments,
    )?;
    let Bound::Trait(trait_application) = predicate.bound else {
        return Err("missing trait application".into());
    };
    let root = facts
        .types
        .intern(Kind::Projection(crate::types::projection::Application {
            item: ItemId(item.0),
            subject: predicate.ty,
            trait_application,
            arguments: GenericArgs::EMPTY,
        }))?;
    let expected = facts.types.intern(Kind::Reference {
        mutable: false,
        region,
        referent: predicate.ty,
    })?;
    let source = report.resolution.definitions[owner.0 as usize]
        .node
        .ok_or("missing owner source")?;
    let pending = delay(&mut facts, &traits)?;
    Ok(Fixture {
        facts,
        pending,
        root,
        expected,
        source,
        base: tests::definition(&report, "B0")?,
    })
}

fn ready(
    fixture: &mut Fixture,
    dependency: crate::sema::traits::selection::Dependency,
) -> TestResult {
    let crate::sema::traits::selection::Dependency::Header(owner) = dependency else {
        return Err("expected an implementation header dependency".into());
    };
    let index = fixture
        .pending
        .iter()
        .position(|header| header.owner == owner)
        .ok_or("unexpected header dependency")?;
    publish(&mut fixture.facts, fixture.pending.swap_remove(index))?;
    Ok(())
}

fn foreign(fixture: &mut Fixture, count: usize) -> TestResult {
    for _ in 0..count {
        fixture.facts.body_regions.allocate(BodyRegion {
            body: fixture.source,
            source: fixture.source,
            origin: BodyRegionOrigin::Annotation,
        })?;
    }
    Ok(())
}

fn prove(facts: &Declarations, proof: &Obligations, universals: u32) -> TestResult<usize> {
    let mut constraints = Constraints::new(
        &facts.types,
        &facts.lifetimes.variances,
        universals,
        100_000,
    )?;
    let mut queries = Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    proof.require(&mut constraints, &mut queries)?;
    Ok(constraints.prove()?.identities.variables().len())
}

#[test]
fn scoped_projection_waits_preserve_one_provider_and_exclude_foreign_regions() -> TestResult {
    for (depth, restricted) in [(8, false), (32, false), (128, false), (8, true)] {
        let mut f = fixture(depth, restricted, "u8", RegionId::bound(0, 0)?)?;
        assert!(f.facts.body_regions.log().is_empty());
        let mut proof = Obligations::new(1);
        let scope = proof.scope(ScopeId::ROOT, 1)?;
        let mut cursor = Cursor::new(None, &proof, scope, Availability::Declarations)?;
        cursor.start(&mut f.facts, f.root)?;
        let mut reservation = Reservation::default();
        let mut calls = 0;
        let mut provider = |_: &mut Declarations, _: Request| {
            calls += 1;
            Ok(None)
        };
        let mut waits = 0;
        let mut gaps = Vec::new();
        loop {
            match cursor.advance(&mut f.facts, &mut proof, &mut provider)? {
                Progress::Dependency(owner) => {
                    let end = cursor.allocation_end(&proof);
                    reservation.reserve(
                        &mut f.facts,
                        f.source,
                        f.source,
                        proof.regions.start..end,
                    )?;
                    gaps.push(end..end + 3);
                    foreign(&mut f, 3)?;
                    let first = reservation.resume(&f.facts)?;
                    let types = f.facts.types.len();
                    for _ in 0..8 {
                        cursor.resume_at(&mut proof, first)?;
                        assert_eq!(
                            cursor.advance(&mut f.facts, &mut proof, &mut provider)?,
                            Progress::Dependency(owner)
                        );
                        assert_eq!(f.facts.types.len(), types);
                        assert_eq!(proof.regions, 1..1);
                        assert_eq!(cursor.allocation_end(&proof), first);
                    }
                    ready(&mut f, owner)?;
                    cursor.resume_at(&mut proof, first)?;
                    waits += 1;
                }
                Progress::Definition(_) => {
                    return Err("completed definitions were unexpectedly lost".into());
                }
                Progress::Complete { ty, steps } => {
                    assert_eq!(ty, f.expected);
                    assert!(steps < 32);
                    break;
                }
            }
        }
        assert_eq!(calls, 1);
        assert_eq!(waits, depth + 2);
        assert_eq!(proof.scope_count(), 1);
        for region in gaps.into_iter().flatten() {
            assert!(!proof.owns_region(RegionId(region)));
        }
        let owned: usize = proof.owned_regions().map(|range| range.len()).sum();
        assert_eq!(owned, depth + 3);
        if restricted {
            assert!(prove(&f.facts, &proof, 0).is_err());
        } else {
            assert_eq!(prove(&f.facts, &proof, 0)?, owned);
        }
        reservation.publish(proof, &mut f.facts, f.source, f.source)?;
    }
    Ok(())
}

#[test]
fn declaration_projection_templates_restore_results_before_fresh_rebasing() -> TestResult {
    let mut f = fixture(32, false, "u8", RegionId::bound(0, 0)?)?;
    let mut fold = Normalizer::default();
    let Step::Resolve(request) = fold.start(&mut f.facts.types, f.root)? else {
        return Err("missing projection request".into());
    };
    let mut progress = normal_form::begin(
        &mut f.facts,
        None,
        request,
        1,
        false,
        Availability::Declarations,
    )?;
    let projected = loop {
        match progress {
            normal_form::Progress::Pending(owner, query) => {
                foreign(&mut f, 3)?;
                ready(&mut f, owner)?;
                progress = query.resume_template(&mut f.facts)?;
            }
            normal_form::Progress::Ready(projected) => break projected,
        }
    };
    let definition = Definition::select(
        &f.facts,
        request
            .alias
            .associated()
            .ok_or("expected associated projection")?
            .item,
        projected.family,
        projected.source.ok_or("missing selected implementation")?,
    )?;
    let ty = definition
        .result(&mut f.facts)?
        .ok_or("missing selected definition type")?;
    let mut restore = projected.proof.restoration()?;
    let mut proof = projected.proof;
    assert!(!proof.has_gaps());
    proof.rebase(&mut f.facts.types, 1000)?;
    assert_eq!(restore.apply(&mut f.facts.types, ty)?, f.expected);
    assert_eq!(prove(&f.facts, &proof, 1)?, 35);
    assert_eq!(proof.regions, 1000..1035);
    Ok(())
}

// A concrete projection may first use a caller assumption, then try a closed
// implementation search for its value. Caller premises cannot authorize that
// fallback, and failed trials cannot grant their discarded lifetime variables.
#[test]
fn closed_projection_fallback_keeps_assumptions_and_discarded_trials_separate() -> TestResult {
    use crate::sema::traits::selection::Source;
    for (base, template) in [("u8", false), ("u16", false), ("u8", true), ("u16", true)] {
        let mut f = fixture(8, false, base, RegionId::STATIC)?;
        let mut fold = Normalizer::default();
        let Step::Resolve(request) = fold.start(&mut f.facts.types, f.root)? else {
            return Err("missing projection request".into());
        };
        let application = request.alias.associated().ok_or("expected projection")?;
        let root = Predicate {
            ty: application.subject,
            bound: Bound::Trait(application.trait_application),
        };
        let arguments = f
            .facts
            .types
            .trait_application(application.trait_application)
            .ok_or("missing trait")?
            .arguments;
        let premise = tests::predicate(&mut f.facts, f.base, root.ty, arguments)?;
        let environment = Arc::new(
            Environment::new(&f.facts.types, &[])?
                .with_predicates(&f.facts.types, &[root, premise])?,
        );
        let mut progress = normal_form::begin(
            &mut f.facts,
            Some(environment.clone()),
            request,
            1,
            false,
            Availability::Declarations,
        )?;
        let mut reservation = Reservation::default();
        let mut gaps = Vec::new();
        let mut waits = 0;
        let mut projected = loop {
            match progress {
                normal_form::Progress::Pending(owner, query) => {
                    let end = query.allocation_end();
                    if !template {
                        reservation.reserve(
                            &mut f.facts,
                            f.source,
                            f.source,
                            query.allocation_start()..end,
                        )?;
                    }
                    foreign(&mut f, 3)?;
                    ready(&mut f, owner)?;
                    progress = if template {
                        query.resume_template(&mut f.facts)?
                    } else {
                        gaps.push(end..end + 3);
                        let first = reservation.resume(&f.facts)?;
                        query.resume(&mut f.facts, first)?
                    };
                    waits += 1;
                }
                normal_form::Progress::Ready(projected) => break projected,
            }
        };
        assert_eq!(waits, if base == "u8" { 10 } else { 9 });
        assert_eq!(Arc::strong_count(&environment), 1);
        if base == "u8" {
            assert!(matches!(
                projected.source,
                Some(Source::Implementation { .. })
            ));
            let definition = Definition::select(
                &f.facts,
                application.item,
                projected.family,
                projected.source.ok_or("missing implementation")?,
            )?;
            let ty = definition.result(&mut f.facts)?.ok_or("missing result")?;
            assert_eq!(
                projected
                    .proof
                    .restoration()?
                    .apply(&mut f.facts.types, ty)?,
                f.expected
            );
        } else {
            let Some(Source::Assumption(premise)) = &projected.source else {
                return Err("failed closed search lost its original caller premise".into());
            };
            assert_eq!(*premise, root);
            assert_eq!(projected.resolution, Some(Resolution::Opaque));
        }
        if template {
            assert!(!projected.proof.has_gaps());
            projected.proof.rebase(&mut f.facts.types, 1000)?;
        } else {
            for id in gaps.into_iter().flatten() {
                assert!(!projected.proof.owns_region(RegionId(id)));
            }
        }
        assert_eq!(
            prove(&f.facts, &projected.proof, 1)?,
            if base == "u8" { 10 } else { 0 }
        );
    }
    Ok(())
}
