//! Early constructor identities must not evaluate constants or prove projections.
use super::{Heads, Rigid};
use crate::support::hash::HashMap;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
        model::DefKind,
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::source::SourceFile,
    syntax::{ast::Kind, item::ItemKind},
    types::{ItemId, Primitive},
};
use std::{fmt::Write as _, path::Path};

type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn fixture(source: &str) -> TestResult<(Report, Declarations, HashMap<String, NodeKey>)> {
    let report = resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "header-heads.rs".into(),
            path: "header-heads.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )?)?;
    let facts = Declarations::new(&report)?;
    let mut aliases = HashMap::default();
    for definition in &report.resolution.definitions {
        if definition.kind != DefKind::TypeAlias {
            continue;
        }
        let key = definition.node.ok_or("missing alias source")?;
        let ast = &report.package.unit(key.file).configured.ast;
        let Kind::Item(index) = ast.node(key.node).kind else {
            return Err("missing alias item".into());
        };
        if let ItemKind::TypeAlias { ty: Some(node), .. } = ast.items[index as usize].kind {
            aliases.insert(
                report.package.names.get(definition.name).to_owned(),
                NodeKey { node, ..key },
            );
        }
    }
    Ok((report, facts, aliases))
}

#[test]
fn resolved_heads_exclude_only_known_outer_constructors() -> TestResult {
    let source = r"
        trait A { type Out; const C: usize; }
        impl A for u8 { type Out = u16; const C: usize = 3; }
        struct X<const N: usize>;
        type Id<T> = T;
        type Default<T = u8> = T;
        type Forward<T = Default<u16>> = Id<T>;
        type Scalar = Id<u8>;
        type Omitted = Default;
        type Forwarded = Forward;
        type Nominal = Id<X<{<u8 as A>::C}>>;
        type Shared = Id<&'static X<{<u8 as A>::C}>>;
        type Mutable = &'static mut u8;
        type Pointer = *const u8;
        type MutPointer = *mut u8;
        type Array = [u8; <u8 as A>::C];
        type Slice = [u8];
        type Tuple = (u8, u16);
        type Function = for<'a> fn(&'a u8) -> u8;
        type Never = !;
        type Unknown<T> = T;
        type Projected = <u8 as A>::Out;
    ";
    let (report, facts, aliases) = fixture(source)?;
    let nominal = report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| d.kind == DefKind::Struct && report.package.names.get(d.name) == "X")
        .map(|(index, _)| u32::try_from(index))
        .transpose()?
        .ok_or("missing X")?;
    let mut heads = Heads::new(&report, &facts);
    for (name, expected) in [
        ("Scalar", Some(Rigid::Primitive(Primitive::U8))),
        ("Omitted", Some(Rigid::Primitive(Primitive::U8))),
        ("Forwarded", Some(Rigid::Primitive(Primitive::U16))),
        ("Nominal", Some(Rigid::Nominal(ItemId(nominal)))),
        ("Shared", Some(Rigid::Reference(false))),
        ("Mutable", Some(Rigid::Reference(true))),
        ("Pointer", Some(Rigid::Pointer(false))),
        ("MutPointer", Some(Rigid::Pointer(true))),
        ("Array", Some(Rigid::Array)),
        ("Slice", Some(Rigid::Slice)),
        ("Tuple", Some(Rigid::Tuple)),
        ("Function", Some(Rigid::Function)),
        ("Never", Some(Rigid::Never)),
        ("Unknown", None),
        ("Projected", None),
    ] {
        assert_eq!(
            heads.shape(*aliases.get(name).ok_or("missing query")?)?,
            expected,
            "{name}"
        );
    }
    assert!(facts.definitions.iter().all(Option::is_none));
    assert!(facts.initializers.is_empty());
    Ok(())
}

#[test]
fn alias_head_graphs_share_work_before_constant_arguments_exist() -> TestResult {
    for depth in [8, 32, 128] {
        let mut source =
            "trait A{const C:usize;}struct X<const N:usize>;type P0=X<{<u8 as A>::C}>;".to_owned();
        for index in 1..=depth {
            write!(source, "type P{index}=P{};", index - 1)?;
        }
        let (report, facts, aliases) = fixture(&source)?;
        let mut heads = Heads::new(&report, &facts);
        let root = *aliases.get(&format!("P{depth}")).ok_or("missing root")?;
        let expected = heads.shape(root)?.ok_or("missing nominal head")?;
        for index in 0..=depth {
            assert_eq!(
                heads.shape(*aliases.get(&format!("P{index}")).ok_or("missing alias")?)?,
                Some(expected)
            );
        }
        assert!(
            heads.steps <= 12 * depth + 64,
            "depth={depth} steps={}",
            heads.steps
        );
        let before = heads.steps;
        for _ in 0..128 {
            assert_eq!(heads.shape(root)?, Some(expected));
        }
        assert_eq!(heads.steps - before, 128);
    }
    Ok(())
}
