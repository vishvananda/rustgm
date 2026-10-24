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
#[path = "qualified_source_rows.rs"]
mod captured;
#[path = "defined_original_carrier.rs"]
mod defining;
#[path = "ordinary_original_probe.rs"]
mod ordinary;
pub(super) use ordinary::observe as observe_ordinary;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}

#[test]
fn genuine_clone_from_and_identity_reach_paired_checked_original_inspection() -> Result {
    let (report, analysis, _) = analyzed_provider()?;
    let mut facts = analysis.facts;
    let concrete = source_function(&report, &facts, "clone")?;
    let ordinary = ordinary::Probe::new(concrete);
    let mut prepared = analysis.prepared;
    defining::required(&report, &facts, &mut prepared)?;
    inspect_selected(
        &report,
        &mut facts,
        prepared,
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
    ordinary.finish();
    Ok(())
}

fn analyzed_provider() -> Result<(Report, crate::pipeline::Analysis, CrateIdentity)> {
    let source = std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs"),
    )?;
    let provider = CrateIdentity::new("minicore", "source_headers")?;
    let context = Context {
        crate_name: Some("minicore".into()),
        current: Some(provider.clone()),
        mode: Mode::Source {
            provider: provider.clone(),
        },
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
    Ok((report, analysis, provider))
}

fn source_function(
    report: &Report,
    facts: &Declarations,
    name: &str,
) -> std::result::Result<crate::sema::model::Function, Error> {
    facts
        .functions
        .iter()
        .find(|function| {
            report
                .package
                .names
                .get(report.resolution.definitions[function.definition.0 as usize].name)
                == name
                && function.body.is_some()
        })
        .copied()
        .ok_or_else(|| diagnostic(report, None, "missing actual source function".into()))
}

fn paired_original(
    report: &Report,
    facts: &mut Declarations,
    checked: &borrow::templates::Checked<'_>,
    name: &str,
    destruct: DefId,
) -> std::result::Result<(), Error> {
    let function = source_function(report, facts, name)?;
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
        .ok_or_else(|| diagnostic(report, None, "missing actual original environment".into()))?;
    assert!(std::sync::Arc::ptr_eq(&template.environment, environment));
    trace_body(facts, name, template, contract);
    let mut conditional = false;
    let self_type = facts
        .associated
        .schema(function.definition)
        .map(|schema| schema.self_type);
    for clause in template
        .environment
        .trait_clauses()
        .map_err(|message| diagnostic(report, None, message.into()))?
    {
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
        defining::completed(template, contract, proof, checked)
            .map_err(|error| diagnostic(report, None, error.to_string()))?;
        assert!(
            conditional,
            "actual conditional Destruct remains an original input"
        );
    }
    trace_source_obligations(name, contract);
    trace_opening(report, facts, body)?;
    Ok(())
}

fn trace_body(
    facts: &Declarations,
    name: &str,
    template: &body::templates::Template,
    contract: &body::regions::Contract,
) {
    trace_shape(facts, name, &template.body, contract);
    trace_sidecars(name, &template.body);
    println!("{name} cleanup-sites={}", template.destruction.sites.len());
    for site in &template.destruction.sites {
        println!("{name} cleanup: {:?}", site.trigger);
    }
}

fn trace_sidecars(name: &str, body: &body::ir::Body) {
    println!(
        "{name} auxiliary guards={} guard-places={} scalars={} closure-calls={} resolved-callables={} promotion-state={:?}",
        body.guards.len(),
        body.guard_places.len(),
        body.scalars.len(),
        body.closure_calls.is_some(),
        body.resolved_callables.is_some(),
        body.promotions.as_ref().map(|facts| (
            facts.is_empty(),
            facts.entries().count(),
            facts.deferred().count(),
            facts.steps
        ))
    );
    if name == "clone_from" {
        assert!(
            body.promotions
                .as_ref()
                .is_some_and(body::promote::Facts::is_empty),
            "actual completed Clone analysis retains no promoted or deferred roots"
        );
    }
}

fn trace_shape(
    facts: &Declarations,
    name: &str,
    body: &body::ir::Body,
    contract: &body::regions::Contract,
) {
    println!(
        "original {name}: definition={} universals={} result={} locals={} projections={} statements={} blocks={}",
        match body.key() {
            body::ir::BodyKey::Declaration(definition) => definition.0,
            _ => unreachable!(),
        },
        contract.universals,
        body.result.0,
        body.locals.len(),
        body.projections.len(),
        body.statements.len(),
        body.blocks.len()
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
}

fn trace_source_obligations(name: &str, contract: &body::regions::Contract) {
    use body::regions::Kind;
    for obligation in &contract.obligations {
        let site = obligation.source;
        match obligation.kind {
            Kind::Variable {
                id,
                needs_provenance,
            } => println!("{name} goal {site:?}: variable {id:?} provenance={needs_provenance}"),
            Kind::Membership { region, captures } => {
                println!("{name} goal {site:?}: membership {region:?} {captures:?}");
            }
            Kind::LocalBorrow(region) => println!("{name} goal {site:?}: local-borrow {region:?}"),
            Kind::Equality(pair) => println!("{name} goal {site:?}: equality {pair:?}"),
            Kind::Outlives(pair) => println!("{name} goal {site:?}: outlives {pair:?}"),
            Kind::Coercion {
                actual,
                expected,
                unsize,
                reference,
            } => println!(
                "{name} goal {site:?}: coercion {actual:?} -> {expected:?} unsize={unsize} reference={reference:?}"
            ),
            Kind::WellFormed(ty) => println!("{name} goal {site:?}: formed {ty:?}"),
            Kind::ScopedWellFormed { ty, bound_regions } => {
                println!("{name} goal {site:?}: scoped-formed {ty:?} binders={bound_regions}");
            }
            Kind::TypeOutlives { ty, shorter } => {
                println!("{name} goal {site:?}: type-outlives {ty:?} {shorter:?}");
            }
        }
    }
    for proof in &contract.proofs {
        println!(
            "{name} proof-input {source:?}: regions={regions:?} equality={equalities:?} outlives={outlives:?} types={types:?} quantified={quantified}",
            source = proof.source,
            regions = proof.proof.owned_regions().collect::<Vec<_>>(),
            equalities = proof.proof.equalities,
            outlives = proof.proof.outlives,
            types = proof.proof.types,
            quantified = proof.proof.has_quantified()
        );
    }
    if let Some(builtins) = &contract.builtins {
        println!("{name} builtin-goals {:?}", builtins.predicates);
        if let Some(copy) = &builtins.source_copy {
            println!("{name} source-copy {:?} {:?}", copy.declaration, copy.goals);
        }
    }
}

fn trace_opening(
    report: &Report,
    facts: &mut Declarations,
    body: &body::ir::Body,
) -> std::result::Result<(), Error> {
    use crate::{
        body::ir::{Action, Rvalue},
        sema::model::{BodyRegionOrigin, Operation},
        types::{Kind, substitute::Bound},
    };
    let failure = |message: &str| diagnostic(report, Some(body.source.root_site()), message.into());
    let owner = body
        .source
        .source()
        .ok_or_else(|| failure("observer requires an actual source body"))?;
    for statement in &body.statements {
        let Action::Evaluate(value) = statement.action else {
            continue;
        };
        let Rvalue::Call {
            signature, callee, ..
        } = body.values[value.0 as usize].kind
        else {
            continue;
        };
        let site = body.source.proof_site(statement.source).map_err(failure)?;
        let key = site
            .source()
            .ok_or_else(|| failure("actual call has no source site"))?;
        let Some(Operation::Call {
            signature: published,
            source_use: Some(context),
        }) = facts.node(key).and_then(|node| node.operation)
        else {
            return Err(failure(
                "actual source Call has no completed owner/effect fact",
            ));
        };
        assert_eq!(signature, published);
        let effect = context.effect_for(facts, owner).map_err(failure)?;
        let arguments: Vec<_> = facts
            .body_regions
            .log()
            .iter()
            .filter(|row| {
                row.region.body == owner
                    && row.region.source == key
                    && matches!(row.region.origin, BodyRegionOrigin::CallArgument)
            })
            .map(|row| row.id)
            .collect();
        let Some(Kind::FunctionItem {
            signature: declared,
            ..
        }) = facts.types.kind(body.values[callee.0 as usize].ty)
        else {
            return Err(failure(
                "actual source call lost its function-item signature",
            ));
        };
        let Some(Kind::Function { bound_regions, .. }) = facts.types.kind(declared) else {
            return Err(failure("actual source call lost its declaration binder"));
        };
        assert_eq!(arguments.len(), bound_regions as usize);
        println!(
            "actual-call owner={owner:?} site={key:?} value={value:?} declaration={declared:?} opened={signature:?} effect={effect:?} exact-CallArgument-allocations={arguments:?}"
        );
        let mut fold = Bound::new(arguments);
        assert_eq!(
            fold.signature(&mut facts.types, declared, 0)
                .map_err(failure)?,
            signature
        );
    }
    Ok(())
}
