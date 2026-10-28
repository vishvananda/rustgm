//! Source-free external contracts enter the ordinary resolver and inference pipeline.
use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{
        crates::{
            CrateIdentity, Dependencies, Dependency, ExternalDef, ExternalKind, Function,
            Interface, Scalar, Signature, Symbol,
        },
        source::SourceFile,
    },
};
use std::{path::Path, sync::Arc};
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn identity(name: &str) -> Result<CrateIdentity, &'static str> {
    CrateIdentity::new(name, "v1")
}
fn artifact(name: &str, dependencies: &[&str], unsafe_: bool) -> Result<Interface, &'static str> {
    let owner = identity(name)?;
    let function = Function::new(
        ExternalDef::new(
            owner.clone(),
            vec!["math".into(), "value".into()],
            ExternalKind::Function,
        )?,
        Signature::new(vec![Scalar::U32], Scalar::U32, unsafe_)?,
        Symbol::Rust,
    )?;
    Interface::new(
        owner,
        dependencies
            .iter()
            .map(|name| identity(name))
            .collect::<Result<_, _>>()?,
        vec![function],
    )
}
fn dependencies(reverse: bool) -> Result<Arc<Dependencies>, &'static str> {
    let mut artifacts = vec![
        artifact("first", &[], false)?,
        artifact("second", &["first"], true)?,
    ];
    let mut aliases = vec![
        Dependency::new("left".into(), identity("first")?)?,
        Dependency::new("right".into(), identity("second")?)?,
    ];
    if reverse {
        artifacts.reverse();
        aliases.reverse();
    }
    Ok(Arc::new(Dependencies::new(artifacts, aliases)?))
}
pub(super) fn resolve(text: &str, dependencies: Arc<Dependencies>) -> Result<Report, String> {
    resolve::expansion::analyze_with_crates(
        InputFile {
            identity: "client.rs".into(),
            path: "client.rs".into(),
            source: SourceFile::new(text.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
        None,
        Some(dependencies),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|error| error.to_string())
}
#[test]
fn imported_contracts_have_canonical_remaps_and_no_fabricated_bodies()
-> Result<(), Box<dyn std::error::Error>> {
    let text = "use left::math::value as call;fn main(){let _=call(7);let _=unsafe{right::math::value(9)};let _:core::option::Option<u8>=None;}";
    let first = resolve(text, dependencies(false)?)?;
    let second = resolve(text, dependencies(true)?)?;
    let analysis = rustgm::pipeline::analyze_source(&first)?;
    let left = first
        .resolution
        .crates
        .as_ref()
        .ok_or("missing imported facts")?;
    let right = second
        .resolution
        .crates
        .as_ref()
        .ok_or("missing reordered imports")?;
    assert_eq!(left.dependencies().edges(), 1);
    for (definition, function) in left.functions() {
        assert_eq!(right.definition(function.definition()), Some(definition));
        assert!(
            first.resolution.definitions[definition.0 as usize]
                .node
                .is_none()
        );
        assert!(analysis.facts.definitions[definition.0 as usize].is_some());
        assert!(
            !analysis
                .facts
                .functions
                .iter()
                .any(|body| body.definition == definition)
        );
        assert!(!analysis.facts.is_const_function(definition));
        let record = analysis
            .facts
            .foreign
            .as_ref()
            .and_then(|registry| registry.function(definition))
            .ok_or("missing imported contract")?;
        assert_eq!(record.linkage, rustgm::support::linkage::Linkage::Import);
        assert_eq!(
            analysis
                .facts
                .crates
                .as_ref()
                .and_then(|linkage| linkage.rust_symbol(definition)),
            Some(function.definition().rust_symbol().as_str())
        );
    }
    Ok(())
}
#[test]
fn external_aliases_respect_explicit_imports_privacy_and_safety()
-> Result<(), Box<dyn std::error::Error>> {
    for text in [
        "extern crate left as renamed;fn main(){let _=renamed::math::value(1);}",
        "mod m{pub fn f()->u32{left::math::value(7)}}fn main(){let _=m::f();}",
    ] {
        let report = resolve(text, dependencies(false)?)?;
        rustgm::pipeline::analyze_source(&report)?;
    }
    for text in [
        "fn main(){let _=right::math::value(1);}",
        "fn main(){let _=left::private(1);}",
        "const C:u32=left::math::value(1);fn main(){}",
    ] {
        let rejected = resolve(text, dependencies(false)?).and_then(|report| {
            rustgm::pipeline::analyze_source(&report)
                .map(|_| ())
                .map_err(|error| error.to_string())
        });
        assert!(rejected.is_err(), "accepted {text}");
        if text.starts_with("const") {
            assert!(rejected.is_err_and(|message| message.contains("non-const")));
        }
    }
    Ok(())
}
#[test]
fn graph_validation_is_bounded_and_rejects_missing_duplicate_and_cyclic_artifacts()
-> Result<(), Box<dyn std::error::Error>> {
    assert!(Dependencies::new(vec![artifact("second", &["first"], false)?], vec![]).is_err());
    assert!(
        Dependencies::new(
            vec![
                artifact("first", &[], false)?,
                artifact("first", &[], false)?
            ],
            vec![]
        )
        .is_err()
    );
    assert!(
        Dependencies::new(
            vec![
                artifact("first", &["second"], false)?,
                artifact("second", &["first"], false)?
            ],
            vec![]
        )
        .is_err()
    );
    assert!(
        Dependencies::new(
            vec![artifact("first", &[], false)?],
            vec![Dependency::new("left".into(), identity("missing")?)?]
        )
        .is_err()
    );
    for width in [8usize, 32, 128] {
        let mut artifacts = Vec::new();
        for index in 0..width {
            let name = format!("library{index}");
            let previous = format!("library{}", index.saturating_sub(1));
            let dependencies = if index == 0 {
                vec![]
            } else {
                vec![previous.as_str()]
            };
            artifacts.push(artifact(&name, &dependencies, false)?);
        }
        artifacts.reverse();
        let graph = Dependencies::new(
            artifacts,
            vec![Dependency::new(
                "last".into(),
                identity(&format!("library{}", width - 1))?,
            )?],
        )?;
        assert_eq!(graph.edges(), width - 1);
        let report = resolve("fn main(){}", Arc::new(graph))?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        assert_eq!(
            report
                .resolution
                .crates
                .as_ref()
                .ok_or("missing graph")?
                .functions()
                .count(),
            width
        );
        assert_eq!(analysis.facts.functions.len(), 1);
    }
    Ok(())
}

#[test]
fn explicit_foreign_deprecation_policy_requires_transport_without_changing_legacy_admission()
-> Result<(), Box<dyn std::error::Error>> {
    let legacy = resolve(
        "fn main(){let _=left::math::value(7);}",
        dependencies(false)?,
    )?;
    assert!(rustgm::pipeline::analyze_source(&legacy).is_ok());
    for policy in ["expect", "deny", "warn"] {
        let source = format!("#[{policy}(deprecated)]fn main(){{let _=left::math::value(7);}}");
        let report = resolve(&source, dependencies(false)?)?;
        let Err(error) = rustgm::pipeline::analyze_source(&report) else {
            return Err(format!("foreign {policy} policy inferred absence").into());
        };
        assert!(
            error
                .to_string()
                .contains("imported deprecation contract transport is not implemented"),
            "{error}"
        );
    }
    Ok(())
}
