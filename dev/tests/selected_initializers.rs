//! Selected initializer bodies retain exact arguments and independent checked execution.
use crate::checked_sources;
use crate::native_references as native;
#[path = "selected_initializers/callers.rs"]
mod callers;
#[path = "selected_initializers/canonical.rs"]
mod canonical;
#[path = "selected_initializers/closed.rs"]
mod closed;
#[path = "selected_initializers/computed.rs"]
mod computed;
#[path = "selected_initializers/open.rs"]
mod open;
#[path = "selected_initializers/paid.rs"]
mod paid;
#[path = "helpers/objects.rs"]
mod rustc;
#[path = "selected_initializers/type_arguments.rs"]
mod type_arguments;
use crate::resolve as source;
const CASES: &[(&str, bool)] = &[
    ("statements", true),
    ("shared-reference", true),
    ("dangling", false),
    ("projected-result", true),
    ("erased-type", true),
    ("forward-callee", true),
    ("const-machine", true),
    ("distinct-recursion", false),
    ("nested-inline", true),
    ("static-suspension", true),
    ("qualified-trait", true),
    ("cycle", false),
    ("fault", false),
    ("unused-nonconst", false),
    ("skipped-nonconst", false),
    ("mutable-static", false),
    ("cross-definition", true),
];
fn fixture(name: &str) -> Result<String, std::io::Error> {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa25/tests/spec/associated-execution/{name}.rs")),
    )
}
#[test]
fn independently_qualified_selected_initializer_outcomes() -> Result<(), Box<dyn std::error::Error>>
{
    for &(name, accepted) in CASES {
        let text = format!("{}fn main(){{let _=run();}}", fixture(name)?);
        rustc::qualify(
            &format!("parallel/pa25-associated/qualification/{name}"),
            &text,
            accepted,
            "lib",
        )?;
    }
    Ok(())
}
#[test]
fn selected_initializer_cfgs_reach_checked_lowering() -> Result<(), Box<dyn std::error::Error>> {
    let mut errors = Vec::new();
    for &(name, accepted) in CASES {
        let text = format!("{}fn main(){{let _=run();}}", fixture(name)?);
        let mut report = source::resolve(&text, true)?;
        let result = rustgm::pipeline::analyze_source(&report).and_then(|analysis| {
            let mut facts = analysis.facts;
            let entry = rustgm::sema::entry::select(&mut report, &facts)?;
            rustgm::lower::instances::build_prepared(&report, &mut facts, entry, analysis.prepared)
        });
        let diagnostic = result.as_ref().err().map(ToString::to_string);
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../obj/parallel/pa25-associated/candidate/{}/{name}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("source.rs"), &text)?;
        std::fs::write(folder.join("stderr"), diagnostic.as_deref().unwrap_or(""))?;
        if !accepted && let Some(diagnostic) = &diagnostic {
            let reason = match name {
                "fault" => diagnostic.contains("zero") && diagnostic.contains("div"),
                "cycle" | "distinct-recursion" => diagnostic.contains("cycl"),
                "mutable-static" => diagnostic.contains("mutable"),
                "dangling" => {
                    diagnostic.contains("lifetime")
                        || diagnostic.contains("outlive")
                        || diagnostic.contains("expired")
                }
                _ => diagnostic.contains("non-const"),
            };
            if !reason {
                errors.push(format!("{name}: wrong rejection: {diagnostic}"));
            }
        }
        if result.is_ok() != accepted {
            errors.push(format!("{name}: {:?}", result.err()));
        }
    }
    assert!(errors.is_empty(), "{}", errors.join("\n"));
    Ok(())
}
#[test]
#[ignore = "requires qualified CPPGM native backend"]
fn selected_initializer_values_execute() -> Result<(), Box<dyn std::error::Error>> {
    for &(name, accepted) in CASES {
        if accepted {
            let text = format!("{}fn main(){{check(run());}}", fixture(name)?);
            native::check_cases("parallel/pa25-associated/native", &[(name, &text, false)])?;
        }
    }
    Ok(())
}

