//! Exact source contracts and reached edition-frame identities survive warm queries.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        bindings::Context,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    syntax::ast::NodeId,
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn actual() -> TestResult<Report> {
    let source = crate::test_source::fixture(
        "pa31/tests/spec/source-panic-macros/permission-valid-wrapper.rs",
    )?;
    Ok(crate::test_source::source_provider(
        &source,
        "panic_permissions",
    )?)
}
fn frame(report: &Report) -> TestResult<Frame> {
    let invocation = report
        .invocations
        .iter()
        .find(|invocation| report.definitions[invocation.definition].edition_panic)
        .ok_or("actual permitted wrapper frame")?;
    let definition = &report.definitions[invocation.definition];
    Ok(Frame {
        context: invocation.context,
        definition: invocation.definition,
        call: invocation.node,
        original: invocation.original,
        parent: invocation.parent,
        owner: definition.node,
        module: definition.module,
        edition: definition.edition,
        skip: definition.edition_panic,
    })
}

#[test]
fn macro_allowance_warm_queries_authenticate_full_attribute_and_retained_trees() -> TestResult {
    let mut report = actual()?;
    let (key, owner, index, capture, attribute, allowance) = {
        let state = report
            .package
            .source_core()
            .ok_or("provider")?
            .macros
            .0
            .borrow();
        let entry = state
            .declarations
            .values()
            .find(|entry| entry.declaration.allowance.is_some())
            .ok_or("actual allowance owner")?;
        let allowance = entry.declaration.allowance.ok_or("allowance")?;
        let witness = entry
            .witnesses
            .iter()
            .find(|witness| witness.attribute == allowance.attribute)
            .ok_or("actual attribute witness")?;
        (
            entry.declaration.node,
            entry.declaration.owner,
            witness.index,
            witness.capture,
            witness.attribute,
            allowance,
        )
    };
    assert!(existing(&report.package, key)?.is_some());
    assert!(allowance.matches(key, attribute));
    assert!(!allowance.matches(
        NodeKey {
            node: NodeId::new(u32::MAX),
            ..key
        },
        attribute
    ));
    let mut changed = attribute;
    changed.unsafe_wrapper = true;
    assert!(!allowance.matches(key, changed));
    report.package.units[key.file.0 as usize].configured.effects[index]
        .attribute
        .attribute = changed;
    assert!(declaration(&report.package, owner).is_err());
    report.package.units[key.file.0 as usize].configured.effects[index]
        .attribute
        .attribute = attribute;
    assert!(declaration(&report.package, owner)?.is_some());
    let forest = &report.package.units[key.file.0 as usize]
        .configured
        .ast
        .attributes[capture as usize]
        .forest;
    let changed = forest.map_spans(&mut |span| {
        Ok(Span {
            start: span.start + 1,
            end: span.end + 1,
        })
    })?;
    report.package.units[key.file.0 as usize]
        .configured
        .ast
        .attributes[capture as usize]
        .forest = changed;
    assert!(declaration(&report.package, owner).is_err());
    Ok(())
}

#[test]
fn panic_frame_cache_checks_actual_callsite_definition_and_permission() -> TestResult {
    let report = actual()?;
    let actual = frame(&report)?;
    let contracts = &report.package.source_core().ok_or("provider")?.macros;
    assert_eq!(contracts.edition(actual)?, Some(Edition::Rust2024));
    for changed in [
        Frame {
            definition: usize::MAX,
            ..actual
        },
        Frame {
            call: NodeKey {
                node: NodeId::new(u32::MAX),
                ..actual.call
            },
            ..actual
        },
        Frame {
            original: Span {
                start: actual.original.start + 1,
                ..actual.original
            },
            ..actual
        },
        Frame {
            parent: ExpansionId(u32::MAX),
            ..actual
        },
        Frame {
            owner: None,
            ..actual
        },
        Frame {
            module: ModuleId(u32::MAX),
            ..actual
        },
        Frame {
            edition: Edition::Rust2021,
            ..actual
        },
        Frame {
            skip: false,
            ..actual
        },
    ] {
        assert!(contracts.edition(changed).is_err());
    }
    assert!(
        contracts
            .0
            .borrow()
            .editions
            .keys()
            .all(|frame| frame.0 != 0)
    );
    assert_eq!(contracts.edition(actual)?, Some(Edition::Rust2024));
    Ok(())
}

#[test]
fn exhausted_macro_contract_work_denies_both_warm_permissions_and_frames() -> TestResult {
    let report = actual()?;
    let actual = frame(&report)?;
    let key = actual.owner.ok_or("source wrapper")?;
    let contracts = &report.package.source_core().ok_or("provider")?.macros;
    assert!(existing(&report.package, key)?.is_some());
    assert!(contracts.edition(actual)?.is_some());
    contracts.0.borrow_mut().steps = 1_000_000;
    assert!(contracts.charge(1, 0).is_err());
    assert!(existing(&report.package, key).is_err());
    assert!(contracts.edition(actual).is_err());
    assert!(
        contracts
            .retain_editions(&[actual], Edition::Rust2024)
            .is_err()
    );
    Ok(())
}

