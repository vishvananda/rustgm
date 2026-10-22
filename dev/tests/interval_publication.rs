//! Deferred pattern values publish before any dependent checked CFG is captured.
use crate::native_references as native;
use crate::resolve as source;
use rustgm::{pipeline, sema::model::Scalar};
use std::{path::Path, process::Command};

const CASES: &[(&str, &str, bool)] = &[
    (
        "direct",
        "const N:u8=7;const C:u8=match 7u8{N=>1,_=>2};fn main(){check(match 1u8{C=>true,_=>false});}",
        false,
    ),
    (
        "forward",
        "const C:u8=match 7u8{N=>1,_=>2};const N:u8=7;fn main(){check(match 1u8{C=>true,_=>false});}",
        false,
    ),
    (
        "main-first",
        "fn main(){check(match 1u8{C=>true,_=>false});}const C:u8=match 7u8{N=>1,_=>2};const N:u8=7;",
        false,
    ),
    (
        "nested",
        "const N:u8=7;const C:u8=match 7u8{N=>match 7u8{N=>1,_=>3},_=>2};fn main(){check(match 1u8{C=>true,_=>false});}",
        false,
    ),
    (
        "range",
        "const LO:u8=6;const HI:u8=8;const C:u8=match 7u8{LO..=HI=>1,_=>2};fn main(){check(match 1u8{C=>true,_=>false});}",
        false,
    ),
    (
        "callee",
        "const N:u8=7;const fn f()->u8{match 7u8{N=>1,_=>2}}const C:u8=f();fn main(){check(match 1u8{C=>true,_=>false});}",
        false,
    ),
    (
        "repeated",
        "const N:u8=7;const C:u8=match 7u8{N=>1,_=>2};fn f()->bool{match 1u8{C=>true,_=>false}}fn main(){check(f()&&match 1u8{C=>true,_=>false});}",
        false,
    ),
    (
        "chain",
        "const C:u8=match 1u8{N=>1,_=>2};const N:u8=match 7u8{M=>1,_=>2};const M:u8=7;fn main(){check(match 1u8{C=>true,_=>false});}",
        false,
    ),
];

#[test]
fn selected_scopes_publish_before_dependent_constant_queries() -> Result<(), String> {
    for &(name, text, _) in CASES {
        let report = source::resolve(&format!("{}{text}", native::CHECK), true)?;
        let analysis =
            pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        let definition = report
            .resolution
            .definitions
            .iter()
            .enumerate()
            .find(|(_, definition)| report.package.names.get(definition.name) == "C")
            .ok_or("missing C definition")?
            .0;
        let value = rustgm::sema::values::Reader::new(&analysis.facts)
            .definition(rustgm::resolve::model::DefId(
                u32::try_from(definition).map_err(|_| "definition overflow")?,
            ))
            .ok_or("missing selected C value")?
            .as_scalar()
            .map_err(|e| e.to_string())?;
        assert!(
            matches!(value, Scalar::Integer(1)),
            "{name}: wrong selected arm"
        );
        assert!(analysis.facts.patterns.pending_bodies.is_empty(), "{name}");
    }
    Ok(())
}

#[test]
#[ignore = "requires the configured CPPGM native backend"]
fn selected_pattern_values_match_independent_native_results()
-> Result<(), Box<dyn std::error::Error>> {
    native::check_cases("parallel/pa25-deferred-patterns", CASES)
}

