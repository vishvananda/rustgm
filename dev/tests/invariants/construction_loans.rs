//! Genuine source constructors; authored observations retain actual temporary loans.
use crate::{pipeline, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn prepare(input: &str) -> Result<crate::lowir::program::Program> {
    let report = test_source::source_provider(input, "argument_construction")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn real_arguments_constructors_preserve_source_calls_borrows_and_drop_endpoints() -> Result {
    for (name, fixture) in [("stored", "real"), ("statement", "statement")] {
        let input = test_source::fixture(&format!(
            "pa31/tests/spec/source-format-macros/arguments/constructor-{fixture}.rs"
        ))?;
        let program = prepare(&input).map_err(|error| format!("{name}: {error}"))?;
        assert!(
            program
                .symbols
                .iter()
                .any(|symbol| symbol.name == "answer" && symbol.body.is_some())
        );
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../obj/parallel/pa31/format-arguments-v1/native/{name}"
        ));
        std::fs::create_dir_all(&folder)?;
        std::fs::write(folder.join("provider.rs"), input)?;
        crate::lowir::writer::write(
            &program,
            &mut std::fs::File::create(folder.join("provider.lowir"))?,
        )?;
    }
    Ok(())
}
#[test]
fn actual_argument_lifetimes_and_unsafe_const_inputs_keep_ordinary_call_proofs() -> Result {
    for (name, accepted, diagnostic) in [
        ("return", false, ""),
        ("block", false, ""),
        ("unsafe", false, "unsafe"),
        ("explicit", true, ""),
        ("const", false, "const"),
        (
            "const-plain",
            false,
            "non-const calls are not allowed in const functions",
        ),
        ("binder", false, ""),
    ] {
        let input = test_source::fixture(&format!(
            "pa31/tests/spec/source-format-macros/arguments/constructor-{name}.rs"
        ))?;
        let result = prepare(&input);
        if accepted {
            result.map_err(|error| format!("{name}: {error}"))?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{name}: invalid actual argument consumer accepted"))?;
            let message = error.to_string();
            if matches!(name, "return" | "block" | "binder") {
                assert!(
                    [
                        "required lifetime does not outlive its target",
                        "borrowed temporary does not live long enough",
                        "access conflicts with a live reference loan",
                    ]
                    .iter()
                    .any(|expected| message.contains(expected)),
                    "{name}: {message}"
                );
            } else {
                assert!(message.contains(diagnostic), "{name}: {message}");
            }
            eprintln!("constructor boundary {name}: {message}");
        }
    }
    Ok(())
}
