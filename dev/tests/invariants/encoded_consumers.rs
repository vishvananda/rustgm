//! Complete encoded source families drive real fresh named executable proof.
use crate::{
    expand::config::Config,
    lower::instances::SourcePublication,
    module_packets::{self, Result},
    resolve::{
        bindings::Context,
        load::{InputFile, Sources},
    },
    support::{
        bindings::Mode,
        crates::{self, CrateIdentity, Dependencies, Dependency},
        source::SourceFile,
    },
};
use std::{path::Path, sync::Arc};
#[path = "decoded_contracts.rs"]
mod contracts;
#[path = "written_policies.rs"]
mod modifiers;

const HOLDER: &str = "\npub trait Holder<V>{fn carry<U>(value:U)->U{value}fn borrow(value:&V)->&V{value}}impl Holder<u8> for u8{}";
const CALLER: &str = r#"#![feature(no_core)]
#![no_core]
unsafe extern "C" {fn observe(value:u8);}
#[unsafe(no_mangle)]
pub extern "C" fn provided_observed()->u8 {
    let value=<u8 as base::Holder<u8>>::carry::<u8>(7u8);
    let borrowed=<u8 as base::Holder<u8>>::borrow(&value);
    *borrowed
}
fn main(){unsafe{observe(provided_observed());}}
"#;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn provider(extra: &str) -> Result<(String, crates::Interface)> {
    let mut source = crate::test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    source.push_str(HOLDER);
    source.push_str(extra);
    let artifact = module_packets::exported_profile(
        &source,
        SourcePublication::ProvidedContracts,
        "provided_contracts",
    )?;
    Ok((source, artifact))
}
fn context(artifact: crates::Interface) -> Result<Context> {
    let provider = artifact.identity().clone();
    Ok(Context {
        crate_name: Some("encoded_consumer".into()),
        current: Some(CrateIdentity::new(
            "encoded_consumer",
            "provided_contracts",
        )?),
        dependencies: Some(Arc::new(Dependencies::new(
            vec![artifact],
            vec![Dependency::new("base".into(), provider.clone())?],
        )?)),
        mode: Mode::SourceClient { provider },
    })
}
fn client<R>(
    artifact: crates::Interface,
    source: &str,
    inspect: impl FnOnce(
        &crate::resolve::expansion::Report,
        &crate::sema::model::Declarations,
        &crate::lowir::program::Program,
    ) -> R,
) -> Result<R> {
    let context = context(artifact)?;
    Ok(crate::lower::instances::with_provided_client(
        crate::resolve::crates::ConsumerRequest::new(
            InputFile {
                identity: "encoded-consumer.rs".into(),
                path: "encoded-consumer.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
            &context,
        ),
        inspect,
    )?)
}
#[test]
fn encoded_full_family_reloads_named_carry_borrow_and_fresh_instances() -> Result {
    let (source, artifact) = provider("")?;
    let bytes = crates::encode(&artifact)?;
    assert_eq!(bytes.get(12..15), Some([11, 1, 0].as_slice()));
    let decoded = crates::decode(&bytes)?;
    assert_eq!(decoded, artifact);
    assert_eq!(crates::encode(&decoded)?, bytes);
    let descriptor = decoded.source_descriptor().ok_or("decoded descriptor")?;
    let rows = descriptor.rows();
    assert!(rows.functions.iter().any(|row| {
        rows.declarations[row.declaration.0 as usize]
            .name
            .as_deref()
            == Some("clone_from")
    }));
    let lowir = client(
        decoded,
        CALLER,
        |report, facts, program| -> Result<Vec<u8>> {
            assert_eq!(
                report.package.units.len(),
                1,
                "provider source was parsed in consumer"
            );
            let imported = report.resolution.crates.as_deref().ok_or("fresh catalog")?;
            let linkage = facts.crates.as_deref().ok_or("fresh linkage")?;
            let catalog = linkage.origin_catalog().ok_or("actual catalog")?;
            let mut instances = 0;
            for definition in imported.generic_definitions() {
                let definition = definition?;
                let actual = &report.resolution.definitions[definition.0 as usize];
                assert!(actual.node.is_none());
                let name = report.package.names.get(actual.name);
                if name == "clone_from" {
                    assert!(
                        !matches!(facts.function_header(definition), Ok(Some(_))),
                        "unrequested default body was eagerly proved"
                    );
                }
                if name == "clone" {
                    let policy = facts.must_use_contract(report, definition)?;
                    // The concrete override has no marker; the actual trait method does.
                    if let Some(policy) = policy {
                        assert_eq!(
                            policy.reason,
                            Some(
                                "cloning is often expensive and is not expected to have side effects"
                            )
                        );
                        assert!(policy.origin.imported().is_some());
                    }
                }
                if !matches!(name, "carry" | "borrow") {
                    continue;
                }
                let header = facts
                    .function_header(definition)?
                    .ok_or("demanded actual body header")?;
                assert_eq!(
                    header.declaration,
                    crate::resolve::body_origin::DeclarationOrigin::Imported(
                        catalog.declaration(definition)?
                    )
                );
                assert!(header.environment.is_some());
                let symbol = linkage
                    .rust_symbol(definition)
                    .ok_or("actual method symbol")?;
                let prefix = format!("{symbol}.i");
                assert_eq!(
                    program
                        .symbols
                        .iter()
                        .filter(|symbol| symbol.body.is_some() && symbol.name.starts_with(&prefix))
                        .count(),
                    1,
                    "{name}"
                );
                instances += 1;
            }
            assert_eq!(instances, 2);
            let mut lowir = Vec::new();
            crate::lowir::writer::write(program, &mut lowir)?;
            Ok(lowir)
        },
    )??;
    let output = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/all-attribute-selected-output-v1/provided-contracts-native");
    std::fs::create_dir_all(&output)?;
    std::fs::write(
        output.join(".keep-artifacts"),
        "Encoded full-family native7 inputs\n",
    )?;
    std::fs::write(output.join("provider.rs"), source)?;
    std::fs::write(output.join("caller.rs"), CALLER)?;
    std::fs::write(output.join("provider.rgcrate"), bytes)?;
    std::fs::write(output.join("consumer.lowir"), lowir)?;
    Ok(())
}

#[test]
fn encoded_calls_reject_real_wrong_type_arity_trait_and_dead_borrow() -> Result {
    let (_, artifact) = provider("")?;
    let bytes = crates::encode(&artifact)?;
    for source in [
        "fn main(){let _=<u8 as base::Holder<u8>>::carry::<u8>(true);}",
        "fn main(){let _=<u8 as base::Holder<u8>>::carry::<u8>();}",
        "fn main(){let _=<u8 as base::Holder<bool>>::carry::<u8>(7u8);}",
        "fn main(){let borrowed;{let value=7u8;borrowed=<u8 as base::Holder<u8>>::borrow(&value);}let _=*borrowed;}",
    ] {
        let source = format!("#![feature(no_core)]#![no_core]{source}");
        let Err(error) = client(crates::decode(&bytes)?, &source, |_, _, _| ()) else {
            return Err("malformed named call accepted".into());
        };
        let message = error.to_string();
        for setup in [
            "header budget",
            "missing value type",
            "no such method",
            "pending",
            "not implemented",
            "unsupported",
        ] {
            assert!(
                !message.contains(setup),
                "wrong call failed at setup instead of its argument/region obligation: {message}"
            );
        }
        eprintln!("actual encoded call rejection: {error}");
    }
    Ok(())
}
