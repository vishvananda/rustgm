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
            source.replace("T:[const]Clone", "T:Clone")
        };
        for warm in [false, true] {
            let report = report(&source)?;
            let crate::pipeline::Analysis {
                mut facts,
                prepared,
            } = crate::pipeline::analyze_source(&report)?;
            let answer = definition(&report, &facts, "answer")?;
            let unused = definition(&report, &facts, "unused")?;
            let catalog = crate::lower::instances::inspect_prepared(
                &report,
                &mut facts,
                prepared,
                |_, checked| {
                    let mut catalog = const_eval::code::Catalog::default();
                    for body in &checked.program().bodies {
                        if matches!(body.owner,body::ir::Owner::Function{definition,..}if definition==unused)
                            && !warm
                        {
                            continue;
                        }
                        let caller = matches!(body.owner,body::ir::Owner::Function{definition,..}if definition==answer);
                        let mut snapshot = body.clone();
                        if caller {
                            snapshot.const_context = true;
                        }
                        let index = catalog.insert(&snapshot)?;
                        if caller {
                            catalog.definitions.insert(answer, index);
                        }
                    }
                    Ok::<_, &'static str>(catalog)
                },
            )??;
            let evaluator = crate::sema::values::Evaluator::new(&report, &mut facts);
            let result =
                const_eval::Machine::catalog(evaluator, &catalog, const_eval::Limits::default())
                    .definition(answer, &[]);
            if conditional {
                let error = result.err().ok_or("unpaid const execution accepted")?;
                assert!(
                    error.message.contains("missing demanded constant body"),
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
