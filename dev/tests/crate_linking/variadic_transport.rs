//! Imported variadics retain the same call, ownership and native ABI contracts.
use rustgm::{
    support::{
        crates::{
            self, Abi, CrateIdentity, Dependencies, Dependency, ExternalDef, ExternalKind,
            Function, Interface, Nominals, Scalar, Signature, Static, Storage, StorageOrigin,
            Symbol, TypeGraph, TypeNode, TypeRef,
        },
        linkage::Linkage,
    },
    types::{Kind, callable::Abi as CallingAbi},
};
use std::sync::Arc;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn variadic_metadata_preserves_source_rules_and_repeated_publication() -> Result {
    let folder =
        super::static_transport::qualify_manifest(30, "variadic-metadata-outcomes.json", false)?;
    let bytes = std::fs::read(folder.join("va_provider.rgm"))?;
    assert_eq!(&bytes[12..15], &[8, 7, 4]);
    let interface = crates::decode(&bytes)?;
    let graph = interface.nominals().ok_or("missing record graph")?.types();
    for function in interface.functions() {
        let ordinary = function.definition().path() == ["ordinary"];
        let signature = function.signature();
        assert_eq!(signature.variadic(), !ordinary);
        assert_eq!(signature.abi(), if ordinary { Abi::Rust } else { Abi::C });
        assert_eq!(
            signature.unsafe_(),
            !ordinary && function.definition().path() != ["ignores"]
        );
        assert!(signature.types().shares_storage(graph));
        if !ordinary {
            assert_eq!(
                function.symbol(),
                &Symbol::Exact(function.definition().path()[0].clone().into())
            );
        }
    }
    assert_eq!(crates::encode(&interface)?, bytes);
    Ok(())
}

#[test]
#[ignore = "requires configured CPPGM object backend, C compiler and pinned Rust toolchain"]
fn variadic_metadata_calls_execute_through_separate_objects() -> Result {
    super::static_transport::qualify_manifest(30, "variadic-metadata-outcomes.json", true)
        .map(|_| ())
}

fn shared(width: u16) -> Result<Interface> {
    let owner = CrateIdentity::new("variadic", "shared")?;
    let graph = TypeGraph::new(vec![TypeNode::Scalar(Scalar::U32)])?;
    let mut functions = Vec::new();
    for index in 0..width {
        let name = format!("f{index:03}");
        functions.push(Function::new(
            ExternalDef::new(owner.clone(), vec![name.clone()], ExternalKind::Function)?,
            Signature::with_shared_types(graph.clone(), 0, vec![TypeRef(0)], TypeRef(0), true)?
                .with_convention(Abi::C, true)?,
            Symbol::Exact(name.into()),
        )?);
    }
    let global = Static::new(
        ExternalDef::new(owner.clone(), vec!["value".into()], ExternalKind::Static)?,
        TypeRef(0),
        Symbol::Exact("foreign_value".into()),
        true,
        false,
        StorageOrigin::Foreign { unsafe_: true },
    )?;
    Ok(Interface::with_storage(
        owner,
        vec![],
        functions,
        vec![],
        Nominals::for_storage(graph, vec![])?,
        Storage::new(vec![global], vec![])?,
    )?)
}

#[test]
fn variadic_imports_remap_one_shared_graph_with_linear_roots() -> Result {
    for width in [8u16, 32, 128] {
        let bytes = crates::encode(&shared(width)?)?;
        let interface = crates::decode(&bytes)?;
        let alias = Dependency::new("many".into(), interface.identity().clone())?;
        let report = super::imports::resolve(
            "fn main(){}",
            Arc::new(Dependencies::new(vec![interface], vec![alias])?),
        )?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let facts = &analysis.facts;
        let work = facts.crates.as_deref().ok_or("linkage")?.import_work();
        assert_eq!(work.type_nodes, 1);
        assert_eq!(work.signatures, usize::from(width));
        assert_eq!(work.roots, 2 * usize::from(width));
        for (id, _) in report
            .resolution
            .crates
            .as_deref()
            .ok_or("imports")?
            .functions()
        {
            let function = facts
                .foreign
                .as_deref()
                .and_then(|registry| registry.function(id))
                .ok_or("foreign declaration")?;
            assert_eq!(function.linkage, Linkage::Import);
            let Some(Kind::Function {
                convention,
                unsafe_,
                ..
            }) = facts.types.kind(function.signature)
            else {
                return Err("missing callable".into());
            };
            assert_eq!(convention.abi, CallingAbi::C);
            assert!(convention.variadic && unsafe_);
            assert!(report.resolution.definitions[id.0 as usize].node.is_none());
        }
        assert!(bytes.len() <= 128 * usize::from(width) + 512);
        eprintln!(
            "variadics={width} types={} signatures={} roots={} bytes={}",
            work.type_nodes,
            work.signatures,
            work.roots,
            bytes.len()
        );
    }
    Ok(())
}
