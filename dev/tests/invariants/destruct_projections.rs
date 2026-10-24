//! Projection promises retain complete applications and require completed original proof.
use super::*;
use crate::borrow::loans::Formed;

fn original(name: &str) -> Result<(Declarations, Template, Contract)> {
    let path = format!(
        "pa31/tests/spec/trait-default-effects/projections/public-context/public-{name}.rs"
    );
    let (_report, mut facts, template, mut contract) = fixture(&path)?;
    crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
    Ok((facts, template, contract))
}
fn state(contract: &Contract) -> Result<&State> {
    Ok(contract
        .builtins
        .as_deref()
        .and_then(|b| b.defining())
        .ok_or("actual projected requirements")?)
}
fn subject(facts: &Declarations, template: &Template) -> Result<TypeId> {
    Ok(template
        .body
        .locals
        .iter()
        .map(|local| local.ty)
        .find(|&ty| {
            matches!(
                facts.types.kind(ty),
                Some(crate::types::Kind::Projection(_))
            )
        })
        .ok_or("actual fully live projected parameter")?)
}
fn regions(
    facts: &mut Declarations,
    template: &Template,
    contract: &Contract,
) -> Result<crate::borrow::regions::Body> {
    contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    let mut queries = crate::types::outlives::Queries::new(
        &facts.types,
        &facts.lifetimes.nominal_requirements,
        &facts.lifetimes.callables,
    );
    Ok(crate::borrow::regions::check_one(
        &facts.types,
        &facts.lifetimes.variances,
        &mut queries,
        contract,
        &template.body,
    )?)
}

#[test]
fn projected_reads_open_the_actual_body_and_declaration_premises_in_one_scope() -> Result {
    for name in ["provider-moved", "provider-partial-container"] {
        let path = format!(
            "pa31/tests/spec/trait-default-effects/projections/public-context/public-{name}.rs"
        );
        let (_report, mut facts, template, mut contract) = fixture(&path)?;
        let capture = crate::sema::copy_trace::Capture::new()?;
        let result =
            crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract);
        let rows = capture.rows()?;
        drop(capture);
        for row in &rows {
            println!("copy_attribution={name} {row}");
        }
        assert!(
            rows.iter()
                .any(|row| row.starts_with("copy-classification"))
        );
        result?;
        assert!(
            !rows
                .iter()
                .any(|row| row.starts_with("assumption-ambiguous"))
        );
    }
    Ok(())
}

#[test]
fn projected_claims_require_genuine_region_formation_and_exact_retained_carrier() -> Result {
    let (mut facts, template, contract) = original("provider-paid")?;
    let ty = subject(&facts, &template)?;
    let requirements = state(&contract)?;
    let builtins = contract.builtins.as_deref().ok_or("builtins")?;
    let mut work = Work::new(1_000_000);
    let prepared =
        requirements.checked(&template.body, builtins.environment.as_ref(), &mut work)?;
    let error = prepared
        .check(&facts.types, ty, &mut work)
        .err()
        .ok_or("raw projected promise consumed")?;
    assert!(error.contains("completed region formation"), "{error}");
    let proof = regions(&mut facts, &template, &contract)?;
    let formed = Formed::new(requirements, &template.body, &proof, &mut work)?;
    formed.check(&facts.types, ty, &mut work)?;
    let (mut foreign_facts, foreign_template, foreign_contract) = original("provider-paid")?;
    let foreign = regions(&mut foreign_facts, &foreign_template, &foreign_contract)?;
    assert_eq!(foreign.key, proof.key);
    assert_eq!(foreign.source, proof.source);
    assert!(Formed::new(requirements, &template.body, &foreign, &mut work).is_err());
    let mut missing = proof;
    missing.builtins = None;
    assert!(Formed::new(requirements, &template.body, &missing, &mut work).is_err());
    let mut tiny = Work::new(0);
    assert!(Formed::new(requirements, &template.body, &missing, &mut tiny).is_err());
    assert!(tiny.steps > 0);
    verify(&mut facts, template, contract)?;
    Ok(())
}

#[test]
fn failed_formation_after_indexing_never_produces_a_projected_use() -> Result {
    let (mut facts, template, mut contract) = original("provider-paid")?;
    let ty = subject(&facts, &template)?;
    let prepared = state(&contract)?.checked(
        &template.body,
        contract
            .builtins
            .as_deref()
            .ok_or("builtins")?
            .environment
            .as_ref(),
        &mut Work::new(1_000_000),
    )?;
    assert!(matches!(
        prepared.claims.get(ty),
        Some(claims::Claim::ProjectedDestruct)
    ));
    contract.obligations.push(crate::body::regions::Obligation {
        source: template.body.source.into(),
        kind: crate::body::regions::Kind::WellFormed(TypeId(u32::MAX)),
    });
    assert!(regions(&mut facts, &template, &contract).is_err());
    let builtins = contract.builtins.as_deref().ok_or("builtins")?;
    let mut work = Work::new(1_000_000);
    assert!(
        state(&contract)?
            .checked(&template.body, builtins.environment.as_ref(), &mut work)?
            .check(&facts.types, ty, &mut work)
            .is_err()
    );
    Ok(())
}

