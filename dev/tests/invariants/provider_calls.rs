//! Warm layout data cannot authorize a different or malformed checked call.
use super::*;
use crate::{
    body,
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
    },
    sema,
    support::{bindings::Mode, crates::CrateIdentity, source::SourceFile},
    types::{TypeId, arguments::Argument},
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn checked_bodies() -> Result<(
    resolve::expansion::Report,
    sema::model::Declarations,
    body::ir::Program,
)> {
    let source = "#![feature(no_core,lang_items,intrinsics,rustc_attrs)]#![no_core]#[lang=\"pointee_sized\"]trait PointeeSized{}#[lang=\"meta_sized\"]trait MetaSized:PointeeSized{}#[lang=\"sized\"]trait Sized:MetaSized{}#[rustc_intrinsic]const fn size_of<T>()->usize;#[rustc_intrinsic]const fn align_of<T>()->usize;const fn good()->usize{size_of::<u32>()}const fn bad()->usize{size_of::<u32>()}";
    let identity = CrateIdentity::new("minicore", "provider_v1")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(identity.clone()),
        mode: Mode::Source { provider: identity },
        ..Default::default()
    };
    let expanded = resolve::expansion::analyze_with_context(
        InputFile {
            identity: "provider.rs".into(),
            path: "provider.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?;
    let report = resolve::analyze_expanded(expanded)?;
    let facts = sema::declarations::analyze(&report)?;
    let facts = sema::bodies::analyze(&report, facts)?;
    let program = body::build(&report, &facts)?;
    Ok((report, facts, program))
}
#[test]
fn warmed_layout_data_still_requires_the_current_direct_checked_call() -> Result {
    let (report, mut facts, mut program) = checked_bodies()?;
    let mut catalog = Catalog::default();
    let good = catalog.insert(program.bodies.first().ok_or("good")?)?;
    let bad_body = program.bodies.get_mut(1).ok_or("bad")?;
    let destination = bad_body
        .values
        .iter()
        .position(|v| matches!(v.kind, Rvalue::Call { .. }))
        .ok_or("call")?;
    bad_body.values[destination].ty = TypeId::primitive(Primitive::U8);
    let bad = catalog.insert(bad_body)?;
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
    machine.begin(good, &[]).map_err(|e| e.message)?;
    let crate::sema::values::execution::Progress::Prepare(
        crate::sema::values::execution::Requirement::FunctionBody(target),
    ) = machine.advance().map_err(|e| e.message)?
    else {
        return Err("missing reached call".into());
    };
    let (state, _) = machine.suspend();
    let call = state
        .source_layout_call(&facts, &catalog, target)?
        .ok_or("witness")?;
    catalog.cache_layout(&call, 4)?;
    assert_eq!(catalog.layout_result(target), Some(4));
    let frame = state.frames.last().ok_or("frame")?;
    assert!(
        validate(
            &facts,
            catalog.code(good).ok_or("code")?,
            frame,
            frame.next + 1,
            target
        )
        .is_err()
    );
    let Some(Kind::FunctionItem {
        arguments,
        signature,
        ..
    }) = facts.types.kind(target.ty())
    else {
        return Err("target".into());
    };
    assert!(matches!(
        facts.types.arguments(arguments).and_then(|mut a| a.next()),
        Some(Argument::Type(_))
    ));
    let definition = facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == "align_of"
        })
        .ok_or("align")?
        .definition;
    let wrong = facts.types.intern(Kind::FunctionItem {
        item: crate::types::ItemId(definition.0),
        arguments,
        signature,
    })?;
    assert!(
        state
            .source_layout_call(&facts, &catalog, Instance::new(&facts.types, wrong)?)
            .is_err()
    );
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
    machine.restore(state);
    let crate::sema::values::execution::Progress::Complete(value) =
        machine.advance().map_err(|e| e.message)?
    else {
        return Err("warmed valid caller suspended".into());
    };
    assert_eq!(
        value.as_scalar().map_err(sema::values::Failure::message)?,
        sema::model::Scalar::Integer(4)
    );
    drop(machine);
    let evaluator = sema::values::Evaluator::new(&report, &mut facts);
    let mut machine = Machine::catalog(evaluator, &catalog, crate::const_eval::Limits::default());
    machine.begin(bad, &[]).map_err(|e| e.message)?;
    assert_eq!(
        machine.advance().err().map(|e| e.message),
        Some("source layout call differs from its checked const contract")
    );
    Ok(())
}
