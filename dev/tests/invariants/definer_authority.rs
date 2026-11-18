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

fn layout_source(suffix: &str) -> TestResult<Report> {
    let mut source = crate::test_source::fixture(&format!("{PACKET}/foundation.rs"))?;
    source.push_str(suffix);
    Ok(crate::test_source::source_provider(
        &source,
        "named_layout_controls",
    )?)
}

#[test]
fn named_layout_waits_for_its_defining_original_before_computing_nested_union_size() -> TestResult {
    let report = layout_source(
        r#"
        #[lang = "maybe_dangling"]
        #[repr(transparent)]
        pub struct MaybeDangling<T: ?Sized>(T);
        #[lang = "manually_drop"]
        #[repr(transparent)]
        pub struct ManuallyDrop<T: ?Sized> { value: MaybeDangling<T> }
        #[lang = "bikeshed_guaranteed_no_drop"]
        #[rustc_deny_explicit_impl]
        #[rustc_dyn_incompatible_trait]
        pub trait GuaranteedNoDrop {}
        pub type Lazy = impl Value;
        #[define_opaque(Lazy)]
        pub fn define() -> Lazy { 7u8 }
        union Slot<F> { original: u8, deferred: ManuallyDrop<F> }
        struct Layer<F> { data: Slot<F> }
        enum Outer { Empty, Present(Layer<Lazy>) }
        pub struct PendingStorage { inner: Outer }
        "#,
    )?;
    let mut declarations = crate::sema::declarations::analyze(&report)?;
    let root = declarations
        .opaques
        .named
        .as_deref()
        .and_then(|named| named.pending_layouts.last_root())
        .ok_or("missing pending outer nominal")?;
    assert!(declarations.layouts.get(root).is_none());
    let strict = crate::abi::templates::Templates::default()
        .declaration(
            &mut declarations.types,
            &mut declarations.aggregates,
            &mut declarations.layouts,
            root,
        )
        .err()
        .ok_or("strict declaration proof accepted an unpublished hidden head")?;
    assert_eq!(
        strict.message,
        "opaque storage requires a checked defining type"
    );

    let facts = crate::pipeline::analyze(&report, declarations)?.facts;
    assert_eq!(
        facts
            .layouts
            .get(root)
            .ok_or("missing resumed layout")?
            .size,
        2
    );
    assert!(
        facts
            .opaques
            .named
            .as_deref()
            .is_some_and(|named| named.pending_layouts.last_root().is_none())
    );
    Ok(())
}

#[test]
fn named_layout_retries_transparent_side_fields_from_checked_hidden_size() -> TestResult {
    for (value, expected) in [
        ("Marker", None),
        (
            "7u8",
            Some("transparent aggregate has more than one nontrivial field"),
        ),
    ] {
        let report = layout_source(&format!(
            "pub type Lazy = impl Value; #[define_opaque(Lazy)] pub fn define() -> Lazy {{ {value} }}
             #[repr(transparent)] pub struct Wrapped(Lazy, u8);"
        ))?;
        let facts = crate::sema::declarations::analyze(&report)?;
        let result = crate::pipeline::analyze(&report, facts);
        match (expected, result) {
            (None, Ok(_)) => {}
            (Some(message), Err(error)) => assert_eq!(error.message, message),
            (None, Err(error)) => return Err(error.into()),
            (Some(_), Ok(_)) => return Err("accepted two nontrivial transparent fields".into()),
        }
    }
    Ok(())
}

#[test]
fn named_layout_cannot_waive_union_drop_or_reclassify_a_failed_defining_body() -> TestResult {
    let report = layout_source(
        "pub type Lazy = impl Value; #[define_opaque(Lazy)] pub fn define() -> Lazy { 7u8 }
         pub union Wrong { value: Lazy }",
    )?;
    let error = crate::sema::declarations::analyze(&report)
        .err()
        .ok_or("accepted union field without a drop proof")?;
    assert_eq!(
        error.message,
        "source union field requires its actual NoDrop or Copy proof"
    );

    let report = layout_source(
        "pub type Lazy = impl Value; #[define_opaque(Lazy)] pub fn define() -> Lazy { () }
         pub struct Waiting(Lazy);",
    )?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let owner = report
        .named_opaque_definitions()
        .and_then(|claims| claims.keys().next())
        .and_then(|owner| report.resolution.definitions[owner.0 as usize].node)
        .ok_or("missing defining body owner")?;
    let span = report
        .package
        .unit(owner.file)
        .configured
        .ast
        .node(owner.node)
        .span;
    let error = crate::pipeline::analyze(&report, facts)
        .err()
        .ok_or("accepted invalid defining body")?;
    assert_eq!(error.file.source(), Some(owner.file));
    assert!((span.start..span.end).contains(&error.span.start));
    Ok(())
}
#[test]
fn named_layout_closes_both_siblings_and_rejects_two_already_nontrivial_fields() -> TestResult {
    let source = "pub type First = impl Value; pub type Second = impl Value;
        #[define_opaque(First)] pub fn first() -> First { Marker }
        #[define_opaque(Second)] pub fn second() -> Second { 7u8 }";
    let report = layout_source(&format!("{source} pub struct Both(First, Second);"))?;
    let declarations = crate::sema::declarations::analyze(&report)?;
    let root = declarations
        .opaques
        .named
        .as_deref()
        .and_then(|named| named.pending_layouts.last_root())
        .ok_or("missing two-dependency layout root")?;
    let facts = crate::pipeline::analyze(&report, declarations)?.facts;
    assert_eq!(
        facts
            .layouts
            .get(root)
            .ok_or("missing resumed layout")?
            .size,
        1
    );
    for id in facts.opaques.declarations.values() {
        assert!(facts.types.storage_head(id.application).is_ok());
    }
    let report = layout_source(&format!(
        "{source} #[repr(transparent)] pub struct Wrong(First, u8, u8);"
    ))?;
    let error = crate::sema::declarations::analyze(&report)
        .err()
        .ok_or("a pending field delayed an already-invalid representation")?;
    assert_eq!(
        error.message,
        "transparent aggregate has more than one nontrivial field"
    );
    Ok(())
}

#[test]
fn named_layout_closes_a_captured_backtrace_resolver() -> TestResult {
    let mut source = crate::test_source::fixture(&format!("{PACKET}/backtrace-shaped.rs"))?;
    source.push_str("\npub struct BacktraceLayout { inner: LazyResolve }\n");
    let report = crate::test_source::source_provider(&source, "backtrace_closure_layout")?;
    let declarations = crate::sema::declarations::analyze(&report)?;
    let root = declarations
        .opaques
        .named
        .as_deref()
        .and_then(|named| named.pending_layouts.last_root())
        .ok_or("missing pending Backtrace representation")?;
    assert!(declarations.layouts.get(root).is_none());
    let opaque = declarations
        .opaques
        .declarations
        .values()
        .next()
        .ok_or("missing named resolver declaration")?
        .application;
    assert!(declarations.types.storage_head(opaque).is_err());

    let facts = crate::pipeline::analyze(&report, declarations)?.facts;
    let hidden = facts.types.storage_head(opaque)?;
    assert!(matches!(
        facts.types.kind(hidden),
        Some(crate::types::Kind::Closure { .. })
    ));
    let layout = facts
        .layouts
        .get(root)
        .ok_or("missing checked Backtrace layout")?;
    assert_eq!(layout.size, 1);
    Ok(())
}