#[test]
fn projected_preparation_rejects_lost_current_headers_and_moved_values_need_no_claim() -> Result {
    let (mut facts, template, mut contract) = original("provider-paid")?;
    facts.associated = crate::sema::associated::Inventory::default();
    let before = facts.body_work.canonical_steps;
    assert!(
        crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract).is_err()
    );
    assert!(facts.body_work.canonical_steps > before);
    assert!(matches!(state(&contract)?, State::Required(_)));
    let (mut facts, template, contract) = original("provider-moved")?;
    let State::Prepared(prepared) = state(&contract)? else {
        return Err("required".into());
    };
    assert_eq!(prepared.claims.projected_subjects().count(), 0);
    verify(&mut facts, template, contract)?;
    Ok(())
}

fn trace_application(
    facts: &Declarations,
    template: &Template,
    name: &str,
    (ty, application): (TypeId, crate::types::projection::Application),
) -> Result {
    let trait_data = facts
        .types
        .trait_application(application.trait_application)
        .ok_or("actual projection trait application")?;
    let trait_arguments = facts
        .types
        .arguments(trait_data.arguments)
        .ok_or("actual projection trait arguments")?
        .collect::<Vec<_>>();
    let family_arguments = facts
        .types
        .arguments(application.arguments)
        .ok_or("actual projection family arguments")?
        .collect::<Vec<_>>();
    println!(
        "projection_index={name} ty={ty:?} application={application:?} subject={:?} trait_item={:?} trait_arguments={trait_arguments:?} family_arguments={family_arguments:?} body={:?} signatures={:?}",
        facts.types.kind(application.subject),
        trait_data.item,
        template.body.source,
        template.body.owner.signatures()
    );
    Ok(())
}

#[test]
fn projected_index_diagnostics_keep_both_original_application_families() -> Result {
    for name in ["provider-trait-arguments", "provider-family-arguments"] {
        let (facts, template, contract) = original(name)?;
        let State::Prepared(prepared) = state(&contract)? else {
            return Err("actual prepared projection index".into());
        };
        let mut subjects = prepared.claims.projected_subjects().collect::<Vec<_>>();
        subjects.sort_unstable_by_key(|ty| ty.0);
        assert!(!subjects.is_empty());
        for ty in subjects {
            let Some(crate::types::Kind::Projection(application)) = facts.types.kind(ty) else {
                return Err("indexed subject lost its projection".into());
            };
            trace_application(&facts, &template, name, (ty, application))?;
        }
    }
    Ok(())
}

#[test]
fn full_trait_and_family_arguments_never_share_a_defining_claim() -> Result {
    for name in ["provider-trait-arguments", "provider-family-arguments"] {
        let (mut facts, template, contract) = original(name)?;
        let prepared = match state(&contract)? {
            State::Prepared(prepared) => prepared,
            State::Required(_) => return Err("unprepared".into()),
        };
        let mut applications = prepared
            .claims
            .projected_subjects()
            .map(|ty| match facts.types.kind(ty) {
                Some(crate::types::Kind::Projection(application)) => Ok((ty, application)),
                _ => Err("claim lost its projection"),
            })
            .collect::<std::result::Result<Vec<_>, _>>()?;
        applications.sort_unstable_by_key(|(ty, _)| ty.0);
        for &(ty, application) in &applications {
            trace_application(&facts, &template, name, (ty, application))?;
        }
        let subjects = actual_projection_subjects(&mut facts, &template, name, &applications)?;
        let proof = regions(&mut facts, &template, &contract)?;
        let formed = Formed::new(
            state(&contract)?,
            &template.body,
            &proof,
            &mut Work::new(1_000_000),
        )?;
        for count in [8, 32, 128] {
            let mut work = Work::new(1_000_000);
            for _ in 0..count {
                for &ty in &subjects {
                    formed.check(&facts.types, ty, &mut work)?;
                }
            }
            assert_eq!(work.steps, count * subjects.len());
            println!(
                "projection_exact={name} repeats={count} lookup_steps={} retained={}",
                work.steps,
                prepared.claims.retained_bytes()
            );
        }
        verify(&mut facts, template, contract)?;
    }
    Ok(())
}

