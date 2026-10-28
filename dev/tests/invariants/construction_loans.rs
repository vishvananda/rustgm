//! Genuine source constructors; authored observations retain actual temporary loans.
use crate::{pipeline, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct TraitDocumentation(Vec<u8>);
impl crate::resolve::load::Sources for TraitDocumentation {
    fn read(
        &mut self,
        _: &std::path::Path,
    ) -> std::result::Result<Option<crate::resolve::load::InputFile>, String> {
        Ok(None)
    }
    fn resource(&mut self, path: &std::path::Path) -> std::result::Result<Option<Vec<u8>>, String> {
        Ok((path == std::path::Path::new("fmt_trait_method_doc.md")).then(|| self.0.clone()))
    }
}
#[test]
fn complete_source_constructor_family_retains_real_methods_and_missing_member_denials() -> Result {
    let input = test_source::fixture(
        "pa31/tests/spec/source-format-macros/constructor-family/source-nine-traits.rs",
    )?;
    let documentation = test_source::fixture(
        "pa31/tests/spec/source-format-macros/constructor-family/fmt_trait_method_doc.md",
    )?;
    let report = test_source::source_provider_with_sources(
        &input,
        "argument_construction",
        &crate::expand::config::Config::default(),
        &mut TraitDocumentation(documentation.clone().into_bytes()),
    )?;
    let program = prepare_report(&report)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/format-constructor-family-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), &input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    let changed = input.replace(
        "pub const fn new_upper_hex<T:",
        "pub const fn unrelated_upper_hex<T:",
    );
    assert_ne!(changed, input);
    let report = test_source::source_provider_with_sources(
        &changed,
        "argument_construction",
        &crate::expand::config::Config::default(),
        &mut TraitDocumentation(documentation.into_bytes()),
    )?;
    let error = prepare_report(&report)
        .err()
        .ok_or("missing actual constructor was accepted")?;
    assert!(
        error.to_string().contains("missing method bucket"),
        "{error}"
    );
    Ok(())
}
fn prepare(input: &str) -> Result<crate::lowir::program::Program> {
    let report = test_source::source_provider(input, "argument_construction")?;
    prepare_report(&report)
}
fn prepare_report(
    report: &crate::resolve::expansion::Report,
) -> Result<crate::lowir::program::Program> {
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(report)?;
    Ok(crate::lower::instances::build_roots_prepared(
        report,
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

#[test]
fn mixed_display_and_debug_pairs_use_real_source_constructors_once() -> Result {
    // Rust accepts the reduced no-Copy provider through ordinary call reborrowing.
    // Keep that outstanding PA16/PA31 boundary distinct from core's real impls.
    let reduced =
        test_source::fixture("pa31/tests/spec/source-format-macros/display/source-mixed.rs")?;
    let error = prepare(&reduced)
        .err()
        .ok_or("revisit the retained shared-reference call-reborrow boundary")?;
    assert!(error.to_string().contains("uninitialized or moved"));
    let input =
        test_source::fixture("pa31/tests/spec/source-format-macros/display/source-shared-refs.rs")?;
    let program = prepare(&input)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/format-display-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn original_lower_hex_constructor_keeps_static_options_and_shared_pair_lifetimes() -> Result {
    let input = test_source::fixture(
        "pa31/tests/spec/source-format-macros/static-options/source-provider-qualified.rs",
    )?;
    let program = prepare(&input)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/pa31/format-static-options-v1/native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("provider.rs"), input)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}

#[test]
fn captured_and_named_constructors_preserve_real_inputs_hygiene_and_drop_endpoints() -> Result {
    for name in [
        "source-captured",
        "source-explicit",
        "source-forward-literal",
        "source-forward-expr",
    ] {
        let input = test_source::fixture(&format!(
            "pa31/tests/spec/source-format-macros/named-captures/{name}.rs"
        ))?;
        let program = prepare(&input).map_err(|error| format!("{name}: {error}"))?;
        let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "../obj/parallel/pa31/format-named-captures-v1/native/{name}"
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
