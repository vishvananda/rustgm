//! Failed protocol controls expose the actual ordinary goal and bounded proof trace.
use super::*;
pub(super) fn details(report: &Report) -> Result {
    let mut facts = crate::sema::declarations::analyze(report)?;
    let definition = root(report, &facts, "residual_into_try_type")?;
    let function = facts.functions[*facts
        .function_indices
        .get(&definition)
        .ok_or("actual function")?];
    let mut checker = crate::sema::bodies::Checker::new(report, &mut facts);
    let capture = crate::sema::copy_trace::Capture::new()?;
    let result = checker.function(function);
    eprintln!("actual ordinary residual body: {result:?}");
    for (index, (node, goal)) in checker.state.call_types.obligations.iter().enumerate() {
        let canonical = checker
            .state
            .infer
            .resolved_predicate(&mut checker.facts.types, goal);
        eprintln!(
            "residual goal {index} node={} trait={:?} builtin={:?} canonical={canonical:?}",
            node.get(),
            goal.trait_arguments(),
            goal.builtin_bound()
        );
        if let Ok(canonical) = canonical {
            let mut text = Vec::new();
            crate::sema::type_view::ty(&checker.facts.types, canonical.ty, &mut text)?;
            eprintln!("  subject {}", String::from_utf8(text)?);
            if let Some((application, _)) = canonical.bound.trait_parts() {
                let application = checker
                    .facts
                    .types
                    .trait_application(application)
                    .ok_or("actual application")?;
                let definition = &report.resolution.definitions[application.item.0 as usize];
                eprintln!(
                    "  trait {} args {:?}",
                    report.package.names.get(definition.name),
                    checker
                        .facts
                        .types
                        .arguments(application.arguments)
                        .ok_or("actual arguments")?
                        .collect::<Vec<_>>()
                );
            }
        }
    }
    if let Some(environment) = &checker.state.trait_environment {
        for clause in environment.trait_clauses()? {
            eprintln!("residual environment {clause:?}");
        }
    }
    for row in capture.rows()? {
        eprintln!("residual trace {row}");
    }
    Ok(())
}