// This relation checks retained source namespaces; it does not construct formation proof.
fn actual_projection_subjects(
    facts: &mut Declarations,
    template: &Template,
    name: &str,
    applications: &[(TypeId, crate::types::projection::Application)],
) -> Result<crate::support::hash::HashSet<TypeId>> {
    let aliases = original_aliases(facts, template)?;
    let mut declared = Vec::new();
    let mut opened = Vec::new();
    for &(ty, application) in applications {
        let trait_data = facts
            .types
            .trait_application(application.trait_application)
            .ok_or("actual projection trait application")?;
        let first = facts
            .types
            .arguments(trait_data.arguments)
            .and_then(|mut arguments| arguments.next())
            .ok_or("declared trait lifetime argument")?;
        match first {
            crate::types::arguments::Argument::Lifetime(region)
                if region == crate::types::RegionId::bound(0, 0)? =>
            {
                declared.push((ty, application));
            }
            crate::types::arguments::Argument::Lifetime(region)
                if aliases.first() == Some(&region) =>
            {
                opened.push((ty, application));
            }
            _ => return Err("projection lost its actual declared/opened lifetime context".into()),
        }
    }
    for group in [&declared, &opened] {
        assert_eq!(group.len(), 2, "{name}");
        let [left, right] = group.as_slice() else {
            return Err("full applications in each proven namespace".into());
        };
        assert_eq!(left.1.item, right.1.item);
        assert_eq!(left.1.subject, right.1.subject);
        assert_ne!(left.0, right.0);
        if name == "provider-trait-arguments" {
            assert_ne!(left.1.trait_application, right.1.trait_application);
        } else {
            assert_ne!(left.1.arguments, right.1.arguments);
        }
    }
    let mut opening = crate::types::substitute::Bound::new(aliases);
    for &(ty, _) in &declared {
        let mapped = opening.apply(&mut facts.types, ty)?;
        assert_eq!(opened.iter().filter(|(ty, _)| *ty == mapped).count(), 1);
    }
    let subjects = template
        .body
        .locals
        .iter()
        .map(|local| local.ty)
        .filter(|&ty| {
            matches!(
                facts.types.kind(ty),
                Some(crate::types::Kind::Projection(_))
            )
        })
        .collect::<crate::support::hash::HashSet<_>>();
    assert_eq!(
        subjects.len(),
        1,
        "fixture declares one projected formal; the other premise is unused"
    );
    assert!(
        subjects
            .iter()
            .all(|ty| applications.iter().any(|(known, _)| known == ty))
    );
    Ok(subjects)
}

fn original_aliases(
    facts: &Declarations,
    template: &Template,
) -> Result<Vec<crate::types::RegionId>> {
    let crate::body::ir::Owner::Function { definition, .. } = template.body.owner else {
        return Err("actual function owner".into());
    };
    let scheme = facts
        .lifetimes
        .functions
        .get(&definition)
        .ok_or("actual callable scheme")?;
    assert_eq!(scheme.total, 1, "fixture has one declared parent lifetime");
    let body = template.body.source.source().ok_or("actual source body")?;
    let mut aliases = vec![None; usize::try_from(scheme.total)?];
    for equality in facts
        .region_equalities
        .iter()
        .filter(|equality| equality.body == body)
    {
        for (bound, alias) in [
            (equality.left, equality.right),
            (equality.right, equality.left),
        ] {
            let Some((0, index)) = bound.bound_parts() else {
                continue;
            };
            let Some(slot) = aliases.get_mut(usize::from(index)) else {
                continue;
            };
            let Some(origin) = facts.body_regions.get(body, alias) else {
                continue;
            };
            if origin.body != body
                || !matches!(
                    origin.origin,
                    crate::sema::model::BodyRegionOrigin::UniversalAlias
                )
            {
                continue;
            }
            if slot.is_some_and(|actual| actual != alias) {
                return Err("actual universal has multiple retained aliases".into());
            }
            *slot = Some(alias);
        }
    }
    aliases
        .into_iter()
        .map(|alias| alias.ok_or_else(|| "missing actual universal opening".into()))
        .collect()
}

