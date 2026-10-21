//! Source growth must not turn capture-prefix discovery into pairwise comparison.
use super::semantic;
use std::fmt::Write;

fn source(width: usize, distinct: bool) -> Result<String, std::fmt::Error> {
    let mut source = String::from("fn main(){let x=(");
    for _ in 0..if distinct { width } else { 1 } {
        source.push_str("7u8,");
    }
    source.push_str(");let f=||{");
    for index in 0..width {
        write!(source, "let _=x.{}+1;", if distinct { index } else { 0 })?;
    }
    source.push_str("};f();}");
    Ok(source)
}

#[test]
fn capture_fields_and_repeated_uses_have_bounded_source_work()
-> Result<(), Box<dyn std::error::Error>> {
    for distinct in [false, true] {
        let mut previous = None;
        for width in [8, 32, 128] {
            let (_, facts) = semantic::analyze(&source(width, distinct)?)?;
            let closure = &facts.closures.all()[0];
            let environment = facts
                .closures
                .environment(closure.identity)
                .ok_or("environment")?;
            assert_eq!(
                environment.plan.captures().len(),
                if distinct { width } else { 1 }
            );
            assert_eq!(
                environment.plan.path_count(),
                if distinct { width + 1 } else { 2 }
            );
            let work = facts.body_work.inference_steps
                + facts.body_work.canonical_steps
                + facts.body_work.steps;
            if let Some(prior) = previous {
                assert!(
                    work <= prior * 5,
                    "distinct={distinct} width={width} work={work} prior={prior}"
                );
            }
            println!(
                "distinct={distinct} width={width} paths={} captures={} graph_work={} body_work={work}",
                environment.plan.path_count(),
                environment.plan.captures().len(),
                environment.plan.work
            );
            previous = Some(work);
        }
    }
    Ok(())
}
