//! Shared supertrait DAGs are indexed once per complete object application.
use crate::{
    expand::config::Config,
    resolve::{
        expansion,
        load::{InputFile, Sources},
    },
    sema::{declarations, model::Declarations},
    support::source::SourceFile,
    types::{Kind, TypeId},
};
use std::{fmt::Write, path::Path, sync::Arc};

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn inherited_membership_does_not_rewalk_the_objects_captured_graph()
-> Result<(), Box<dyn std::error::Error>> {
    for count in [8, 32, 128] {
        let mut source = "trait T0{type A;}type A0<'a>=&'a u8;".to_owned();
        for i in 1..count {
            write!(
                source,
                "trait T{i}:T{}{{}}type A{i}<'a>=(&'a A{}<'a>,&'a A{}<'a>);",
                i - 1,
                i - 1,
                i - 1
            )?;
        }
        write!(
            source,
            "type Object<'a>=dyn T{}<A=A{}<'a>>+'a;",
            count - 1,
            count - 1
        )?;
        let mut facts = declarations(&source)?;
        let ty = (0..u32::try_from(facts.types.len())?)
            .map(TypeId)
            .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
            .ok_or("missing captured object type")?;
        let (index, work) = super::get(&mut facts, ty)?;
        eprintln!("captured graph {count}: {work} indexing steps");
        assert_eq!(index.traits.len(), count);
        assert!(
            work <= 300 * count,
            "{count} nodes and captures: {work} steps"
        );
    }
    Ok(())
}

fn declarations(source: &str) -> Result<Declarations, Box<dyn std::error::Error>> {
    let report = expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?;
    let report = crate::resolve::analyze_expanded(report)?;
    Ok(declarations::analyze(&report)?)
}

#[test]
fn shared_diamonds_use_linear_work_and_one_cached_index() -> Result<(), Box<dyn std::error::Error>>
{
    for quantified in [false, true] {
        for count in [8, 32, 128] {
            let binder = if quantified { "for<'a>" } else { "" };
            let mut source = "trait T0{type A;}trait T1:T0{}".to_owned();
            for i in 2..count {
                write!(
                    source,
                    "trait T{i}:{binder}T{}+{binder}T{}{{}}",
                    i - 1,
                    i - 2
                )?;
            }
            write!(source, "type Object=dyn T{}<A=u8>;", count - 1)?;
            let mut facts = declarations(&source)
                .map_err(|error| format!("{count} nodes, quantified={quantified}: {error}"))?;
            let ty = (0..u32::try_from(facts.types.len())?)
                .map(TypeId)
                .find(|&ty| matches!(facts.types.kind(ty), Some(Kind::Dynamic { .. })))
                .ok_or("missing indexed object type")?;
            let (index, work) = super::get(&mut facts, ty)?;
            eprintln!("diamond {count}, quantified={quantified}: {work} indexing steps");
            assert_eq!(index.traits.len(), count);
            assert_eq!(index.equalities.len(), 1);
            assert!(work <= 180 * count, "{count} nodes: {work} steps");
            let cached = facts.types.len();
            for _ in 0..count {
                let (reused, work) = super::get(&mut facts, ty)?;
                assert!(Arc::ptr_eq(&index, &reused));
                assert_eq!(work, 1);
            }
            assert_eq!(facts.types.len(), cached);
        }
    }
    Ok(())
}
