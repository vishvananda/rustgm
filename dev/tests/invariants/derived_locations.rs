//! Actual source Location retains nominal identity and ordinary generated bodies.
use super::*;
use crate::{support::bindings::LangItem, test_source};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
#[path = "debug_fields.rs"]
mod debug_fields;
#[path = "formatter_family.rs"]
mod formatter_family;
#[path = "synthetic_ancestry.rs"]
mod synthetic_ancestry;
#[path = "constructed_defaults.rs"]
mod tuple_defaults;
const PROVIDER: &str = "pa31/tests/spec/source-location/location-copy-clone.rs";

#[test]
fn actual_location_derives_reach_source_clone_field_obligations_and_lowering() -> TestResult {
    let source = test_source::fixture(PROVIDER)?;
    let report = test_source::source_provider(&source, "location_source")?;
    let transactions = report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual derive transaction")?;
    assert_eq!(transactions.entries.len(), 1);
    let actual = &transactions.entries[0];
    report.expanded_derive(actual.owner, actual.effect)?;
    assert_eq!(actual.contexts.len(), 2);
    assert_eq!(report.invocations[actual.invocation].parser_calls, 1);
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    assert!(
        facts
            .source_bindings
            .as_deref()
            .ok_or("actual catalog")?
            .validated_language(LangItem::PanicLocation)?
            .is_some()
    );
    let identity = report
        .package
        .source_core()
        .ok_or("actual source provider")?
        .provider
        .clone();
    for result in [
        crate::sema::crates::authored_export::Stage::begin(&report, &mut facts, &identity),
        crate::sema::crates::authored_export::Stage::begin_with_ordinary_traits(
            &report, &mut facts, &identity,
        ),
        crate::sema::crates::authored_export::Stage::begin_module_contracts(
            &report, &mut facts, &identity,
        ),
    ] {
        assert_eq!(
            result.err(),
            Some("source stability contracts are outside this metadata profile")
        );
    }
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    for name in ["location_line", "location_column"] {
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == name && symbol.body.is_some()),
            "{name}"
        );
    }
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/native/source-location");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("provider.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn list_failures_and_changed_capture_image_owner_cannot_consume_derives() -> TestResult {
    client_derive_is_closed()?;
    let source = test_source::fixture(PROVIDER)?;
    for (name, mutated) in [
        (
            "unresolved member",
            source.replace("#[derive(Copy, Clone)]", "#[derive(Copy, Missing)]"),
        ),
        (
            "wrong member kind",
            source.replace("#[derive(Copy, Clone)]", "#[derive(Copy, crate::derive)]"),
        ),
        (
            "duplicate member",
            source.replace("#[derive(Copy, Clone)]", "#[derive(Copy, Clone, Clone)]"),
        ),
    ] {
        super::super::role_declarations::with_engine(&mutated, true, |engine| {
            let before = engine.loader.package.units[0].source.text().len();
            assert!(engine.run().is_err(), "{name}");
            assert_eq!(
                engine.loader.package.units[0].source.text().len(),
                before,
                "{name}"
            );
            assert!(
                engine.loader.package.units[0]
                    .configured
                    .effects
                    .iter()
                    .all(|effect| effect.attribute.effect != Effect::ExpandedDerive),
                "{name}"
            );
            Ok(())
        })?;
    }
    let mut report = expanded_provider(&source, "location_replay")?;
    assert!(!report.names_resolved);
    let (owner, effect, capture) = {
        let entry = &report
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.derives.as_deref())
            .ok_or("actual transaction")?
            .entries[0];
        (entry.owner, entry.effect, entry.capture)
    };
    report.expanded_derive(owner, effect)?;
    synthetic_ancestry::replay(&mut report, owner, effect)?;
    report.package.units[0].configured.effects[effect].capture = capture + 1;
    assert!(report.expanded_derive(owner, effect).is_err());
    report.package.units[0].configured.effects[effect].capture = capture;
    report.expanded_derive(owner, effect)?;
    let owner_index = report.package.units[0].owner_indices[owner.node.get() as usize]
        .ok_or("actual owner index")?
        .get() as usize
        - 1;
    report.package.units[0].configured.owners[owner_index].enabled = false;
    assert!(report.expanded_derive(owner, effect).is_err());
    report.package.units[0].configured.owners[owner_index].enabled = true;
    report.expanded_derive(owner, effect)?;
    let member = report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.derives.as_deref())
        .ok_or("actual transaction")?
        .entries[0]
        .contexts
        .start;
    let definition = report.invocations[member].definition;
    let Body::Builtin(original_role) = report.definitions[definition].body else {
        return Err("actual source role".into());
    };
    report.definitions[definition].body = Body::Builtin(Builtin::Line);
    assert!(report.expanded_derive(owner, effect).is_err());
    report.definitions[definition].body = Body::Builtin(original_role);
    report.expanded_derive(owner, effect)?;
    let fresh = crate::support::source::SourceFile::new(
        report.package.units[0].source.text().as_bytes().to_vec(),
    )?;
    let previous = std::mem::replace(&mut report.package.units[0].source, fresh);
    assert!(report.expanded_derive(owner, effect).is_err());
    report.package.units[0].source = previous;
    report.expanded_derive(owner, effect)?;
    let unit = report.package.units[0]
        .fresh_instance()
        .map_err(|error| error.message)?;
    assert!(
        unit.configured
            .effects
            .iter()
            .all(|effect| effect.attribute.effect != Effect::ExpandedDerive)
    );
    assert_eq!(unit.source.text(), unit.source.original());
    Ok(())
}

