//! Completed attribute expressions retain ordinary lookup and their source owner.
use super::*;
use std::fmt::Write as _;
type TestResult = Result<(), Box<dyn std::error::Error>>;
fn fixture(name: &str) -> Result<String, std::io::Error> {
    crate::test_source::fixture(&format!("pa31/tests/spec/diagnostic-values/{name}.rs"))
}
fn provider(source: &str) -> Result<super::super::Report, String> {
    crate::test_source::source_provider(source, "attribute_values")
}
#[test]
fn actual_nested_and_opaque_values_use_real_macro_definitions() -> TestResult {
    for (name, expected) in [
        ("legacy", vec!["u8_legacy_mod", "i16_legacy_mod"]),
        ("stacked-module", vec!["u8_legacy_mod", "i16_legacy_mod"]),
        ("nested", vec!["prefixu8"]),
        ("alias-function", vec!["aliasu8"]),
        ("alias-module", vec!["aliasu8"]),
        ("shadow", vec!["shadowed"]),
        ("ordinary", vec!["ordinary\\n"]),
        ("forwarded", vec!["forwarded"]),
        ("signed-concat", vec!["n-7"]),
    ] {
        let report = provider(&fixture(name)?)?;
        let values = report
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.values.as_deref())
            .ok_or("values")?;
        let mut actual = Vec::new();
        assert!(values.pending.is_empty(), "{name}");
        for value in values.entries.values() {
            let unit = report.package.unit(value.owner.file);
            let mut owner = value.owner;
            while let Kind::Attributed { value, .. } = unit.configured.ast.node(owner.node).kind {
                owner.node = value;
            }
            let root = values
                .completed(&report.package, owner, value.attribute)?
                .ok_or("root")?;
            actual.push(
                unit.assembly_string(root)
                    .map_err(|e| e.message)?
                    .to_owned(),
            );
            assert_eq!(
                unit.configured.effects[value.effect].attribute.attribute,
                value.attribute
            );
            assert!(matches!(value.attribute.input, AttributeInput::Equals(_)));
        }
        actual.sort_unstable();
        let mut expected = expected;
        expected.sort_unstable();
        assert_eq!(actual, expected, "{name}");
    }
    Ok(())
}
#[test]
fn hard_payload_scope_and_feature_opponents_still_reject() -> TestResult {
    for name in [
        "nonliteral",
        "wrong-kind",
        "byte-string",
        "block",
        "duplicate",
        "missing-feature",
        "missing-macro",
        "multiple-values",
        "module-parent-alias",
    ] {
        assert!(provider(&fixture(name)?).is_err(), "accepted {name}");
    }
    for name in ["literal", "cfg-removed"] {
        let report = provider(&fixture(name)?)?;
        assert!(
            report
                .attribute_effects
                .as_deref()
                .is_none_or(|effects| effects.values.is_none())
        );
        assert!(report.invocations.is_empty());
    }
    Ok(())
}
#[test]
fn completed_value_rejects_changed_effect_and_configured_owner() -> TestResult {
    let mut report = provider(&fixture("ordinary")?)?;
    let values = report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.values.as_deref())
        .ok_or("values")?;
    let value = values.entries.values().next().ok_or("value")?;
    let unit = &mut report.package.units[value.owner.file.0 as usize];
    let mut owner = value.owner;
    while let Kind::Attributed { value, .. } = unit.configured.ast.node(owner.node).kind {
        owner.node = value;
    }
    let index = unit.owner_indices[value.owner.node.get() as usize]
        .ok_or("owner")?
        .get() as usize
        - 1;
    unit.configured.owners[index].enabled = false;
    assert!(
        values
            .completed(&report.package, owner, value.attribute)
            .is_err()
    );
    report.package.units[owner.file.0 as usize]
        .configured
        .owners[index]
        .enabled = true;
    values.completed(&report.package, owner, value.attribute)?;
    report.package.units[owner.file.0 as usize]
        .configured
        .effects[value.effect]
        .attribute
        .attribute
        .unsafe_wrapper = true;
    assert!(
        values
            .completed(&report.package, owner, value.attribute)
            .is_err()
    );
    Ok(())
}
#[test]
fn external_module_values_keep_the_declaration_file_and_the_child_scope() -> TestResult {
    use crate::{
        resolve::load::{InputFile, Sources},
        support::source::SourceFile,
    };
    struct Files;
    impl Sources for Files {
        fn read(&mut self, path: &std::path::Path) -> Result<Option<InputFile>, String> {
            if path.file_name().is_none_or(|name| name != "selected.rs") {
                return Ok(None);
            }
            let source = crate::test_source::fixture(
                "pa31/tests/spec/diagnostic-values/external/selected.rs",
            )
            .map_err(|e| e.to_string())?;
            Ok(Some(InputFile {
                identity: "selected.rs".into(),
                path: path.into(),
                source: SourceFile::new(source.into_bytes())?,
            }))
        }
    }
    let report = crate::test_source::source_provider_with_sources(
        &fixture("external/root")?,
        "attribute_values",
        &crate::expand::config::Config::default(),
        &mut Files,
    )?;
    assert_eq!(report.package.units.len(), 2);
    let values = report
        .attribute_effects
        .as_deref()
        .and_then(|effects| effects.values.as_deref())
        .ok_or("values")?;
    assert_eq!(values.entries.len(), 1);
    assert_eq!(
        values.entries.values().next().ok_or("value")?.owner.file,
        FileId(0)
    );
    assert_eq!(report.invocations.len(), 2);
    assert!(
        report
            .invocations
            .iter()
            .all(|call| call.node.file == FileId(0))
    );
    assert!(
        crate::test_source::source_provider_with_sources(
            &fixture("external/nonliteral")?,
            "attribute_values",
            &crate::expand::config::Config::default(),
            &mut Files,
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn attribute_work_is_sparse_and_scales_with_demanded_values() -> TestResult {
    let seed = fixture("nested")?;
    let (base, _) = seed.split_once("#[rustc_diagnostic_item=").ok_or("base")?;
    let mut previous = None;
    for width in [8_usize, 32, 128] {
        let mut source = base.to_owned();
        for index in 0..width {
            write!(
                source,
                "#[rustc_diagnostic_item=concat!(\"item_\",stringify!({index}))]pub mod m{index}{{}}"
            )?;
        }
        let report = provider(&source)?;
        let values = report
            .attribute_effects
            .as_deref()
            .and_then(|effects| effects.values.as_deref())
            .ok_or("values")?;
        let nodes = report.package.units[0].configured.ast.nodes.len();
        let work = report.lint_producer().ok_or("policy")?.work();
        assert_eq!(values.entries.len(), width);
        assert_eq!(report.invocations.len(), 2 * width);
        if let Some((prior_nodes, steps, bytes)) = previous {
            assert!(nodes < 6 * prior_nodes);
            assert!(work.steps < 6 * steps && work.bytes < 6 * bytes);
        }
        previous = Some((nodes, work.steps, work.bytes));
        eprintln!(
            "attribute width={width} nodes={nodes} work={} bytes={} invocations={}",
            work.steps,
            work.bytes,
            report.invocations.len()
        );
    }
    let plain = crate::test_source::resolve("fn plain(){}", true)?;
    assert!(plain.attribute_effects.is_none());
    assert!(size_of::<Visit>() <= 64);
    assert_eq!(size_of::<Work>(), 16);
    eprintln!(
        "attribute layouts Visit={} Work={} Effects={} Value={} Unit={} Call={}",
        size_of::<Visit>(),
        size_of::<Work>(),
        size_of::<super::super::attribute_effects::Effects>(),
        size_of::<Value>(),
        size_of::<crate::resolve::unit::Unit>(),
        size_of::<super::super::Call>()
    );
    Ok(())
}
#[test]
fn expanded_diagnostic_identity_keeps_ordinary_executable_body() -> TestResult {
    let source = fixture("native")?;
    let report = provider(&source)?;
    let crate::pipeline::Analysis {
        mut facts,
        prepared,
    } = crate::pipeline::analyze_source(&report)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let out = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/diagnostic-values-v1/candidate-native");
    std::fs::create_dir_all(&out)?;
    std::fs::write(out.join("source.rs"), source)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(out.join("provider.lowir"))?,
    )?;
    Ok(())
}
