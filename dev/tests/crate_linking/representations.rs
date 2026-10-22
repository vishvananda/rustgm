//! Source-qualified arrays and record representations use ordinary imported facts.
use super::{artifacts, crates, imports};
use crates::{
    CrateIdentity, Dependencies, Dependency, ExternalDef, ExternalKind, Function, Interface,
    Nominals, Record, RecordField, RecordRepr, Scalar, Signature, Symbol, TypeGraph, TypeNode,
    TypeRef,
};
use std::{fmt::Write, path::Path, process::Command, sync::Arc};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn qualify(native: bool) -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let evidence = root.join(format!(
        "obj/parallel/aggregate-metadata/maintainer-{}-{native}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos()
    ));
    let mut command = Command::new("python3");
    command
        .current_dir(root)
        .arg(root.join("scripts/check_aggregate_metadata.py"))
        .arg("--app")
        .arg(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--evidence-root")
        .arg(&evidence);
    if native {
        command.arg("--native");
    }
    artifacts::checked("aggregate-contract", &evidence, &mut command)?;
    Ok(())
}

#[test]
fn array_and_c_record_contracts_cross_metadata_boundaries() -> Result {
    qualify(false)
}

#[test]
#[ignore = "requires configured CPPGM object backend, host C compiler and pinned Rust"]
fn arrays_and_c_records_link_with_independent_clients() -> Result {
    qualify(true)
}

fn graph_artifact(width: u16, length: u64) -> Result<Interface> {
    let owner = CrateIdentity::new("arrays", "remap")?;
    let graph = TypeGraph::new(vec![
        TypeNode::Scalar(Scalar::U8),
        TypeNode::Array {
            element: TypeRef(0),
            length,
        },
    ])?;
    let functions = (0..width)
        .map(|index| {
            Function::new(
                ExternalDef::new(
                    owner.clone(),
                    vec![format!("f{index:03}")],
                    ExternalKind::Function,
                )?,
                Signature::with_shared_types(
                    graph.clone(),
                    0,
                    vec![TypeRef(1)],
                    TypeRef(1),
                    false,
                )?,
                Symbol::Rust,
            )
        })
        .collect::<std::result::Result<Vec<_>, _>>()?;
    let record = Record::new(
        ExternalDef::new(owner.clone(), vec!["Packet".into()], ExternalKind::Record)?,
        true,
        vec![RecordField::new("items".into(), TypeRef(1), true)?],
    )?
    .with_repr(RecordRepr::C);
    Ok(Interface::with_nominals(
        owner,
        vec![],
        functions,
        vec![],
        Nominals::new(graph, vec![record])?,
    )?)
}

#[test]
fn array_import_work_depends_on_graph_nodes_not_roots_or_extents() -> Result {
    for width in [8, 32, 128] {
        for length in [0, 8, 4096] {
            let artifact = graph_artifact(width, length)?;
            let owner = artifact.identity().clone();
            let definitions = artifact.records()[0].definition().clone();
            let artifact = crates::decode(&crates::encode(&artifact)?)?;
            let dependencies = Arc::new(Dependencies::new(
                vec![artifact],
                vec![Dependency::new("arrays".into(), owner)?],
            )?);
            let report = imports::resolve("fn main(){}", dependencies)?;
            let definition = report
                .resolution
                .crates
                .as_ref()
                .ok_or("imports")?
                .definition(&definitions)
                .ok_or("record")?;
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let work = analysis
                .facts
                .crates
                .as_ref()
                .ok_or("linkage")?
                .import_work();
            assert_eq!(work.type_nodes, 2);
            assert_eq!(work.signatures, usize::from(width));
            assert_eq!(work.roots, 2 * usize::from(width));
            let record = analysis
                .facts
                .aggregates
                .get(rustgm::types::ItemId(definition.0))
                .ok_or("aggregate")?;
            assert_eq!(
                record.repr.representation,
                rustgm::types::aggregate::Representation::C
            );
        }
    }
    Ok(())
}

#[test]
fn export_array_graphs_preserve_shared_fields_and_reject_unfinished_representations() -> Result {
    for width in [8, 32, 128] {
        let mut source = "pub struct Packet{pub items:[[u16;2];2]}".to_owned();
        for index in 0..width {
            write!(source, "pub fn f{index:03}(value:Packet)->Packet{{value}}")?;
        }
        let (_, interface) = super::exports::compile(&source, "arrays")?;
        let graph = interface.nominals().ok_or("graph")?.types();
        assert_eq!(graph.nodes().len(), 4);
        assert!(
            interface
                .functions()
                .iter()
                .all(|function| graph.shares_storage(function.signature().types()))
        );
    }
    for attribute in ["C,packed", "C,align(16)", "transparent"] {
        let text = format!("#[repr({attribute})] pub struct Packet{{pub byte:u8}}");
        let error = super::exports::compile(&text, "unsupported")
            .err()
            .ok_or("unsupported contract accepted")?;
        assert!(error.to_string().contains("metadata"), "{error}");
    }
    Ok(())
}
