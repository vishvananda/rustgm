use rustgm::{
    expand::{
        config::Config,
        crate_syntax,
        macros::session,
        stream::{ContextId, Environment, Source, Stream},
    },
    literal::Value,
    resolve::{
        self,
        load::{self, InputFile, Sources},
        unit::Unit,
    },
    support::source::{FileId, SourceFile},
    syntax::{
        ast::{Kind, NodeId},
        expression::fragment,
        item::ItemKind,
    },
};
use std::path::Path;

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Err("unexpected source load".into())
    }
}
fn install(unit: &mut Unit, original: &str, corrupt: bool) -> Result<(), String> {
    let report = session::analyze(original, &Config::default()).map_err(|e| format!("{e:?}"))?;
    let call = &report.invocations[0];
    let stream = Stream::transcribe(
        &call.transcription,
        &call.matched,
        &call.input,
        &Environment {
            definition: Source {
                file: FileId(0),
                text: original,
            },
            invocation: Source {
                file: FileId(0),
                text: original,
            },
            context: ContextId(1),
            crate_id: 0,
        },
    )
    .map_err(|e| format!("{e:?}"))?;
    let splices = stream
        .syntax_inputs(&call.matched, original)
        .map_err(|e| format!("{e:?}"))?;
    let (index, context) = unit
        .configured
        .ast
        .nodes
        .iter()
        .enumerate()
        .find_map(|(i, n)| {
            if let Kind::Macro { context, .. } = n.kind {
                Some((i, context))
            } else {
                None
            }
        })
        .ok_or("missing macro target")?;
    let ast = fragment::parse_expansion(&stream.source, &stream.tokens, context, &splices)
        .map_err(|e| format!("{e:?}"))?;
    let mut configured = crate_syntax::configure(&stream.source, ast, &Config::default())
        .map_err(|e| format!("{e:?}"))?;
    if corrupt {
        configured.ast.nodes[0].span.end = u32::MAX;
    }
    unit.install_expansion(
        NodeId::new(u32::try_from(index).map_err(|e| e.to_string())?),
        &stream.source,
        configured,
        &stream.origins,
    )
    .map_err(|e| format!("{e:?}"))?;
    Ok(())
}

#[test]
fn generated_items_enter_name_resolution_with_stable_source_facts() -> Result<(), String> {
    let source =
        "macro_rules! make {()=>{fn answer()->i32{42}}} make!(); fn entry()->i32{answer()}";
    let mut package = load::load(
        InputFile {
            identity: "/crate/lib.rs".into(),
            path: "/crate/lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .map_err(|e| format!("{e:?}"))?;
    let root = package.units[0].configured.ast.root;
    let original_nodes = package.units[0].configured.ast.nodes.len();
    install(&mut package.units[0], source, false)?;
    assert_eq!(package.units[0].source.original(), source);
    assert_eq!(package.units[0].configured.ast.root, root);
    assert!(package.units[0].configured.ast.nodes.len() > original_nodes);
    let facts = resolve::analyze(&mut package).map_err(|e| format!("{e:?}"))?;
    assert!(
        facts
            .definitions
            .iter()
            .any(|d| package.names.get(d.name) == "answer")
    );
    assert!(facts.expansions.is_empty());
    assert!(!package.units[0].origins.is_empty());
    Ok(())
}

#[test]
fn generated_configuration_and_literal_ownership_survive_installation() -> Result<(), String> {
    let source = "macro_rules! m {()=>{#[cfg(any())] fn hidden(){unknown!();1wrong;} fn live(){let a=\"plain\";let b=\"escaped\\n\";}}} m!();";
    let mut unit = Unit::parse(
        SourceFile::new(source.as_bytes().to_vec())?,
        &Config::default(),
    )
    .map_err(|e| format!("{e:?}"))?;
    install(&mut unit, source, false)?;
    let owner = &unit.configured.owners[0];
    assert!(!owner.enabled);
    assert!(!unit.configured.active[owner.node.get() as usize]);
    assert_eq!(
        unit.owner_indices[owner.node.get() as usize].map(std::num::NonZeroU32::get),
        Some(1)
    );
    let hidden = unit
        .configured
        .ast
        .items
        .iter()
        .find(|item| {
            item.name
                .is_some_and(|name| unit.spelling(name).is_ok_and(|s| s == "hidden"))
        })
        .ok_or("missing hidden item")?;
    assert!(matches!(hidden.kind, ItemKind::Function { .. }));
    let values = (0..unit.configured.ast.literals.len())
        .map(|i| {
            unit.literal(u32::try_from(i).map_err(|e| e.to_string())?)
                .map(|v| match v {
                    Value::String(bytes) => bytes.into_owned(),
                    _ => vec![],
                })
                .map_err(|e| format!("{e:?}"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(values, [b"plain".to_vec(), b"escaped\n".to_vec()]);
    assert_eq!(unit.literals.decoded_bytes(), 8);
    Ok(())
}

#[test]
fn failed_installation_restores_syntax_literals_and_source() -> Result<(), String> {
    let source = "macro_rules! m {()=>{\"escaped\\n\"}} fn f(){m!();}";
    let mut unit = Unit::parse(
        SourceFile::new(source.as_bytes().to_vec())?,
        &Config::default(),
    )
    .map_err(|e| format!("{e:?}"))?;
    let nodes = unit.configured.ast.nodes.len();
    let active = unit.configured.active.clone();
    assert!(install(&mut unit, source, true).is_err());
    assert_eq!(unit.source.text(), source);
    assert_eq!(unit.configured.ast.nodes.len(), nodes);
    assert_eq!(unit.configured.active, active);
    assert_eq!(unit.literals.decoded_bytes(), 0);
    assert!(unit.origins.is_empty());
    install(&mut unit, source, false)?;
    Ok(())
}
