//! Observe real paired checked originals; no source reconstruction or wire recipe.
use super::*;
use crate::{
    expand::config::Config,
    resolve::{
        self,
        bindings::Context,
        load::{InputFile, Sources},
    },
    support::{
        bindings::{LangItem, Mode},
        crates::CrateIdentity,
        source::SourceFile,
    },
    types::{
        ItemId,
        parameters::{clauses::Clause, predicates::Bound},
    },
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn genuine_clone_from_and_identity_reach_paired_checked_original_inspection() -> Result {
    let source = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs"),
    )?;
    let provider = CrateIdentity::new("minicore", "source_headers")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(provider.clone()),
        mode: Mode::Source { provider },
        ..Default::default()
    };
    let report = resolve::analyze_expanded(resolve::expansion::analyze_with_context(
        InputFile {
            identity: "provider.rs".into(),
            path: "provider.rs".into(),
            source: SourceFile::new(source)?,
        },
        &mut NoFiles,
        &Config::default(),
        &context,
    )?)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    inspect_selected(
        &report,
        &mut facts,
        analysis.prepared,
        |facts, checked| {
            let catalog = facts
                .source_bindings
                .as_deref()
                .ok_or_else(|| diagnostic(&report, None, "missing source roles".into()))?;
            let destruct = catalog
                .language(LangItem::Destruct)
                .ok_or_else(|| diagnostic(&report, None, "missing actual Destruct".into()))?
                .definition;
            for name in ["clone_from", "identity"] {
                paired_original(&report, facts, checked, name, destruct)?;
            }
            let exports = super::super::roots::exports(facts, &[])
                .map_err(|error| diagnostic(&report, None, error.into()))?;
            Ok((None, exports))
        },
        |_, _, _| (),
    )?;
    Ok(())
}

fn paired_original(
    report: &Report,
    facts: &Declarations,
    checked: &borrow::templates::Checked<'_>,
    name: &str,
    destruct: DefId,
) -> std::result::Result<(), Error> {
    let function = facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
        })
        .ok_or_else(|| diagnostic(report, None, "missing actual source function".into()))?;
    let (template, contract, proof) = checked
        .function_proof(ItemId(function.definition.0))
        .ok_or_else(|| diagnostic(report, None, "missing actual paired original proof".into()))?;
    let body = &template.body;
    assert!(body.instance.is_none());
    assert_eq!(body.key(), contract.key);
    assert_eq!(body.source, contract.source);
    assert_eq!(body.source, proof.source);
    assert_eq!(contract.signature, Some(function.signature));
    assert!(matches!(body.owner, body::ir::Owner::Function {
        definition, signature, execution: None,
    } if definition == function.definition && signature == function.signature));
    let environment = facts
        .generics
        .body_environment(function.definition)
        .ok_or_else(|| diagnostic(report, None, "missing exact original environment".into()))?;
    assert!(std::sync::Arc::ptr_eq(&template.environment, environment));
    trace_body(facts, name, template, contract);
    let mut conditional = false;
    let self_type = facts
        .associated
        .schema(function.definition)
        .map(|schema| schema.self_type);
    for clause in template.environment.trait_clauses() {
        println!("{name} original-environment {clause:?}");
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            Clause::ForAll(id) => {
                facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| {
                        diagnostic(report, None, "invalid retained quantified premise".into())
                    })?
                    .predicate
            }
        };
        if let Bound::MaybeConstTrait(id) = predicate.bound {
            conditional |= Some(predicate.ty) == self_type
                && facts
                    .types
                    .trait_application(id)
                    .is_some_and(|application| {
                        application.item.0 == destruct.0
                            && facts
                                .types
                                .arguments(application.arguments)
                                .is_some_and(|arguments| arguments.len() == 0)
                    });
        }
    }
    if name == "clone_from" {
        assert!(
            conditional,
            "actual conditional Destruct remains an original input"
        );
    }
    Ok(())
}

fn trace_body(
    facts: &Declarations,
    name: &str,
    template: &body::templates::Template,
    contract: &body::regions::Contract,
) {
    let body = &template.body;
    println!(
        "original {name}: definition={} universals={} result={} locals={} projections={} statements={} blocks={} cleanup-sites={}",
        match body.key() {
            body::ir::BodyKey::Declaration(definition) => definition.0,
            _ => unreachable!(),
        },
        contract.universals,
        body.result.0,
        body.locals.len(),
        body.projections.len(),
        body.statements.len(),
        body.blocks.len(),
        template.destruction.sites.len()
    );
    for (index, local) in body.locals.iter().enumerate() {
        println!(
            "{name} local{index}: type={} {:?} kind={:?} definition={:?}",
            local.ty.0,
            facts.types.kind(local.ty),
            local.kind,
            local.definition
        );
    }
    for (index, projection) in body.projections.iter().enumerate() {
        println!(
            "{name} projection{index}: type={} {:?} base={:?} {:?}",
            projection.ty.0,
            facts.types.kind(projection.ty),
            projection.base,
            projection.kind
        );
    }
    for (index, value) in body.values.iter().enumerate() {
        println!(
            "{name} value{index}: type={} {:?} {:?}",
            value.ty.0,
            facts.types.kind(value.ty),
            value.kind
        );
    }
    for (index, statement) in body.statements.iter().enumerate() {
        println!("{name} statement{index}: {:?}", statement.action);
    }
    for (index, block) in body.blocks.iter().enumerate() {
        println!(
            "{name} block{index}: {}..{} {:?}",
            block.start, block.end, block.terminator
        );
    }
    println!(
        "{name} contract: signature={:?} parameters={:?} outlives={:?} type-outlives={:?} proof-count={} obligation-count={}",
        contract.signature,
        contract.parameters,
        contract.assumptions,
        contract.type_assumptions,
        contract.proofs.len(),
        contract.obligations.len()
    );
    for site in &template.destruction.sites {
        println!("{name} cleanup: {:?}", site.trigger);
    }
}
