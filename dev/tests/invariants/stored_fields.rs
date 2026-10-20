//! Shared storage shapes do not authorize another caller's erased lifetime inputs.
use super::Preparation;
#[path = "suspended_recipes.rs"]
mod waits;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    sema::{model::Declarations, projections},
    support::source::SourceFile,
    types::{
        ItemId, Kind, Primitive, RegionId, TypeId,
        arguments::Argument,
        lifetime_proof::{Obligations, ScopeId},
    },
};
use std::path::Path;

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str) -> Result<resolve::expansion::Report, Box<dyn std::error::Error>> {
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "fields.rs".into(),
            path: "fields.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)?)
}
fn context() -> Result<(Declarations, TypeId), Box<dyn std::error::Error>> {
    let source = "trait A<'a>{type Out;}impl<'a,'b:'a>A<'a>for &'b u8{type Out=u16;}struct W<'a,T:A<'a>>{value:T::Out}";
    let report = report(source)?;
    let owner = report
        .resolution
        .definitions
        .iter()
        .position(|definition| report.package.names.get(definition.name) == "W")
        .ok_or("missing field owner")?;
    let mut facts = crate::sema::declarations::analyze(&report)?;
    let argument = facts.types.intern(Kind::Reference {
        region: RegionId(8),
        mutable: false,
        referent: TypeId::primitive(Primitive::U8),
    })?;
    let arguments = facts
        .types
        .intern_arguments(&[Argument::Lifetime(RegionId(7)), Argument::Type(argument)])?;
    let ty = facts
        .types
        .intern(Kind::Adt(ItemId(u32::try_from(owner)?), arguments))?;
    facts.aggregates.instantiate(&mut facts.types, ty)?;
    Ok((facts, ty))
}

#[test]
fn normalized_storage_preserves_raw_inputs_and_separate_caller_debts()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut facts, ty) = context()?;
    let raw = facts
        .aggregates
        .raw_shape(&facts.types, ty)?
        .variant(0)
        .ok_or("missing raw field group")?
        .fields;
    assert!(facts.aggregates.shape(&facts.types, ty).is_err());
    let mut caller = Preparation::default();
    let mut first = Obligations::new(100);
    caller.prepare(
        &mut facts,
        ty,
        &mut projections::concrete(&mut first, ScopeId::ROOT)?,
    )?;
    assert!(!first.outlives.is_empty());
    let fields = facts
        .aggregates
        .shape(&facts.types, ty)?
        .variant(0)
        .ok_or("missing normalized field group")?
        .fields;
    assert_eq!(
        facts.types.list(fields),
        Some([TypeId::primitive(Primitive::U16)].as_slice())
    );
    assert_eq!(
        facts
            .aggregates
            .raw_shape(&facts.types, ty)?
            .variant(0)
            .ok_or("missing retained field group")?
            .fields,
        raw
    );

    let mut repeated = Obligations::new(first.regions.end);
    caller.prepare(
        &mut facts,
        ty,
        &mut projections::concrete(&mut repeated, ScopeId::ROOT)?,
    )?;
    assert!(repeated.is_empty());
    let mut independent = Obligations::new(first.regions.end);
    Preparation::default().prepare(
        &mut facts,
        ty,
        &mut projections::concrete(&mut independent, ScopeId::ROOT)?,
    )?;
    assert!(!independent.outlives.is_empty());
    assert!(independent.regions.start >= first.regions.end);
    assert_eq!(
        facts
            .aggregates
            .shape(&facts.types, ty)?
            .variant(0)
            .ok_or("missing shared normalized group")?
            .fields,
        fields
    );
    Ok(())
}

#[test]
fn repeated_stored_fields_share_graph_work_within_one_caller()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut facts, ty) = context()?;
    for width in [1, 16, 64, 256] {
        let list = facts.types.intern_list(&vec![ty; width])?;
        let root = facts.types.intern(Kind::Tuple(list))?;
        let mut caller = Preparation::default();
        let mut proof = Obligations::new(100);
        caller.prepare(
            &mut facts,
            root,
            &mut projections::concrete(&mut proof, ScopeId::ROOT)?,
        )?;
        assert!(!proof.outlives.is_empty());
        assert!(
            caller.steps <= 4 * width + 128,
            "width {width}: {} steps",
            caller.steps
        );
    }
    Ok(())
}

#[test]
fn independent_callers_replay_shared_projection_leaves_without_rewalking_storage()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut facts, leaf) = context()?;
    let mut root = leaf;
    for depth in 1..=256 {
        // A diamond at every depth has only one unique projected field leaf.
        // A flattened per-root descendant set or per-caller storage walk would
        // retain or revisit prefixes instead of sharing the dependency graph.
        let list = facts.types.intern_list(&[root, root])?;
        root = facts.types.intern(Kind::Tuple(list))?;
        if ![1, 16, 64, 256].contains(&depth) {
            continue;
        }
        let mut first = Obligations::new(100);
        Preparation::default().prepare(
            &mut facts,
            root,
            &mut projections::concrete(&mut first, ScopeId::ROOT)?,
        )?;
        let mut next = first.regions.end;
        for _ in 0..8 {
            let mut proof = Obligations::new(next);
            let mut caller = Preparation::default();
            caller.prepare(
                &mut facts,
                root,
                &mut projections::concrete(&mut proof, ScopeId::ROOT)?,
            )?;
            assert!(!proof.outlives.is_empty());
            assert!(caller.steps <= 128, "depth {depth}: {} steps", caller.steps);
            next = proof.regions.end;
        }
    }
    Ok(())
}

#[test]
fn growing_field_frontiers_share_prefixes_without_copying_prior_leaves()
-> Result<(), Box<dyn std::error::Error>> {
    let (mut facts, mut root) = context()?;
    let Some(Kind::Adt(owner, _)) = facts.types.kind(root) else {
        return Err("missing stored nominal identity".into());
    };
    for width in 1..=128_u32 {
        let input = facts.types.intern(Kind::Reference {
            region: RegionId(width * 2 + 8),
            mutable: false,
            referent: TypeId::primitive(Primitive::U8),
        })?;
        let arguments = facts.types.intern_arguments(&[
            Argument::Lifetime(RegionId(width * 2 + 7)),
            Argument::Type(input),
        ])?;
        let leaf = facts.types.intern(Kind::Adt(owner, arguments))?;
        let list = facts.types.intern_list(&[root, leaf])?;
        root = facts.types.intern(Kind::Tuple(list))?;
        let mut proof = Obligations::new(1_000);
        let mut caller = Preparation::default();
        caller.prepare(
            &mut facts,
            root,
            &mut projections::concrete(&mut proof, ScopeId::ROOT)?,
        )?;
        assert!(proof.outlives.len() >= width as usize);
        assert!(caller.steps <= 128 * (width as usize + 1));
        let graph = facts
            .stored_fields
            .as_ref()
            .ok_or("missing storage graph")?;
        assert!(graph.recipes.len() <= 2 * width as usize + 1);
    }
    Ok(())
}
