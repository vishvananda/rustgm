//! Actual scoped declaration ownership and denied publication authority.
use super::*;

fn first_scoped(package: &Package) -> TestResult<(NodeId, Item, NodeId)> {
    let unit = &package.units[0];
    for owner in &unit.configured.owners {
        let mut node = owner.node;
        while let Kind::Attributed { value, .. } = unit.configured.ast.node(node).kind {
            node = value;
        }
        if let Kind::Item(index) = unit.configured.ast.node(node).kind {
            let item = unit.configured.ast.items[index as usize];
            if matches!(item.kind, ItemKind::DeclarativeMacro { .. }) {
                return Ok((node, item, owner.node));
            }
        }
    }
    Err("missing real attributed scoped item".into())
}

#[test]
fn failed_actual_owner_feature_and_matcher_checks_publish_no_declaration() -> TestResult {
    for (prefix, attribute, declaration, wrong_owner) in [
        (
            "#![feature(rustc_attrs)]",
            "#[rustc_macro_transparency=\"opaque\"]",
            "pub macro one(){7u32}",
            false,
        ),
        (
            "#![feature(decl_macro,rustc_attrs)]",
            "#[rustc_macro_transparency=\"opaque\"]",
            "pub macro one(){7u32}",
            true,
        ),
        (
            "#![feature(decl_macro,rustc_attrs)]",
            "#[rustc_macro_transparency=\"opaque\"]",
            "pub macro one{()=>{7u32};()=>{8u32}}",
            false,
        ),
    ] {
        let mut package = loaded(&format!(
            "{prefix}{attribute}{declaration}{attribute}pub macro two(){{7u32}}"
        ))?;
        let (node, item, owner) = first_scoped(&package)?;
        let other = package.units[0]
            .configured
            .owners
            .iter()
            .find_map(|candidate| {
                if candidate.node == owner {
                    return None;
                }
                let mut node = candidate.node;
                while let Kind::Attributed { value, .. } =
                    package.units[0].configured.ast.node(node).kind
                {
                    node = value;
                }
                let Kind::Item(index) = package.units[0].configured.ast.node(node).kind else {
                    return None;
                };
                matches!(
                    package.units[0].configured.ast.items[index as usize].kind,
                    ItemKind::DeclarativeMacro { .. }
                )
                .then_some(candidate.node)
            })
            .ok_or("second actual configured scoped owner")?;
        let mut state = crate::resolve::index::State::default();
        let mut resolver = state.bind(&mut package);
        let scope = resolver.new_scope(None, ModuleId(0), true)?;
        let context = resolver.context(ModuleId(0), scope);
        let probe = Probe::start()?;
        let error = resolver
            .declare_scoped(
                context,
                node,
                item,
                Access::Public,
                Some(if wrong_owner { other } else { owner }),
            )
            .err()
            .ok_or("invalid scoped input published")?;
        let trace = probe.finish()?;
        assert!(resolver.facts.definitions.is_empty());
        assert!(!resolver.facts.item_definitions.contains_key(&(
            ModuleId(0),
            node,
            Namespace::Macro
        )));
        assert!(
            !trace
                .events
                .iter()
                .any(|event| matches!(event, Event::Published { .. }))
        );
        if wrong_owner {
            assert!(error.message.contains("another item"), "{error}");
        }
    }
    Ok(())
}

#[test]
fn duplicates_never_replace_an_existing_actual_macro_binding() -> TestResult {
    let mut package =
        loaded("#![feature(decl_macro)]pub macro same(){7u32}pub macro same(){8u32}")?;
    let probe = Probe::start()?;
    let error = resolve::analyze(&mut package)
        .err()
        .ok_or("duplicate scoped items accepted")?;
    let trace = probe.finish()?;
    assert!(error.message.contains("duplicate"), "{error}");
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|event| matches!(event, Event::Compile { .. }))
            .count(),
        2
    );
    assert_eq!(
        trace
            .events
            .iter()
            .filter(|event| matches!(event, Event::Published { .. }))
            .count(),
        1
    );
    Ok(())
}

#[test]
fn scoped_markers_have_no_legacy_textual_binding_or_external_metadata_permission() -> TestResult {
    use crate::expand::textual::{Lookup, Scope};
    let mut report = crate::test_source::resolve(
        "#![feature(decl_macro)]pub macro unused(){unresolved()}pub fn answer()->u32{7u32}",
        true,
    )?;
    let marker = report
        .definitions
        .iter()
        .position(|definition| {
            matches!(
                definition.body,
                crate::resolve::expansion::Body::ScopedDeclaration(_)
            )
        })
        .ok_or("actual scoped marker")?;
    let definition = &report.definitions[marker];
    assert_eq!(
        report
            .textual_scopes
            .lookup(Scope::ROOT, definition.name)
            .map_err(|error| error.message)?,
        Lookup::Missing
    );
    let key = definition.node.ok_or("actual source macro")?;
    assert!(
        report
            .resolution
            .definitions
            .iter()
            .any(|entry| entry.node == Some(key) && entry.kind == DefKind::Macro)
    );
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let identity = crate::support::crates::CrateIdentity::new("scope_export", "opponent")?;
    let error = crate::sema::crates::export(&report, &mut facts, identity)
        .err()
        .ok_or("unsupported macro metadata exported")?;
    assert!(error.message.contains("metadata profile"), "{error}");
    Ok(())
}

#[test]
fn forged_report_frames_cannot_borrow_an_unexecuted_scoped_allowance() -> TestResult {
    use crate::resolve::{bindings::KnownFeature, expansion::Body};
    let source =
        "#![feature(no_core,lang_items,decl_macro,rustc_attrs,allow_internal_unstable)]#![no_core]
#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}
#[lang=\"sized\"]trait Sized:MetaSized{}
#[allow_internal_unstable(panic_internals)]macro unused(){probe()}
macro_rules!live{()=>{probe()};}fn probe()->u32{7u32}fn answer()->u32{live!()}";
    let mut report = crate::test_source::source_provider(source, "unavailable_frame")?;
    let scoped = report
        .definitions
        .iter()
        .position(|definition| matches!(definition.body, Body::ScopedDeclaration(_)))
        .ok_or("actual unavailable scoped definition")?;
    let invocation = report
        .invocations
        .first()
        .ok_or("actual legacy invocation")?;
    let live = invocation.definition;
    let file = invocation.node.file;
    let unit = &report.package.units[file.0 as usize];
    let site = unit
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(index, node)| {
            if let Kind::Name(span, _) = node.kind
                && unit.configured.active[index]
                && unit
                    .source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    == Some("probe")
            {
                Some(crate::expand::stream::Location { file, span })
            } else {
                None
            }
        })
        .ok_or("actual expanded probe token")?;
    assert!(
        !report
            .macro_allowances()
            .allows(site, &KnownFeature::PanicInternals.name())?
    );
    report.invocations[0].definition = scoped;
    let error = report
        .macro_allowances()
        .allows(site, &KnownFeature::PanicInternals.name())
        .err()
        .ok_or("unexecuted scoped marker supplied provenance")?;
    assert!(error.message.contains("no executed expansion frame"));
    let compiled = std::mem::replace(
        &mut report.definitions[live].body,
        Body::Builtin(crate::expand::builtin::Builtin::Line),
    );
    report.definitions[scoped].body = compiled;
    let error = report
        .macro_allowances()
        .allows(site, &KnownFeature::PanicInternals.name())
        .err()
        .ok_or("substituted body hid the actual scoped declaration")?;
    assert!(error.message.contains("no executed expansion frame"));
    Ok(())
}
