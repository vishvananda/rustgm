//! A warm code snapshot cannot grant an unpaid source const-call permission.
use super::*;
use crate::provider_cases as cases;
use crate::resolve::{
    bindings::Context,
    load::{InputFile, Sources},
};
use crate::support::{crates::CrateIdentity, source::SourceFile};
use crate::{body, const_eval, expand::config::Config, resolve, support::bindings::Mode};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str) -> Result<resolve::expansion::Report> {
    let identity = CrateIdentity::new("minicore", "source_const_call_test")?;
    Ok(resolve::analyze_expanded(
        resolve::expansion::analyze_with_context(
            InputFile {
                identity: "provider.rs".into(),
                path: "provider.rs".into(),
                source: SourceFile::new(source.as_bytes().to_vec())?,
            },
            &mut NoFiles,
            &Config::default(),
            &Context {
                crate_name: Some("minicore".into()),
                current: Some(identity.clone()),
                mode: Mode::Source { provider: identity },
                ..Default::default()
            },
        )?,
    )?)
}
fn definition(
    report: &resolve::expansion::Report,
    facts: &Declarations,
    name: &str,
) -> Result<DefId> {
    facts
        .functions
        .iter()
        .find(|f| {
            report
                .package
                .names
                .get(report.resolution.definitions[f.definition.0 as usize].name)
                == name
        })
        .map(|f| f.definition)
        .ok_or_else(|| format!("missing {name}").into())
}
fn execution_bodies(
    report: &resolve::expansion::Report,
    conditional: bool,
) -> Result<(Declarations, Vec<body::ir::Body>)> {
    if conditional {
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = crate::pipeline::analyze_source(report)?;
        let bodies = crate::lower::instances::inspect_prepared(
            report,
            &mut facts,
            prepared,
            |_, checked| checked.program().bodies.clone(),
        )?;
        return Ok((facts, bodies));
    }
    // Observe the genuine Code-purpose bodies demanded by the source constant.
    // Native instances intentionally carry runtime context and cannot substitute
    // for these separately proved constant-execution bodies.
    let targets = ["answer", "unused"].map(|name| {
        report
            .resolution
            .definitions
            .iter()
            .position(|definition| {
                definition.kind == resolve::model::DefKind::Function
                    && report.package.names.get(definition.name) == name
            })
            .and_then(|index| u32::try_from(index).ok())
            .map(DefId)
            .ok_or("actual source const execution target")
    });
    let [answer, unused] = targets;
    let (analysis, bodies) =
        crate::pipeline::execution::materialized_bodies::during(&[answer?, unused?], || {
            crate::pipeline::analyze_source(report)
        })?;
    assert!(bodies.iter().all(|body| body.const_context));
    Ok((analysis.facts, bodies))
}
#[test]
fn source_const_call_gate_rejects_cold_and_warm_invalid_execution_attempts() -> Result {
    // This is an adversarial CTFE attempt on an independently accepted *runtime*
    // caller. The forged constant-context bit below bypasses its outer entry
    // check to exercise the reached-call gate; it grants no source permission.
    let source = cases::UNUSED_CONDITIONAL_RUNTIME;
    for conditional in [true, false] {
        let source = if conditional {
            source.to_owned()
        } else {
            let ordinary = source.replace("T:[const]Clone", "T:Clone").replace(
                "pub extern \"C\" fn answer",
                "pub const extern \"C\" fn answer",
            );
            format!("{ordinary}\nconst OBSERVED:u8=answer();\n")
        };
        for warm in [false, true] {
            let report = report(&source)?;
            let (mut facts, bodies) = execution_bodies(&report, conditional)?;
            let answer = definition(&report, &facts, "answer")?;
            let unused = definition(&report, &facts, "unused")?;
            let mut catalog = const_eval::code::Catalog::default();
            for mut body in bodies {
                if matches!(body.owner,body::ir::Owner::Function{definition,..}if definition==unused)
                    && !warm
                {
                    continue;
                }
                let caller = matches!(body.owner,body::ir::Owner::Function{definition,..}if definition==answer);
                if caller && conditional {
                    body.const_context = true;
                }
                let index = catalog.insert(&body)?;
                if caller {
                    catalog.definitions.insert(answer, index);
                }
            }
            let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
            let result =
                const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default())
                    .definition(answer, &[]);
            if conditional {
                let error = result.err().ok_or("unpaid const execution accepted")?;
                assert!(
                    error
                        .message
                        .contains("source function root differs from its actual const owner"),
                    "warm={warm}: {}",
                    error.message
                );
            } else if warm {
                assert_eq!(
                    result.map_err(|e| e.message)?.as_scalar()?,
                    crate::sema::model::Scalar::Integer(7)
                );
            } else {
                assert!(result.is_err(), "missing ordinary target code fabricated");
            }
        }
    }
    Ok(())
}
#[test]
fn source_const_call_effects_include_actual_inherited_clauses_and_keep_pending_distinct() -> Result
{
    let source = cases::INHERITED_CONDITIONAL_HEADER;
    let report = report(source)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let member = definition(&report, &facts, "seven")?;
    assert_eq!(declared_effect(&mut facts, member)?, Effect::Conditional);
    // The raw default method is not yet an independently const-eligible target.
    let catalog = facts.source_bindings.as_deref().ok_or("catalog")?;
    assert!(catalog.check_const_call(member).is_err());
    assert!(catalog.check_const_call(DefId(u32::MAX)).is_err());
    println!(
        "SourceCatalog={} SourceCallEffect={}",
        size_of::<Catalog>(),
        size_of::<Effect>()
    );
    Ok(())
}

