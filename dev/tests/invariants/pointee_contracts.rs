//! Real source declaration ownership is distinct from normalization and transport.
use super::{Catalog, Contracts, LangItem, require_transport};
use crate::{
    resolve::{expansion::Report, model::DefKind},
    sema::{declarations, header_attributes, model::Declarations},
    types::{ItemId, Kind, projection},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn source(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/pointee-declarations/{name}.rs"
    ))?)
}
fn provider(name: &str) -> TestResult<Report> {
    Ok(crate::test_source::source_provider(
        &source(name)?,
        "pointee_declarations",
    )?)
}
fn actual_application(facts: &Declarations) -> TestResult<projection::Application> {
    let member = facts
        .source_bindings
        .as_deref()
        .ok_or("actual catalog")?
        .validated_language(LangItem::MetadataType)?
        .ok_or("actual member")?;
    let ty = facts
        .associated
        .require_projection_header(member.definition)?
        .projection;
    let Some(Kind::Projection(application)) = facts.types.kind(ty) else {
        return Err("actual Metadata projection".into());
    };
    Ok(application)
}
fn check(
    report: &Report,
    facts: &Declarations,
    application: projection::Application,
) -> TestResult {
    let token = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .pointee_member(report, facts, application)?;
    assert_eq!(token.current(report, facts)?, application);
    Ok(())
}
#[test]
fn actual_member_nominal_and_result_obligations_remain_bound_to_the_provider() -> TestResult {
    for name in [
        "provider",
        "no-dyn-role",
        "result-obligations",
        "policy-provider",
    ] {
        let report = provider(name)?;
        let facts = declarations::analyze(&report)?;
        let application = actual_application(&facts)?;
        check(&report, &facts, application)?;
        let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
        let parent = catalog
            .validated_language(LangItem::PointeeTrait)?
            .ok_or("Pointee")?;
        let member = catalog
            .validated_language(LangItem::MetadataType)?
            .ok_or("Metadata")?;
        assert_eq!(member.parent, Some(parent.definition));
        assert_eq!(
            catalog.forces_dyn_incompatible(parent.definition)?,
            name == "policy-provider"
        );
        assert_eq!(
            report
                .resolution
                .source_bindings
                .as_deref()
                .ok_or("reservations")?
                .denies_implementation(parent.definition),
            name == "policy-provider"
        );

        assert_eq!(
            facts.associated.trait_owner(member.definition),
            Some(parent.definition)
        );
        assert_eq!(
            facts
                .types
                .projection_schema(application.item)
                .ok_or("schema")?
                .trait_item,
            ItemId(parent.definition.0)
        );
        assert_eq!(
            catalog.validated_language(LangItem::DynMetadata)?.is_some(),
            name != "no-dyn-role"
        );
        for declaration in [Some(member), catalog.language(LangItem::DynMetadata)]
            .into_iter()
            .flatten()
        {
            let view = facts.header_attribute_contracts(&report, declaration.definition)?;
            assert!(matches!(
                view.must_use,
                header_attributes::ContractKnowledge::Unknown
            ));
            assert!(matches!(
                view.inline,
                header_attributes::ContractKnowledge::Unknown
            ));
        }
        let wrong = projection::Application {
            item: ItemId(parent.definition.0),
            ..application
        };
        assert!(catalog.pointee_member(&report, &facts, wrong).is_err());
        if name == "result-obligations" {
            let actual = report
                .resolution
                .definitions
                .iter()
                .enumerate()
                .find(|(_, row)| {
                    row.kind == DefKind::Trait && report.package.names.get(row.name) == "Actual"
                })
                .ok_or("Actual")?;
            assert!(
                facts
                    .associated
                    .require_projection_header(member.definition)?
                    .bounds
                    .iter()
                    .any(|clause| {
                        let crate::types::parameters::clauses::Clause::Predicate(predicate) =
                            clause
                        else {
                            return false;
                        };
                        predicate.bound.trait_parts().is_some_and(|(id, _)| {
                            facts
                                .types
                                .trait_application(id)
                                .is_some_and(|a| a.item.0 as usize == actual.0)
                        })
                    })
            );
        }
    }
    Ok(())
}
#[test]
fn fresh_images_arenas_catalogs_and_changed_owners_cannot_reuse_the_member() -> TestResult {
    let report = provider("provider")?;
    let facts = declarations::analyze(&report)?;
    let application = actual_application(&facts)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let token = catalog.pointee_member(&report, &facts, application)?;
    let other = provider("provider")?;
    assert!(token.current(&other, &facts).is_err());
    let mut other_facts = declarations::analyze(&other)?;
    assert!(token.current(&other, &other_facts).is_err());
    assert!(
        catalog
            .pointee_member(&other, &other_facts, application)
            .is_err()
    );
    let other_token = other_facts
        .source_bindings
        .as_deref()
        .ok_or("other catalog")?
        .pointee_member(&other, &other_facts, actual_application(&other_facts)?)?;
    let store = std::mem::take(&mut other_facts.types);
    assert!(other_token.current(&other, &other_facts).is_err());
    other_facts.types = store;
    other_token.current(&other, &other_facts)?;
    let mut changed = provider("provider")?;
    let changed_facts = declarations::analyze(&changed)?;
    let changed_application = actual_application(&changed_facts)?;
    let member = changed_facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .language(LangItem::MetadataType)
        .ok_or("Metadata")?;
    let changed_token = changed_facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .pointee_member(&changed, &changed_facts, changed_application)?;
    changed.resolution.definitions[member.definition.0 as usize].scope =
        changed.resolution.module_scopes[0];
    assert!(changed_token.current(&changed, &changed_facts).is_err());
    let mut physical = provider("provider")?;
    let physical_facts = declarations::analyze(&physical)?;
    let physical_token = physical_facts
        .source_bindings
        .as_deref()
        .ok_or("physical catalog")?
        .pointee_member(
            &physical,
            &physical_facts,
            actual_application(&physical_facts)?,
        )?;
    let copy = physical.package.units[0].source.physical_copy();
    assert_eq!(copy.original(), physical.package.units[0].source.original());
    physical.package.units[0].source = copy;
    assert!(physical_token.current(&physical, &physical_facts).is_err());
    configured_owner_changes_are_rejected()?;
    Ok(())
}
fn configured_owner_changes_are_rejected() -> TestResult {
    let mut report = provider("provider")?;
    let facts = declarations::analyze(&report)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    let member = catalog.language(LangItem::MetadataType).ok_or("Metadata")?;
    let token = catalog.pointee_member(&report, &facts, actual_application(&facts)?)?;
    let unit = &report.package.units[member.origin.file.0 as usize];
    let owner = unit
        .configured
        .owners
        .iter()
        .position(|owner| {
            let mut node = owner.node;
            while let crate::syntax::ast::Kind::Attributed { value, .. } =
                unit.configured.ast.node(node).kind
            {
                node = value;
            }
            node == member.origin.node
        })
        .ok_or("actual configured Metadata owner")?;
    report.package.units[member.origin.file.0 as usize]
        .configured
        .owners[owner]
        .enabled = false;
    assert!(token.current(&report, &facts).is_err());
    report.package.units[member.origin.file.0 as usize]
        .configured
        .owners[owner]
        .enabled = true;
    report.package.units[member.origin.file.0 as usize]
        .configured
        .owners[owner]
        .attributes
        .start += 1;
    assert!(token.current(&report, &facts).is_err());
    Ok(())
}
#[test]
fn actual_kinds_features_and_active_owners_are_checked_without_spelling_fallback() -> TestResult {
    for name in [
        "wrong-trait-kind",
        "wrong-member-kind",
        "wrong-nominal-kind",
        "missing-feature",
    ] {
        let text = source(name)?;
        let result =
            crate::test_source::source_provider(&text, "pointee_declarations").and_then(|report| {
                declarations::analyze(&report)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            });
        let error = result
            .err()
            .ok_or_else(|| format!("{name} unexpectedly accepted"))?;
        let expected = match name {
            "wrong-nominal-kind" => "source binding requires an indexed declaration owner",
            "missing-feature" => "lang requires its explicit source feature",
            _ => "source Pointee role has the wrong declaration kind or owner",
        };
        assert!(error.contains(expected), "{name}: {error}");
    }
    let wrong = provider("wrong-associated-owner")?;
    let error = declarations::analyze(&wrong)
        .err()
        .ok_or("actual owner mismatch must fail")?;
    assert_eq!(
        error.message,
        "source Metadata role ownership is outside the supported Pointee declaration profile"
    );
    let prohibited = provider("explicit-policy-impl")?;
    let prohibited_error = declarations::analyze(&prohibited)
        .err()
        .ok_or("explicit Pointee policy implementation must fail")?;
    assert!(
        prohibited_error
            .message
            .contains("explicit implementation is prohibited by its source trait declaration"),
        "{prohibited_error}"
    );
    let implicit = provider("implicit-protocol-impl")?;
    let implicit_error = declarations::analyze(&implicit)
        .err()
        .ok_or("actual builtin selection remains closed")?;
    assert_eq!(
        implicit_error.message,
        "explicit source Pointee implementation selection is not implemented"
    );
    let inactive = provider("inactive-claims")?;
    let facts = declarations::analyze(&inactive)?;
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    for role in [
        LangItem::PointeeTrait,
        LangItem::MetadataType,
        LangItem::DynMetadata,
    ] {
        assert!(catalog.language(role).is_none());
    }
    assert!(catalog.traits.is_none());
    require_transport(&inactive, &facts)?;
    Ok(())
}
#[test]
fn transport_stays_closed_and_ordinary_storage_and_enclosing_layouts_are_observed() -> TestResult {
    let report = provider("provider")?;
    let facts = declarations::analyze(&report)?;
    let application = actual_application(&facts)?;
    let before =
        header_attributes::source_contract_storage(&facts).ok_or("existing header budget")?;
    let owner = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .traits
        .as_deref()
        .ok_or("validated")?
        .pointee
        .as_ref()
        .ok_or("Pointee owner")?;
    let capacities = owner.capacities();
    assert_eq!(std::rc::Rc::strong_count(owner), 1);
    for _ in 0..32 {
        check(&report, &facts, application)?;
    }
    let after =
        header_attributes::source_contract_storage(&facts).ok_or("existing header budget")?;
    assert_eq!(owner.capacities(), capacities);
    assert_eq!(std::rc::Rc::strong_count(owner), 1);
    assert!(after.0 > before.0);
    assert_eq!(
        (before.1, before.2, before.3, before.4),
        (after.1, after.2, after.3, after.4)
    );
    let error = require_transport(&report, &facts)
        .err()
        .ok_or("unsupported transport")?;
    assert_eq!(
        error.message,
        "source Pointee contract transport is not implemented"
    );
    let ordinary = crate::test_source::resolve("fn ordinary(){}", true)?;
    let ordinary_facts = declarations::analyze(&ordinary)?;
    assert!(ordinary_facts.source_bindings.is_none());
    assert!(header_attributes::source_contract_storage(&ordinary_facts).is_none());
    assert!(
        header_attributes::charge_source_contract_query(
            &facts,
            crate::support::crates::MAX_COMPONENTS
        )
        .is_err()
    );
    let failed = header_attributes::source_contract_storage(&facts).ok_or("failed work")?;
    assert!(check(&report, &facts, application).is_err());
    let retried = header_attributes::source_contract_storage(&facts).ok_or("retry work")?;
    assert!(retried.0 > failed.0);
    assert_eq!(retried.1, failed.1);
    println!(
        "Catalog={} Reservations={} Validated={} Contracts-State={:?} CheckedMember={} SourceImage={} Package={} Declarations={} Unit={} Node={} Scope={} Definition={} source-budget-before={before:?} after={after:?} contract-vec-capacities={capacities:?}",
        size_of::<Catalog>(),
        size_of::<crate::resolve::bindings::Reservations>(),
        size_of::<super::super::roles::Validated>(),
        Contracts::layouts(),
        size_of::<super::CheckedMember>(),
        size_of::<header_attributes::SourceImage>(),
        size_of::<crate::resolve::package::Package>(),
        size_of::<Declarations>(),
        size_of::<crate::resolve::unit::Unit>(),
        size_of::<crate::syntax::ast::Node>(),
        size_of::<crate::resolve::model::Scope>(),
        size_of::<crate::resolve::model::Definition>()
    );
    Ok(())
}
