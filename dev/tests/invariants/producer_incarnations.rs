//! Source completion checks the actual selected unit incarnations before sealing.
use super::*;
use crate::{
    expand::config::Config,
    lower::transported_source::collector_origin::CollectorOrigin,
    resolve::{
        bindings::Context,
        load::{InputFile, Sources},
    },
    sema::crates::authored_export::Stage,
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
};
use std::path::Path;

struct Part;
impl Sources for Part {
    fn read(&mut self, path: &Path) -> std::result::Result<Option<InputFile>, String> {
        if path.file_name().is_none_or(|name| name != "part.rs") {
            return Ok(None);
        }
        Ok(Some(InputFile {
            identity: path.to_owned(),
            path: path.to_owned(),
            source: SourceFile::new(b"pub fn selected<T>(value:T)->T{value}".to_vec())?,
        }))
    }
}
fn actual() -> Result<(Report, Declarations, Stage, Box<CollectorOrigin>)> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    source.push_str("pub mod part;");
    let provider = CrateIdentity::new("minicore", "collector_units")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(provider.clone()),
        mode: Mode::Source {
            provider: provider.clone(),
        },
        ..Context::default()
    };
    let input = InputFile {
        identity: "provider.rs".into(),
        path: "provider.rs".into(),
        source: SourceFile::new(source.into_bytes())?,
    };
    let expanded = crate::resolve::expansion::analyze_with_context(
        input,
        &mut Part,
        &Config::default(),
        &context,
    )?;
    let report = crate::resolve::analyze_expanded(expanded)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let mut stage = Stage::begin_with_ordinary_traits(&report, &mut facts, &provider)?;
    let origin = CollectorOrigin::new(&report, &mut facts, &mut stage)?;
    Ok((report, facts, stage, origin))
}
pub(super) fn reject_replaced_producer_units() -> Result {
    let (mut report, facts, mut stage, origin) = actual()?;
    assert!(report.package.units.len() > 1);
    origin.require_root(&report, &facts, &mut stage)?;
    stage.collect_header_columns_checked(&report, &facts, |stage, declaration| {
        origin
            .require_header(&report, declaration, stage)
            .map_err(Error::unlocated)
    })?;
    let selected = report
        .resolution
        .definitions
        .iter()
        .find_map(|definition| {
            (definition.kind == crate::resolve::model::DefKind::Function)
                .then_some(definition.node)
                .flatten()
                .filter(|key| key.file.0 != 0)
        })
        .ok_or("genuine selected nonroot source")?;
    report.package.units[selected.file.0 as usize].source =
        report.package.unit(selected.file).source.physical_copy();
    // Root and arena are unchanged; the actual selected-header walk must fail.
    origin.require_root(&report, &facts, &mut stage)?;
    let error = stage
        .collect_header_columns_checked(&report, &facts, |stage, declaration| {
            origin
                .require_header(&report, declaration, stage)
                .map_err(Error::unlocated)
        })
        .err()
        .ok_or("replaced nonroot source admitted")?;
    assert!(
        error.message.contains("replaced source incarnation"),
        "{error}"
    );
    let (_, foreign, _, _) = actual()?;
    assert!(origin.require_store(&foreign, &mut stage).is_err());
    report.package.units[0].source = report.package.units[0].source.physical_copy();
    assert!(origin.require_root(&report, &facts, &mut stage).is_err());
    eprintln!(
        "collector-layout source={} origin={} trait_inspection={}",
        size_of::<Source>(),
        size_of::<CollectorOrigin>(),
        size_of::<TraitInspection>()
    );
    Ok(())
}

pub(super) fn retain_actual_failed_freeze_cost() -> Result {
    let (source, _, _) = inspect(HOLDER)?;
    let wrong = CrateIdentity::new("foreign", "freeze_owner")?;
    let mut work = g::Work::default();
    let mut observed = Vec::new();
    let error = g::Payload::from_source_provided_observed(
        &wrong,
        source.rows,
        0,
        &mut work,
        &mut |bytes, peak| observed.push((bytes, peak)),
    )
    .err()
    .ok_or("foreign actual freeze owner admitted")?;
    assert_eq!(
        error,
        "source module root differs from its actual artifact identity"
    );
    assert!(work.steps > 0);
    assert_eq!(observed.len(), 1);
    assert!(observed[0].0 > 0 && observed[0].1 >= observed[0].0);
    eprintln!(
        "collector-failed-freeze retained={} peak={} steps={}",
        observed[0].0, observed[0].1, work.steps
    );
    Ok(())
}