#[test]
fn missing_prepared_callable_effect_remains_pending_after_real_header_completion() -> Result {
    use crate::types::{
        ItemId, Kind,
        infer::{Context as Infer, Instantiation},
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    };
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa31/tests/spec/source-const-payment/callable/const-function-item.rs"),
    )?;
    let report = report(&source)?;
    let mut facts = crate::pipeline::analyze_source(&report)?.facts;
    let item = definition(&report, &facts, "item")?;
    let ty = facts.definitions[item.0 as usize].ok_or("item")?;
    let role = facts
        .source_bindings
        .as_deref()
        .ok_or("catalog")?
        .validated_language(crate::support::bindings::LangItem::FnOnce)?
        .ok_or("role")?;
    let list = facts.types.intern_list(&[])?;
    let unit = facts.types.intern(Kind::Tuple(list))?;
    let arguments = facts
        .types
        .intern_arguments(&[crate::types::arguments::Argument::Type(unit)])?;
    let application = facts.types.intern_trait_application(Application {
        item: ItemId(role.definition.0),
        arguments,
    })?;
    let predicate = Predicate {
        ty,
        bound: Bound::ConstTrait(application),
    };
    let previous = facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .call_effects
        .as_deref_mut()
        .ok_or("effects")?
        .remove(&item)
        .ok_or("prepared")?;
    let mut context = Infer::default();
    let mut fold = Instantiation::new(&mut context, &facts.types, &[])?;
    let goal = context.instantiate_predicate(&mut facts.types, predicate, &mut fold)?;
    let selected = crate::sema::traits::selection::select(
        &mut facts,
        &mut context,
        &goal,
        crate::sema::traits::selection::Caller {
            assumptions: &crate::sema::traits::assumptions::Assumptions::default(),
            environment: None,
            first_region: 1,
            retain_sized: false,
            placeholders: None,
        },
    )?;
    assert!(matches!(
        selected,
        crate::sema::traits::selection::Selection::Deferred
    ));
    facts
        .source_bindings
        .as_deref_mut()
        .ok_or("catalog")?
        .call_effects
        .as_deref_mut()
        .ok_or("effects")?
        .insert(item, previous);
    let selected = crate::sema::traits::selection::select(
        &mut facts,
        &mut context,
        &goal,
        crate::sema::traits::selection::Caller {
            assumptions: &crate::sema::traits::assumptions::Assumptions::default(),
            environment: None,
            first_region: 1,
            retain_sized: false,
            placeholders: None,
        },
    )?;
    assert!(matches!(
        selected,
        crate::sema::traits::selection::Selection::Proven(_)
    ));
    Ok(())
}