fn object_outcome(
    name: &str,
    text: &str,
    accepted: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa25-pattern-outcomes/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let path = folder.join(format!("{name}.rs"));
    std::fs::write(&path, text)?;
    let result = Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-name=pattern_outcome",
            "--emit=obj",
            "-o",
        ])
        .arg(folder.join(format!("{name}.o")))
        .arg(&path)
        .output()?;
    std::fs::write(folder.join(format!("{name}.rustc.stderr")), &result.stderr)?;
    assert_eq!(
        result.status.code(),
        Some(i32::from(!accepted)),
        "{name}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn cross_body_patterns_complete_before_cfg_capture() -> Result<(), Box<dyn std::error::Error>> {
    let text = "const N:u8=7;const C:u8=f();fn main(){let _=match 1u8{C=>true,_=>false};}const fn f()->u8{match 7u8{N=>1,_=>2}}";
    object_outcome("valid-cross-body-boundary", text, true)?;
    let report = source::resolve(text, true)?;
    let analysis = pipeline::analyze_source(&report)?;
    assert!(analysis.facts.patterns.pending_bodies.is_empty());
    let cycle = "const C:u8=match 7u8{N=>1,_=>2};const N:u8=match 7u8{C=>1,_=>2};fn main(){let _=match 1u8{C=>true,_=>false};}";
    object_outcome("true-pattern-cycle", cycle, false)?;
    let report = source::resolve(cycle, true)?;
    let Err(error) = pipeline::analyze_source(&report) else {
        return Err("pattern dependency cycle accepted".into());
    };
    assert!(
        error
            .to_string()
            .contains("cycle in constant pattern dependencies"),
        "{error}"
    );
    Ok(())
}

#[test]
fn repeated_scope_dependencies_have_linear_graph_visits() -> Result<(), String> {
    for width in [8, 32, 128] {
        use std::fmt::Write;
        let mut text = String::from("const N:u8=7;const C:u8=match 7u8{N=>1,_=>2};");
        for index in 0..width {
            write!(text, "fn f{index}()->bool{{match 1u8{{C=>true,_=>false}}}}")
                .map_err(|e| e.to_string())?;
        }
        let report = source::resolve(&text, true)?;
        let facts = pipeline::analyze_source(&report)
            .map_err(|e| e.to_string())?
            .facts;
        assert_eq!(facts.patterns.completion_work, 5 * width + 3);
        assert!(facts.patterns.pending_bodies.is_empty());
        assert_eq!(facts.constant_work.expressions, 1);
    }
    Ok(())
}

#[test]
fn dynamic_scope_work_and_repeated_value_demands_reuse_completed_roots()
-> Result<(), Box<dyn std::error::Error>> {
    use std::fmt::Write;
    let mut observations = Vec::new();
    for width in [8, 32, 128] {
        let mut text = String::from("const N:isize=7;enum E{");
        for index in 0..width {
            write!(text, "V{index}=match 7isize{{N=>{index},_=>1000}},")?;
        }
        text.push('}');
        for index in 0..width {
            write!(text, "const C{index}:isize=E::V{index} as isize;")?;
        }
        let report = source::resolve(&text, true)?;
        let mut facts = pipeline::analyze_source(&report)?.facts;
        assert!(facts.patterns.pending_bodies.is_empty());
        let graph = facts.patterns.completion_work;
        let expressions = facts.constant_work.expressions;
        let values = facts
            .initializers
            .iter()
            .map(|initializer| {
                let definition = initializer.definition;
                rustgm::sema::values::Reader::new(&facts)
                    .definition(definition)
                    .map(|value| (definition, value))
                    .ok_or("missing completed scalar")
            })
            .collect::<Result<Vec<_>, _>>()?;
        {
            let mut evaluator = rustgm::sema::values::Evaluator::new(&report, &mut facts);
            for _ in 0..3 {
                for &(definition, value) in &values {
                    assert!(evaluator.definition(definition)? == value);
                }
            }
        }
        assert_eq!(facts.patterns.completion_work, graph);
        assert_eq!(facts.constant_work.expressions, expressions);
        assert_eq!(expressions, width);
        observations.push((width, graph, facts.constant_work.discriminant_visits));
    }
    println!("dynamic pattern scope graph/entry visits: {observations:?}");
    assert_eq!(observations[1].1 - observations[0].1, 3 * (32 - 8));
    assert_eq!(observations[2].1 - observations[1].1, 3 * (128 - 32));
    Ok(())
}