#[test]
fn selected_initializer_preparation_is_shared_across_complete_instances()
-> Result<(), Box<dyn std::error::Error>> {
    let mut template_bytes = None;
    for count in [16usize, 64, 256] {
        use std::fmt::Write;
        let mut text = String::from(
            "struct S<const N:usize>;impl<const N:usize>S<N>{const C:usize={let n=N;n+1};}fn main(){",
        );
        for n in 0..count {
            write!(text, "let _=S::<{n}>::C;let _=S::<{n}>::C;")?;
        }
        text.push('}');
        let mut report = source::resolve(&text, true)?;
        let rustgm::pipeline::Analysis {
            mut facts,
            prepared,
        } = rustgm::pipeline::analyze_source(&report)?;
        let entry = rustgm::sema::entry::select(&mut report, &facts)?;
        rustgm::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
        assert_eq!(facts.constant_work.associated_templates, 1);
        assert_eq!(facts.constant_work.associated_instances, count);
        let bytes = facts.constant_work.template_bytes;
        if let Some(previous) = template_bytes {
            assert_eq!(bytes, previous);
        } else {
            template_bytes = Some(bytes);
        }
        assert!(facts.constant_work.code_bytes <= count * 8192);
        assert!(facts.constant_work.associated_value_bytes <= count * 512);
        for initializer in &facts.initializers {
            assert!(
                rustgm::sema::values::Reader::new(&facts)
                    .definition(initializer.definition)
                    .is_none()
            );
        }
    }
    Ok(())
}

#[test]
fn associated_inventory_retains_instance_types_without_running_queries()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa25-associated/inspection/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join("instances.rs");
    std::fs::write(&input, fixture("const-machine")?)?;
    let mut observations = Vec::new();
    for n in 0..2 {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .arg("--emit-constants")
            .arg(&input)
            .output()?;
        std::fs::write(folder.join(format!("{n}.stdout")), &output.stdout)?;
        std::fs::write(folder.join(format!("{n}.stderr")), &output.stderr)?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        observations.push(output.stdout);
    }
    assert_eq!(observations[0], observations[1]);
    let view = std::str::from_utf8(&observations[0])?;
    let roots = view
        .lines()
        .filter(|line| line.starts_with("root associated-instance "))
        .collect::<Vec<_>>();
    assert_eq!(roots.len(), 2);
    assert_ne!(roots[0], roots[1]);
    assert!(view.lines().any(|line|line.contains("kind=const")&&line.contains("status=deferred-initializer")));
    Ok(())
}

#[test]
#[ignore = "requires qualified CPPGM native backend"]
fn symbolic_selected_values_execute_after_caller_specialization()
-> Result<(), Box<dyn std::error::Error>> {
    for (name, path) in [
        (
            "caller-lifetime",
            "selection/lookup-readiness/semantics/trait-caller-lifetime.rs",
        ),
        (
            "erased-type",
            "constants/pattern/semantics/erased-type-called.rs",
        ),
    ] {
        let text = std::fs::read_to_string(
            std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../pa20/tests/spec")
                .join(path),
        )?;
        let text = text
            .strip_prefix(native::CHECK)
            .ok_or("fixture lacks native check prefix")?;
        native::check_cases(
            "parallel/pa25-symbolic-selection/native",
            &[(name, text, false)],
        )?;
    }
    Ok(())
}

#[test]
fn source_completed_inventory_defers_unmaterialized_results()
-> Result<(), Box<dyn std::error::Error>> {
    let input = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../pa20/tests/spec/constants/pattern/semantics/erased-type-called.rs");
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-constants")
        .arg(input)
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::str::from_utf8(&output.stdout)?;
    let rows = text
        .lines()
        .filter(|row| row.starts_with("associated-value "))
        .collect::<Vec<_>>();
    assert!(!rows.is_empty());
    assert!(
        rows.iter()
            .all(|row| row.ends_with("status=source-complete result=deferred"))
    );
    assert!(
        !text
            .lines()
            .any(|row| row.starts_with("root associated-instance "))
    );
    Ok(())
}

#[path = "selected_initializers/forms.rs"]
mod forms;

#[path = "selected_initializers/inline_query.rs"]
mod inline_query;