#[test]
fn permitted_wrapper_suffix_work_grows_with_depth_plus_reached_width() -> TestResult {
    use std::fmt::Write;
    for (depth, width) in [(8_usize, 8_usize), (32, 32), (128, 128)] {
        let mut source = String::from("#![recursion_limit=\"512\"]#![feature(no_core,lang_items,rustc_attrs,allow_internal_unstable)]#![no_core]
#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}
#[lang=\"sized\"]trait Sized:MetaSized{}
mod panic{macro_rules!chosen{($($input:tt)*)=>{21u32};}pub(crate)use chosen as panic_2021;}
#[rustc_builtin_macro(core_panic)]macro_rules!route{($($input:tt)*)=>{9u32};}
#[allow_internal_unstable(edition_panic)]macro_rules!layer0{($($input:tt)*)=>{{");
        for _ in 1..width {
            source.push_str("route!($($input)*);");
        }
        source.push_str("route!($($input)*)}};}");
        for layer in 1..depth {
            write!(
                source,
                "#[allow_internal_unstable(edition_panic)]macro_rules!layer{layer}{{($($input:tt)*)=>{{layer{}!($($input)*)}};}}",
                layer - 1
            )?;
        }
        write!(
            source,
            "pub fn answer()->u32{{layer{}!(tag[nested]{{tokens}})}}",
            depth - 1
        )?;
        let report = crate::test_source::source_provider(&source, "panic_width")?;
        let state = report
            .package
            .source_core()
            .ok_or("provider")?
            .macros
            .0
            .borrow();
        assert_eq!(state.editions.len(), depth);
        assert!(
            state.steps <= 256 * (depth + width) + 4096,
            "{depth}/{width}: {}",
            state.steps
        );
        eprintln!(
            "panic suffix depth={depth} width={width} steps={} frames={} attempted-bytes={}",
            state.steps,
            state.editions.len(),
            state.bytes
        );
    }
    Ok(())
}

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
#[test]
fn pinned_legacy_core_header_retains_all_attributes_around_actual_builtin_claim() -> TestResult {
    let header =
        "#![feature(no_core,lang_items,rustc_attrs,allow_internal_unstable,staged_api)]#![no_core]
#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}
#[lang=\"sized\"]trait Sized:MetaSized{}";
    // Exact declaration shape from pinned core/macros/mod.rs:1-13 (file hash in proposal).
    let declaration_source = r#"
#[doc = include_str!("panic.md")]
#[macro_export]
#[rustc_builtin_macro(core_panic)]
#[allow_internal_unstable(edition_panic)]
#[stable(feature = "core", since = "1.6.0")]
#[rustc_diagnostic_item = "core_panic_macro"]
macro_rules! panic {
    ($($arg:tt)*) => { /* compiler built-in */ };
}
"#;
    let identity = CrateIdentity::new("minicore", "panic_header")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let package = crate::resolve::load::load_with_context(
        InputFile {
            identity: "header.rs".into(),
            path: "header.rs".into(),
            source: SourceFile::new(format!("{header}{declaration_source}").into_bytes())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    let key = package.units[0].configured.owners.iter().find_map(|owner| {
        let mut node = owner.node;
        while let Kind::Attributed { value, .. } = package.units[0].configured.ast.node(node).kind { node = value; }
        matches!(package.units[0].configured.ast.node(node).kind,
            Kind::Item(index) if matches!(package.units[0].configured.ast.items[index as usize].kind, ItemKind::MacroRules { .. }))
            .then_some(NodeKey { file: crate::resolve::unit::FileId(0), node: owner.node })
    }).ok_or("actual attributed macro_rules declaration")?;
    let contract = declaration(&package, key)?.ok_or("actual source builtin claim")?;
    assert_eq!(contract.builtin, Some(Builtin::CorePanic));
    assert!(contract.allowance.is_some());
    let state = package
        .source_core()
        .ok_or("source provider")?
        .macros
        .0
        .borrow();
    let entry = state
        .declarations
        .get(&contract.node)
        .ok_or("retained contract")?;
    assert_eq!(entry.witnesses.len(), 2);
    assert_eq!(
        entry.declaration.attributes.1 - entry.declaration.attributes.0,
        6
    );
    eprintln!(
        "panic cold layouts SourceCore={} Contracts={} Declaration={} Entry={} Frame={} Definition={} payload={}",
        size_of::<super::super::SourceCore>(),
        size_of::<Contracts>(),
        size_of::<Declaration>(),
        size_of::<Entry>(),
        size_of::<Frame>(),
        size_of::<crate::resolve::expansion::Definition>(),
        state.witness_bytes
    );
    Ok(())
}
