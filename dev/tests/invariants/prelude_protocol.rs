//! Real source prelude scopes stay separate from lexical and bootstrap bindings.
use super::*;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String, Box<dyn std::error::Error>> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-preludes/{name}.rs"
    ))?)
}
#[test]
fn real_constructor_method_and_shadow_paths_reach_checked_local_lowering() -> TestResult {
    for name in [
        "constructor-method",
        "no-implicit-explicit",
        "local-shadow",
        "glob-shadow",
        "opaque-names",
    ] {
        let source = fixture(name)?;
        let report = crate::test_source::resolve(&source, true)?;
        let scope = report
            .resolution
            .source_prelude_scope(&report.package)?
            .ok_or("actual source module")?;
        let root = report.resolution.module_scopes[0];
        let chosen = report
            .resolution
            .definitions
            .iter()
            .find(|row| report.package.names.get(row.name) == "Chosen")
            .map(|row| row.name)
            .ok_or("actual constructor name")?;
        assert!(
            report
                .resolution
                .bindings
                .get(scope, chosen, Namespace::Value)
                .is_some()
        );
        assert!(
            report
                .resolution
                .bindings
                .get(root, chosen, Namespace::Value)
                .is_none()
        );
        let mut analysis = crate::pipeline::analyze_source(&report)?;
        let program = crate::lower::instances::build_local_prepared(
            &report,
            &mut analysis.facts,
            None,
            analysis.prepared,
        )?;
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == "observed" && symbol.body.is_some()),
            "{name}"
        );
    }
    let plain = crate::test_source::resolve(&fixture("constructor-method")?, false)?;
    let scope = plain
        .resolution
        .source_prelude_scope(&plain.package)?
        .ok_or("plain selected scope")?;
    assert!(
        plain
            .resolution
            .bindings
            .scope(scope)
            .any(|(name, namespace, _)| namespace == Namespace::Value
                && plain.package.names.get(name) == "Chosen")
    );
    assert!(!plain.names_resolved);
    Ok(())
}
#[test]
fn prelude_gates_owners_privacy_and_no_implicit_fail_at_their_actual_consumer() -> TestResult {
    for (name, expected) in [
        ("missing-gate", "prelude_import"),
        ("wrong-owner", "actual use item"),
        ("payload", "word attribute"),
        ("unsafe-wrapper", "word attribute"),
        ("missing-target", "prelude"),
        ("forward-selector", "prelude"),
        ("no-implicit-name", "unresolved value name"),
        ("no-implicit-method", "method"),
        ("private-member", "unresolved value name"),
        ("no-bootstrap-fallback", "unresolved type name"),
    ] {
        let source = fixture(name)?;
        let (phase, error) = match crate::test_source::resolve(&source, true) {
            Err(error) => ("resolve", error),
            Ok(report) => (
                "types",
                crate::pipeline::analyze_source(&report)
                    .err()
                    .ok_or_else(|| format!("{name} unexpectedly accepted"))?
                    .to_string(),
            ),
        };
        assert_eq!(
            phase,
            if name == "no-implicit-method" {
                "types"
            } else {
                "resolve"
            },
            "{name}: {error}"
        );
        let marker = match name {
            "no-implicit-name" | "private-member" => Some("Chosen(7)"),
            "no-bootstrap-fallback" => Some("Option<u32>"),
            _ => None,
        };
        if let Some(marker) = marker {
            let offset = source.find(marker).ok_or("authored opposing use")?;
            assert!(
                error.contains(&format!("file 0: {offset}:")),
                "{name}: {error}"
            );
        }
        assert!(error.contains(expected), "{name}: {error}");
        assert!(!error.contains("budget exceeded"), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn valid_unsupported_selector_shapes_remain_explicit_requirements() -> TestResult {
    for (name, expected) in [
        ("non-glob-root", "single glob"),
        ("multiple", "multiple source prelude"),
    ] {
        let error = crate::test_source::resolve(&fixture(name)?, true)
            .err()
            .ok_or("unsupported shape accepted")?;
        assert!(error.contains(expected), "{name}: {error}");
    }
    Ok(())
}
#[test]
fn source_prelude_owner_effect_and_module_incarnations_are_reauthenticated() -> TestResult {
    let mut report = crate::test_source::resolve(&fixture("constructor-method")?, true)?;
    for corrupt in 0..5 {
        let prelude = report
            .resolution
            .introductions
            .item_scopes
            .as_deref_mut()
            .and_then(|sites| sites.prelude.as_deref_mut())
            .ok_or("actual source selector")?;
        let effect = prelude.effect;
        let node = prelude.item_key.node;
        let context = prelude.context;
        let selected = prelude.selected;
        let selector = prelude.selector;
        match corrupt {
            0 => prelude.effect = usize::MAX,
            1 => prelude.item_key.node = prelude.owner.node,
            2 => prelude.context.scope = ScopeId(u32::MAX),
            3 => prelude.selector.glob = None,
            _ => prelude.selected = selected.map(|(definition, _)| (definition, ScopeId(u32::MAX))),
        }
        assert!(
            report
                .resolution
                .source_prelude_scope(&report.package)
                .is_err()
        );
        let prelude = report
            .resolution
            .introductions
            .item_scopes
            .as_deref_mut()
            .and_then(|sites| sites.prelude.as_deref_mut())
            .ok_or("actual selector")?;
        prelude.effect = effect;
        prelude.item_key.node = node;
        prelude.context = context;
        prelude.selected = selected;
        prelude.selector = selector;
        assert!(
            report
                .resolution
                .source_prelude_scope(&report.package)?
                .is_some()
        );
    }
    let prelude = report
        .resolution
        .introductions
        .item_scopes
        .as_deref()
        .and_then(|sites| sites.prelude.as_deref())
        .ok_or("actual selector")?;
    let context = prelude.context;
    let ItemKind::Use(tree) = prelude.item.kind else {
        return Err("actual use tree".into());
    };
    let selected = prelude.selected;
    let mut state = crate::resolve::index::State::default();
    state.facts = report.resolution;
    assert!(
        state
            .bind(&mut report.package)
            .select_source_prelude(context, tree)
            .is_err()
    );
    assert_eq!(
        state
            .facts
            .introductions
            .item_scopes
            .as_deref()
            .and_then(|sites| sites.prelude.as_deref())
            .ok_or("retained selector")?
            .selected,
        selected
    );
    let payer = report.package.macro_context()?.ok_or("actual payer")?;
    let before = payer.contracts().usage();
    state.facts.source_prelude_scope(&report.package)?;
    let after = payer.contracts().usage();
    assert!(after.0 > before.0);
    assert_eq!(after.1, before.1);
    assert_eq!(size_of::<Resolution>(), 2032);
    assert_eq!(size_of::<crate::resolve::package::Package>(), 256);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    eprintln!(
        "source prelude cold={} owner={}",
        size_of::<Prelude>(),
        size_of::<super::super::container_scopes::ItemScopes>()
    );
    Ok(())
}
#[test]
fn source_prelude_failure_payment_precedes_selector_publication() -> TestResult {
    let mut report = crate::test_source::resolve(&fixture("constructor-method")?, true)?;
    let prelude = report
        .resolution
        .introductions
        .item_scopes
        .as_deref_mut()
        .and_then(|sites| sites.prelude.take())
        .ok_or("actual selector")?;
    let context = report
        .package
        .macro_context()?
        .ok_or("actual request payer")?;
    let remaining = (64 * 1024 * 1024_usize)
        .checked_sub(context.contracts().usage().1)
        .and_then(|bytes| bytes.checked_sub(1))
        .ok_or("remaining real storage budget")?;
    context.contracts().charge(0, remaining)?;
    for attempt in 0..2 {
        let before = context.contracts().usage();
        assert!(
            claim(
                &report.package,
                &mut report.resolution,
                prelude.owner,
                prelude.effect,
                prelude.context
            )
            .is_err()
        );
        assert!(!report.resolution.introductions.has_source_prelude());
        let after = context.contracts().usage();
        assert!(after.0 > before.0);
        if attempt == 0 {
            assert!(after.1 > before.1);
        }
    }
    let report = crate::test_source::resolve("fn ordinary()->u32{7}", true)?;
    assert!(
        report
            .resolution
            .source_prelude_scope(&report.package)?
            .is_none()
    );
    Ok(())
}
#[test]
fn actual_source_builtin_macro_survives_no_implicit_without_name_based_grants() -> TestResult {
    let mut report =
        crate::test_source::source_provider(&fixture("builtin-sized")?, "prelude_builtin")?;
    assert!(
        report
            .resolution
            .source_prelude_scope(&report.package)?
            .is_some()
    );
    crate::pipeline::analyze_source(&report)?;
    assert!(macro_fallback(&mut report)?);
    let impostor = fixture("builtin-sized")?
        .replace("#[rustc_builtin_macro]\n", "")
        .replace("/* compiler built-in */", "true");
    let mut report = crate::test_source::source_provider(&impostor, "ordinary_named_cfg")?;
    assert!(
        !macro_fallback(&mut report)?,
        "a familiar ordinary macro received the builtin exception"
    );
    let ordinary = fixture("constructor-method")?
        .replace("mod consumer {", "#[no_implicit_prelude] mod consumer {");
    assert!(crate::test_source::resolve(&ordinary, true).is_err());
    Ok(())
}

fn macro_fallback(
    report: &mut crate::resolve::expansion::Report,
) -> Result<bool, Box<dyn std::error::Error>> {
    let module = report
        .package
        .modules
        .iter()
        .position(|module| report.package.names.get(module.name) == "consumer")
        .map(|index| u32::try_from(index).map(crate::resolve::package::ModuleId))
        .transpose()?
        .ok_or("actual consumer")?;
    let file = report.package.modules[module.0 as usize].file;
    let scope = report.resolution.module_scopes[module.0 as usize];
    let name = report
        .resolution
        .definitions
        .iter()
        .find(|row| report.package.names.get(row.name) == "cfg")
        .map(|row| row.name)
        .ok_or("actual macro name")?;
    let mut state = crate::resolve::index::State::default();
    state.facts = std::mem::take(&mut report.resolution);
    let selected = state
        .bind(&mut report.package)
        .source_prelude_selection(
            Context {
                module,
                file,
                scope,
                foreign: None,
            },
            name,
            Namespace::Macro,
            ItemContext::ROOT,
            None,
        )?
        .is_some();
    report.resolution = state.facts;
    Ok(selected)
}
