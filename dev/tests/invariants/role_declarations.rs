//! Known compiler declarations are typed identities, never execution permission.
use super::*;
use crate::support::{bindings::Mode, crates::CrateIdentity, source::SourceFile};
use std::path::Path;
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fixture(name: &str) -> TestResult<String> {
    Ok(crate::test_source::fixture(&format!(
        "pa31/tests/spec/source-builtin-declarations/{name}"
    ))?)
}
fn with_engine(
    source: &str,
    provider: bool,
    body: impl FnOnce(&mut Engine<'_, NoFiles>) -> TestResult,
) -> TestResult {
    let config = Config::default();
    let mut files = NoFiles;
    let context = if provider {
        let current = CrateIdentity::new("minicore", "known_roles")?;
        super::super::bindings::Context {
            crate_name: Some("minicore".into()),
            current: Some(current.clone()),
            mode: Mode::Source { provider: current },
            ..Default::default()
        }
    } else {
        super::super::bindings::Context::default()
    };
    let mut engine = Engine::create(
        InputFile {
            identity: "known-roles.rs".into(),
            path: "known-roles.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut files,
        &config,
        &context,
    )?;
    body(&mut engine)
}

fn real_frames_cannot_borrow_unavailable_role_headers() -> TestResult {
    let mut report =
        crate::test_source::source_provider(&fixture("frame-forgery.rs")?, "role_frames")?;
    let unit = &report.package.units[0];
    let span = unit
        .configured
        .ast
        .nodes
        .iter()
        .find_map(|node| {
            if let Kind::Name(span, _) = node.kind {
                (unit
                    .source
                    .text()
                    .get(span.start as usize..span.end as usize)
                    == Some("probe"))
                .then_some(span)
            } else {
                None
            }
        })
        .ok_or("executed source token")?;
    let site = Location {
        file: FileId(0),
        span,
    };
    let feature = super::super::bindings::KnownFeature::PanicInternals.name();
    assert!(report.macro_allowances().allows(site, &feature)?);
    let unavailable: Vec<_> = report
        .definitions
        .iter()
        .enumerate()
        .filter_map(|(index, definition)| {
            matches!(definition.body, Body::Builtin(role) if !role.executable()).then_some(index)
        })
        .collect();
    assert_eq!(unavailable.len(), 38);
    for index in unavailable {
        report.invocations[0].definition = index;
        for forged in [false, true] {
            if forged {
                report.definitions[index].body = Body::Builtin(Builtin::Line);
            }
            let error = report
                .macro_allowances()
                .allows(site, &feature)
                .err()
                .ok_or("unavailable role lent a frame")?;
            assert!(
                error.message.contains("has no executed expansion frame"),
                "{error}"
            );
        }
    }
    Ok(())
}

#[test]
fn pinned_roles_keep_exact_kind_owner_and_closed_execution_availability() -> TestResult {
    let golden = fixture("roles.tsv")?;
    let report =
        crate::test_source::source_provider(&fixture("all-known-roles.rs")?, "role_inventory")?;
    for line in golden.lines() {
        let fields: Vec<_> = line.split_ascii_whitespace().collect();
        let [alias, name, kind, available] = fields.as_slice() else {
            return Err("invalid role source contract".into());
        };
        let definition = report
            .definitions
            .iter()
            .find(|definition| report.package.names.get(definition.name) == *alias)
            .ok_or("actual known role declaration missing")?;
        let Body::Builtin(role) = definition.body else {
            return Err("known role would execute a placeholder".into());
        };
        assert_eq!(role.name(), *name);
        assert_eq!(format!("{:?}", role.kind()), *kind);
        assert_eq!(role.executable(), *available == "1");
        let key = definition.node.ok_or("source role has no actual owner")?;
        let header = super::super::bindings::panic_contracts::published(
            &report.package,
            key,
            definition.header.as_ref(),
            1,
        )?;
        assert_eq!(header.builtin, Some(role));
        assert_ne!(header.source_owner(), key);
        if role.bootstrap_slot().is_none() {
            assert!(role.source_only());
        }
    }
    assert_eq!(golden.lines().count(), 61);
    assert_eq!(
        report
            .definitions
            .iter()
            .filter(|d| matches!(d.body, Body::Builtin(_)))
            .count(),
        61
    );
    assert!(report.invocations.is_empty());
    assert_eq!(report.captures.len(), 0);
    assert_eq!(report.package.marks.context_count(), 1);
    Ok(())
}

#[test]
fn unavailable_alias_and_kind_opponents_stop_before_execution_work() -> TestResult {
    for (name, message) in [
        (
            "unavailable-bang.rs",
            "builtin bang macro execution is not implemented",
        ),
        (
            "unavailable-attribute-alias.rs",
            "builtin attribute execution is not implemented",
        ),
        ("derive-as-bang.rs", "unresolved macro"),
        ("derive-as-attribute.rs", "unresolved attribute macro"),
        (
            "source-derive-consumer.rs",
            "builtin attribute execution is not implemented",
        ),
    ] {
        with_engine(&fixture(name)?, true, |engine| {
            let error = engine
                .run()
                .err()
                .ok_or("unavailable compiler role executed")?;
            assert!(error.message.contains(message), "{name}: {error}");
            assert!(engine.invocations.is_empty());
            assert_eq!(engine.captures.len(), 0);
            assert_eq!(engine.loader.package.marks.context_count(), 1);
            Ok(())
        })?;
    }
    with_engine(&fixture("disabled-unavailable-use.rs")?, true, |engine| {
        engine.run()?;
        assert!(engine.invocations.is_empty());
        Ok(())
    })
}

#[test]
fn unknown_claims_forms_and_provider_opponents_do_not_change_ordinary_derives() -> TestResult {
    for name in [
        "unknown-role.rs",
        "wrong-owner.rs",
        "wrong-derive-form.rs",
        "missing-gate.rs",
        "malformed-claim.rs",
        "duplicate-claim.rs",
    ] {
        assert!(
            crate::test_source::source_provider(&fixture(name)?, "role_opponent").is_err(),
            "{name}"
        );
    }
    with_engine(
        "#![feature(rustc_attrs)]#[rustc_builtin_macro(assert)]macro_rules!actual{()=>{9u8};}",
        false,
        |engine| {
            assert!(engine.run().is_err());
            assert!(engine.loader.package.source_core().is_none());
            Ok(())
        },
    )?;
    let report = crate::test_source::resolve(&fixture("ordinary-derive.rs")?, true)?;
    crate::pipeline::analyze_source(&report)?;
    real_frames_cannot_borrow_unavailable_role_headers()?;
    Ok(())
}

#[test]
fn declaration_width_and_ordinary_synthetic_storage_keep_their_owners() -> TestResult {
    let all = fixture("all-known-roles.rs")?;
    let (header, _) = all
        .split_once("#[rustc_builtin_macro")
        .ok_or("role header")?;
    for width in [8_usize, 32, 128] {
        let mut source = header.to_owned();
        for index in 0..width {
            use std::fmt::Write;
            writeln!(
                source,
                "#[rustc_builtin_macro(offload_kernel)]pub macro role_{index}($item:item){{}}"
            )?;
        }
        with_engine(&source, true, |engine| {
            engine.run()?;
            assert_eq!(engine.definitions.len(), width);
            let (work, bytes, _) = engine
                .loader
                .package
                .source_core()
                .ok_or("provider")?
                .macros
                .usage();
            assert!(work < 8192 + width * 4096);
            assert!(bytes < 131_072 + width * 16_384);
            eprintln!("known-role-width={width} work={work} retained={bytes}");
            Ok(())
        })?;
    }
    with_engine("fn ordinary(){}", false, |engine| {
        engine.run()?;
        assert_eq!(engine.builtin_definitions.len(), 24);
        assert_eq!(engine.builtin_definitions.capacity(), 24);
        for (ordinal, role) in Builtin::BOOTSTRAP.into_iter().enumerate() {
            assert_eq!(role as usize, ordinal);
        }
        for role in Builtin::ALL
            .into_iter()
            .filter(|role| role.bootstrap_slot().is_none())
        {
            assert!(engine.builtin_definition(role).is_err());
        }
        assert_eq!(engine.builtin_definitions.capacity(), 24);
        assert_eq!(size_of::<Builtin>(), 1);
        assert_eq!(size_of::<Option<Builtin>>(), 1);
        assert_eq!(size_of::<MacroTarget>(), 16);
        assert_eq!(size_of::<Body>(), 96);
        assert_eq!(size_of::<Definition>(), 136);
        eprintln!(
            "role layouts kind={} call={} engine={} body={} definition={} target={} slots={} payload={}",
            size_of::<builtin::BuiltinKind>(),
            size_of::<Call>(),
            size_of::<Engine<'_, NoFiles>>(),
            size_of::<Body>(),
            size_of::<Definition>(),
            size_of::<MacroTarget>(),
            engine.builtin_definitions.capacity(),
            engine.builtin_definitions.capacity() * size_of::<Option<usize>>()
        );
        Ok(())
    })
}