#[test]
fn projected_subjects_and_cleanup_sites_have_bounded_work_and_noise() -> Result {
    let mut baseline = None;
    for (axis, count, noise) in [
        ("subjects", 8, 0),
        ("subjects", 32, 0),
        ("subjects", 128, 0),
        ("subjects", 8, 128),
        ("sites", 8, 0),
        ("sites", 32, 0),
        ("sites", 128, 0),
    ] {
        let name = format!("{axis}-{count}-noise-{noise}");
        let path = format!("pa31/tests/spec/trait-default-effects/projections/{name}.rs");
        let (_report, mut facts, template, mut contract) = fixture(&path)?;
        let before = facts.body_work.canonical_steps;
        crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
        let work = facts.body_work.canonical_steps - before;
        let requirements = state(&contract)?;
        let expected = if axis == "subjects" { count } else { 1 };
        let State::Prepared(prepared) = requirements else {
            return Err("required".into());
        };
        assert_eq!(prepared.claims.projected_subjects().count(), expected);
        let bytes = requirements.retained_bytes();
        println!(
            "projection_axis={axis} count={count} noise={noise} preparation_steps={work} retained={bytes}"
        );
        if axis == "subjects" {
            if let Some((base_work, base_bytes)) = baseline {
                if noise == 128 {
                    assert_eq!((work, bytes), (base_work, base_bytes));
                } else {
                    assert!(work <= base_work * (count / 8) * 2);
                    assert!(bytes <= base_bytes * (count / 8) * 2);
                }
            } else {
                baseline = Some((work, bytes));
            }
        }
        verify(&mut facts, template, contract)?;
    }
    assert_eq!(size_of::<Formed<'_>>(), size_of::<usize>());
    assert_eq!(size_of::<State>(), 96);
    assert_eq!(size_of::<crate::body::regions::BuiltinRequirements>(), 56);
    assert_eq!(size_of::<Body>(), 448);
    assert_eq!(size_of::<Contract>(), 184);
    Ok(())
}

fn opened_subject_pairs(
    facts: &mut Declarations,
    template: &Template,
    contract: &Contract,
    members: usize,
) -> Result {
    let State::Prepared(prepared) = state(contract)? else {
        return Err("prepared defining claims".into());
    };
    let subjects = prepared.claims.projected_subjects().collect::<Vec<_>>();
    let mut declared = Vec::new();
    let mut opened = crate::support::hash::HashSet::default();
    for ty in subjects {
        if facts.types.escaping_binders(ty).ok_or("projection scope")? == 0 {
            assert!(opened.insert(ty));
        } else {
            declared.push(ty);
        }
    }
    assert_eq!((declared.len(), opened.len()), (members, members));
    let mut fold = crate::types::substitute::Bound::new(original_aliases(facts, template)?);
    for subject in declared {
        assert!(opened.remove(&fold.apply(&mut facts.types, subject)?));
    }
    assert!(opened.is_empty());
    Ok(())
}

#[test]
fn actual_body_opening_keeps_source_width_sites_noise_and_retained_work_bounded() -> Result {
    let mut baseline = None;
    for (axis, count, noise) in [
        ("subjects", 8, 0),
        ("subjects", 32, 0),
        ("subjects", 128, 0),
        ("subjects", 8, 128),
        ("sites", 8, 0),
        ("sites", 32, 0),
        ("sites", 128, 0),
    ] {
        let path = format!(
            "pa31/tests/spec/trait-default-effects/projections/body-opening/opening-{axis}-{count}-noise-{noise}.rs"
        );
        let (_report, mut facts, template, mut contract) = fixture(&path)?;
        let aliases = template
            .environment
            .body_aliases()
            .ok_or("actual producer alias slots")?;
        assert_eq!(aliases.len(), 1);
        let before = facts.body_work.canonical_steps;
        crate::body::copy_requirements::prepare(&mut facts, &template.body, &mut contract)?;
        let work = facts.body_work.canonical_steps - before;
        let builtins = contract.builtins.as_deref().ok_or("body inputs")?;
        let goals = &builtins
            .source_copy
            .as_ref()
            .ok_or("actual Copy reads")?
            .goals;
        let members = if axis == "subjects" { count } else { 1 };
        assert_eq!(goals.len(), members);
        for goal in goals {
            assert_eq!(
                builtins.copy_decision(&template.body, goal.ty)?,
                Some(false)
            );
        }
        let bytes = state(&contract)?.retained_bytes();
        println!(
            "actual_opening_axis={axis} count={count} noise={noise} copy_queries={members} slots=1 steps={work} retained={bytes}"
        );
        if axis == "subjects" {
            if let Some((base_work, base_bytes)) = baseline {
                if noise == 128 {
                    assert_eq!((work, bytes), (base_work, base_bytes));
                } else {
                    assert!(work <= base_work * (count / 8) * 2);
                    assert!(bytes <= base_bytes * (count / 8) * 2);
                }
            } else {
                baseline = Some((work, bytes));
            }
        }
        opened_subject_pairs(&mut facts, &template, &contract, members)?;
        verify(&mut facts, template, contract)?;
    }
    Ok(())
}
