//! Source alias identities, body maps and the actual payer remain separate.
use super::*;
use crate::{
    resolve::{expansion::Report, model::DefKind},
    types::{Primitive, arguments::Argument},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa34/tests/spec/named-opaques";
fn report(name: &str) -> TestResult<Report> {
    let text = crate::test_source::fixture(&format!("{PACKET}/{name}.rs"))?;
    let mut config = crate::expand::config::Config::default();
    config.set_lint_cap(crate::support::diagnostics::LintCap::Allow);
    Ok(crate::test_source::source_provider_config(
        &text,
        "named_owner_controls",
        &config,
    )?)
}
fn analyze(name: &str) -> TestResult<(Report, Declarations)> {
    let report = report(name)?;
    let facts = crate::pipeline::analyze_source(&report)?.facts;
    Ok((report, facts))
}
#[test]
fn alias_declaration_and_each_authorized_body_retain_distinct_owners() -> TestResult {
    let (report, facts) = analyze("consistent-defining-bodies")?;
    let named = facts.opaques.named.as_deref().ok_or("named rows")?;
    assert_eq!(named.aliases.len(), 1);
    assert_eq!(named.applications.len(), 2);
    let (&alias, ids) = named.aliases.iter().next().ok_or("alias")?;
    assert_eq!(ids.len(), 1);
    let id = ids[0];
    let declaration = facts.opaques.declarations.get(&id).ok_or("opaque")?;
    assert_eq!(declaration.owner, alias);
    assert_eq!(
        report.resolution.definitions[alias.0 as usize].kind,
        DefKind::TypeAlias
    );
    assert_eq!(declaration.hidden, Some(TypeId::primitive(Primitive::U8)));
    for (&(function, actual_id), &application) in &named.applications {
        assert_ne!(function, alias);
        assert_eq!(actual_id, id);
        assert!(facts.opaques.owners[&function].contains(&id));
        assert_eq!(
            report.resolution.definitions[function.0 as usize].kind,
            DefKind::Function
        );
        assert_eq!(
            facts.types.kind(application),
            facts.types.kind(declaration.application)
        );
        assert!(facts.typed_function_bodies.contains(&function));
    }
    assert!(named.pending.is_empty() && named.previous.is_empty());
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::resolve::model::Resolution>(), 2032);
    assert_eq!(size_of::<crate::sema::model::NodeFacts>(), 80);
    assert_eq!(size_of::<crate::sema::opaque::Opaques>(), 152);
    Ok(())
}
#[test]
fn nested_components_reverse_generic_and_lifetime_coordinates() -> TestResult {
    for (name, count) in [
        ("generic-remapping", 1),
        ("lifetime-remapping", 1),
        ("nested-splitpaths-shaped", 2),
    ] {
        let (report, mut facts) = analyze(name)?;
        let named = facts.opaques.named.as_deref().ok_or("named rows")?;
        assert_eq!(named.aliases.values().map(Vec::len).sum::<usize>(), count);
        let mappings: Vec<_> = named
            .applications
            .iter()
            .map(|(&(owner, id), &actual)| (owner, id, actual))
            .collect();
        for (owner, id, actual) in mappings {
            let declaration = facts.opaques.declarations.get(&id).ok_or("declaration")?;
            let hidden = declaration.hidden.ok_or("closed hidden")?;
            assert!(declaration.free_regions.is_empty());
            assert_ne!(owner, declaration.owner);
            let Some(Kind::Opaque(_, formal)) = facts.types.kind(declaration.application) else {
                return Err("formal application".into());
            };
            let Some(Kind::Opaque(_, arguments)) = facts.types.kind(actual) else {
                return Err("body application".into());
            };
            let formal: Vec<_> = facts
                .types
                .arguments(formal)
                .ok_or("formal captures")?
                .collect();
            let actual: Vec<_> = facts
                .types
                .arguments(arguments)
                .ok_or("body captures")?
                .collect();
            assert_eq!(formal.len(), actual.len());
            for (&formal, &actual) in formal.iter().zip(&actual) {
                if let (Argument::Type(formal), Argument::Type(actual)) = (formal, actual) {
                    assert_ne!(formal, actual);
                }
            }
            let mut opening = facts
                .named_opaque_opening(&report, owner, id)?
                .ok_or("opening")?;
            let opened = opening.apply(&mut facts.types, hidden)?;
            assert_eq!(
                facts.close_named_hidden(&report, owner, id, opened)?,
                hidden
            );
        }
    }
    Ok(())
}
#[test]
fn foreign_source_arena_cannot_rebind_equal_numeric_aliases() -> TestResult {
    let first = report("simple")?;
    let foreign = report("simple")?;
    let mut facts = crate::sema::declarations::analyze(&first)?;
    let named = facts.opaques.named.as_deref().ok_or("named rows")?;
    assert!(named.applications.is_empty());
    let error = facts
        .prepare_named_opaques(&foreign)
        .err()
        .ok_or("foreign arena accepted")?;
    assert!(
        error.to_string().contains("different source arena"),
        "{error}"
    );
    let named = facts.opaques.named.as_deref().ok_or("named rows")?;
    assert!(named.applications.is_empty() && !named.prepared);
    facts.prepare_named_opaques(&first)?;
    Ok(())
}
#[test]
fn exhausted_named_payment_cannot_mutate_alias_tables() -> TestResult {
    let report = report("simple")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let named = facts.opaques.named.as_deref().ok_or("named rows")?;
    let before = (named.aliases.len(), named.aliases.capacity());
    let (&alias, ids) = named.aliases.iter().next().ok_or("alias")?;
    let id = ids[0];
    assert!(report.package.pay_named_opaque(0, usize::MAX).is_err());
    assert!(facts.register_named_opaque(&report, alias, id).is_err());
    let named = facts.opaques.named.as_deref().ok_or("named rows")?;
    assert_eq!((named.aliases.len(), named.aliases.capacity()), before);
    assert_eq!(named.aliases[&alias], vec![id]);
    assert!(facts.register_named_opaque(&report, alias, id).is_err());
    Ok(())
}
#[test]
fn repeated_alias_definitions_pay_linear_work_and_leave_warm_maps_unchanged() -> TestResult {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for count in [8_usize, 32, 128] {
        let mut source = crate::test_source::fixture(&format!("{PACKET}/foundation.rs"))?;
        for index in 0..count {
            writeln!(
                source,
                "pub type Alias{index}=impl Value;#[define_opaque(Alias{index})]pub fn define{index}()->Alias{index}{{7u8}}"
            )?;
        }
        let report = crate::test_source::source_provider(&source, "named_scaling")?;
        let mut facts = crate::sema::declarations::analyze(&report)?;
        let before = report.package.named_opaque_usage()?;
        facts.prepare_named_opaques(&report)?;
        let after = report.package.named_opaque_usage()?;
        assert!(after.0 > before.0 && after.1 > before.1);
        assert_eq!(
            facts
                .opaques
                .named
                .as_deref()
                .ok_or("named rows")?
                .applications
                .len(),
            count
        );
        for _ in 0..128 {
            facts.prepare_named_opaques(&report)?;
        }
        assert_eq!(report.package.named_opaque_usage()?, after);
        observations.push((after.0 - before.0, after.1 - before.1));
    }
    for pair in observations.windows(2) {
        assert!(pair[1].0 <= pair[0].0 * 6 + 4096, "{observations:?}");
        assert!(pair[1].1 <= pair[0].1 * 6 + 4096, "{observations:?}");
    }
    Ok(())
}

#[test]
fn another_opaque_identity_cannot_replace_an_owned_application() -> TestResult {
    let (report, mut facts) = analyze("simple")?;
    let (&(owner, id), _) = facts
        .opaques
        .named
        .as_deref()
        .ok_or("named rows")?
        .applications
        .iter()
        .next()
        .ok_or("application")?;
    let contract = facts
        .types
        .opaque_declaration(id)
        .ok_or("opaque contract")?;
    let foreign = facts.types.declare_opaque(contract)?;
    let application = facts
        .types
        .intern(Kind::Opaque(foreign, crate::types::GenericArgs::EMPTY))?;
    facts
        .opaques
        .named
        .as_deref_mut()
        .ok_or("named rows")?
        .applications
        .insert((owner, id), application);
    assert!(facts.require_named_application(owner, id).is_err());
    assert!(facts.named_opaque_opening(&report, owner, id).is_err());
    assert!(
        facts
            .close_named_hidden(&report, owner, id, TypeId::primitive(Primitive::U8))
            .is_err()
    );
    assert!(facts.close_opaque_results(&report, owner, &[]).is_err());
    assert_eq!(
        facts.opaques.declarations[&id].hidden,
        Some(TypeId::primitive(Primitive::U8))
    );
    Ok(())
}
