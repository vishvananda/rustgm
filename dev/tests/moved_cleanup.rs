//! Proven initialization controls whether symbolic cleanup needs a type query.
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body,
    borrow::{destruction::symbolic, ownership::initialization},
    types::{copy::Copies, destruction::NeedsDrop},
};
use std::{fmt::Write as _, sync::Arc};

#[test]
fn moved_values_demand_no_recursive_cleanup_properties() -> Result<(), Box<dyn std::error::Error>> {
    for depth in [4, 16, 64] {
        for live in [false, true] {
            let mut source = String::from("struct S0<T>(T);");
            for level in 1..=depth {
                write!(source, "struct S{level}<T>(S{}<T>);", level - 1)?;
            }
            if live {
                write!(
                    source,
                    "fn probe<T>(x:S{depth}<T>,take:bool){{if take{{let y=x;}}}}"
                )?;
            } else {
                write!(source, "fn probe<T>(x:S{depth}<T>)->S{depth}<T>{{x}}")?;
            }
            rustc::qualify(&format!("cleanup-demand-{depth}-{live}"), &source, true)?;
            let (report, facts) = semantic::analyze(&source)?;
            let templates = body::templates::build(&report, &facts)?;
            let template = templates.bodies.first().ok_or("missing probe")?;
            let mut copies = Copies::in_environment(Arc::clone(&template.environment));
            let initialized = initialization::analyze(
                &facts.types,
                &facts.aggregates,
                &facts.derives,
                &mut copies,
                &template.body,
            )
            .map_err(|error| error.message)?;
            let mut needed = NeedsDrop::in_environment([], Arc::clone(&template.environment));
            symbolic::select(
                &facts.types,
                &facts.aggregates,
                &mut needed,
                &initialized,
                |_| false,
                |_, _| Err("unexpected field move"),
            )
            .map_err(|error| error.message)?;
            if live {
                assert!(needed.steps > depth, "live cleanup was not checked");
            } else {
                assert_eq!(needed.steps, 0, "moved value opened a type graph");
            }
        }
    }
    Ok(())
}