#[test]
fn marker_shape_ordinary_storage_and_derive_owner_layouts_preserve_boundaries() -> TestResult {
    synthetic_ancestry::units()?;
    trivial_role_headers()?;
    for (source, accept) in [
        (
            "#![feature(marker_trait_attr)]#[marker]trait M<'a>{}impl<'a>M<'a>for u8{}",
            true,
        ),
        ("#[marker]trait M{}", false),
        (
            "#![feature(marker_trait_attr)]#[marker]trait M{fn f();}",
            false,
        ),
        (
            "#![feature(marker_trait_attr)]#[marker]trait M{}impl<T>M for T{}impl M for u8{}",
            false,
        ),
    ] {
        let result = test_source::resolve(source, true).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accept, "{source}: {result:?}");
    }
    for (name, source, accepted) in [
        (
            "literal allowance",
            "#![feature(rustc_attrs,allow_internal_unstable)]#[allow_internal_unstable(marker_trait_attr)]macro_rules!m{()=>{#[marker]trait M{}}}m!();",
            true,
        ),
        (
            "captured caller",
            "#![feature(rustc_attrs,allow_internal_unstable)]#[allow_internal_unstable(marker_trait_attr)]macro_rules!m{($a:meta)=>{#[$a]trait M{}}}m!(marker);",
            false,
        ),
    ] {
        let result = test_source::resolve(source, true).and_then(|report| {
            crate::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert_eq!(result.is_ok(), accepted, "{name}: {result:?}");
    }
    super::super::role_declarations::with_engine("fn ordinary(){}", false, |engine| {
        engine.run()?;
        assert!(
            engine
                .attribute_effects
                .as_deref()
                .and_then(|effects| effects.derives.as_deref())
                .is_none()
        );
        assert!(engine.calls.is_empty());
        assert_eq!(engine.builtin_definitions.len(), 24);
        eprintln!(
            "derive layouts Effect={} ConfiguredAttribute={} EffectRow={} CallGrammar={} Call={} Engine={} Effects={} Transaction={} Transactions={} Report={} Body={} Definition={} Reservations={} Validated={}",
            size_of::<Effect>(),
            size_of::<ConfiguredAttribute>(),
            size_of::<crate::expand::crate_syntax::AttributeEffect>(),
            size_of::<CallGrammar>(),
            size_of::<super::super::Call>(),
            size_of_val(engine),
            size_of::<super::super::attribute_effects::Effects>(),
            size_of::<Transaction>(),
            size_of::<Transactions>(),
            size_of::<super::super::Report>(),
            size_of::<super::super::Body>(),
            size_of::<super::super::Definition>(),
            size_of::<crate::resolve::bindings::Reservations>(),
            crate::sema::compiler_items::validated_layout_bytes()
        );
        // Inline import work adds one lineage pointer and nine usize counters.
        assert_eq!(size_of::<super::super::Report>(), 3096);
        assert_eq!(size_of::<super::super::Body>(), 96);
        assert_eq!(size_of::<super::super::Definition>(), 136);
        assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
        Ok(())
    })?;
    Ok(())
}

fn client_derive_is_closed() -> TestResult {
    use crate::{
        resolve,
        support::{
            bindings::Mode,
            crates::{self, CrateIdentity, Dependencies, Dependency},
        },
    };
    let text =
        test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?;
    let provider = test_source::source_provider(&text, "derive_provider")?;
    let identity = provider
        .package
        .source_core()
        .ok_or("actual provider")?
        .provider
        .clone();
    let mut analysis = crate::pipeline::analyze_source(&provider)?;
    let (_, interface) = crate::lower::instances::build_library_prepared(
        &provider,
        &mut analysis.facts,
        identity.clone(),
        analysis.prepared,
    )?;
    let interface = crates::decode(&crates::encode(&interface)?)?;
    let dependencies = Arc::new(Dependencies::new(
        vec![interface],
        vec![Dependency::new("base".into(), identity.clone())?],
    )?);
    let context = resolve::bindings::Context {
        crate_name: Some("derive_client".into()),
        current: Some(CrateIdentity::new("derive_client", "v1")?),
        dependencies: Some(dependencies),
        mode: Mode::SourceClient { provider: identity },
    };
    let source = "#![feature(no_core)]#![no_core]extern crate base;#[derive(Copy,Clone)]struct D;";
    let expanded = resolve::expansion::analyze_with_context(
        resolve::load::InputFile {
            identity: "derive-client.rs".into(),
            path: "derive-client.rs".into(),
            source: crate::support::source::SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut super::super::role_declarations::NoFiles,
        &crate::expand::config::Config::default(),
        &context,
    )?;
    let report = resolve::analyze_expanded(expanded)?;
    assert!(report.package.source_client().is_some());
    let error = crate::pipeline::analyze_source(&report)
        .err()
        .ok_or("foreign derive used bootstrap properties")?;
    assert!(
        error
            .message
            .contains("source derive requires its actual completed local expansion"),
        "{error}"
    );
    Ok(())
}

fn expanded_provider(source: &str, tag: &str) -> TestResult<super::super::Report> {
    use crate::{
        resolve,
        support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    };
    let identity = CrateIdentity::new("minicore", tag)?;
    let context = resolve::bindings::Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    Ok(resolve::expansion::analyze_with_context(
        resolve::load::InputFile {
            identity: "location-expansion.rs".into(),
            path: "location-expansion.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut super::super::role_declarations::NoFiles,
        &crate::expand::config::Config::default(),
        &context,
    )?)
}

fn trivial_role_headers() -> TestResult {
    let prefix = test_source::fixture("pa31/tests/spec/source-functions-transport/minimal-provider.rs")?
        .replace("#![feature(no_core,lang_items,rustc_attrs,fundamental)]", "#![feature(no_core,lang_items,rustc_attrs,fundamental,const_trait_impl,marker_trait_attr)]")
        .replace("pub trait Clone: Sized", "pub const trait Clone: Sized");
    // Authored header reducers isolate structural role validation. The complete
    // Location source above retains all original stability attributes and its
    // required execution control is never replaced with these header checks.
    let declaration =
        "#[lang=\"trivial_clone\"]#[marker]pub const unsafe trait Renamed:[const]Clone{}";
    let report = test_source::source_provider(&(prefix.clone() + declaration), "trivial_header")?;
    let facts = crate::sema::declarations::analyze(&report)?;
    let claimed = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(LangItem::TrivialClone)?
        .ok_or("actual TrivialClone role")?;
    assert_eq!(
        report
            .package
            .names
            .get(report.resolution.definitions[claimed.definition.0 as usize].name),
        "Renamed"
    );
    assert_eq!(LangItem::TrivialClone.index(), 34);
    assert_eq!(LangItem::ALL[34], LangItem::TrivialClone);
    for malformed in [
        declaration.replace("unsafe trait", "trait"),
        declaration.replace("const unsafe", "unsafe"),
        declaration.replace("#[marker]", ""),
        declaration.replace("[const]Clone", "Clone"),
    ] {
        let report = test_source::source_provider(&(prefix.clone() + &malformed), "wrong_trivial")?;
        assert!(
            crate::sema::declarations::analyze(&report).is_err(),
            "{malformed}"
        );
    }
    Ok(())
}
