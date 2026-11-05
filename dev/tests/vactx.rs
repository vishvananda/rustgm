//! Genuine source vararg bodies retain actual cursor, borrowed call and cleanup authority.
use crate::resolve as test_source;
use rustgm::{
    lowir::program::Program,
    pipeline,
    resolve::{
        expansion::Report,
        load::{InputFile, Sources},
        model::{DefId, DefKind},
    },
    support::source::SourceFile,
    types::{Kind, callable::Abi},
};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Resources;
impl Sources for Resources {
    fn read(&mut self, path: &Path) -> std::result::Result<Option<InputFile>, String> {
        if path.file_name().is_none_or(|name| name != "c_void.md") {
            return Ok(None);
        }
        let text = test_source::fixture("pa31/tests/spec/source-va-runtime/c_void.md")
            .map_err(|e| e.to_string())?;
        Ok(Some(InputFile {
            identity: path.into(),
            path: path.into(),
            source: SourceFile::new(text.into_bytes())?,
        }))
    }
}
fn analyze(name: &str) -> Result<(Report, pipeline::Analysis)> {
    let text = test_source::fixture(&format!("pa31/tests/spec/source-va-runtime/{name}.rs"))?;
    let report = test_source::source_provider_with_sources(
        &text,
        "source_va_runtime",
        &rustgm::expand::config::Config::default(),
        &mut Resources,
    )?;
    let facts = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
fn program(name: &str) -> Result<Program> {
    compiled(name).map(|(program, _)| program)
}
fn compiled(name: &str) -> Result<(Program, usize)> {
    let (report, analysis) = analyze(name)?;
    let mut roots = Vec::new();
    for function in &analysis.facts.functions {
        let declaration = &report.resolution.definitions[function.definition.0 as usize];
        let name = report.package.names.get(declaration.name);
        if declaration.kind == DefKind::Function
            && (name.starts_with("va_work")
                || matches!(
                    name,
                    "va_int"
                        | "va_copy"
                        | "va_float"
                        | "va_ptr"
                        | "va_stack"
                        | "va_early"
                        | "va_owned"
                        | "va_zero"
                        | "va_order"
                        | "va_return"
                        | "cv_echo"
                        | "cv_local"
                        | "probe"
                ))
            && matches!(analysis.facts.types.kind(function.signature),Some(Kind::Function{convention,..}) if convention.abi==Abi::C)
        {
            roots.push(DefId(function.definition.0));
        }
    }
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    let output = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?;
    Ok((output, facts.body_work.canonical_steps))
}
#[test]
fn actual_source_cursor_and_void_header_reach_typed_lowir() -> Result {
    for name in [
        "source", "srczero", "unused2", "cvoid", "cvloc2", "dc_ok", "dc_order", "fixarg", "fdcall",
    ] {
        let output = program(name).map_err(|e| format!("{name}: {e}"))?;
        let mut text = Vec::new();
        rustgm::lowir::writer::write(&output, &mut text)?;
        let text = String::from_utf8(text)?;
        if matches!(name, "source" | "srczero") {
            assert!(text.contains("va_start "), "{name}: {text}");
            assert!(text.contains("va_arg "), "{name}: {text}");
        }
    }
    Ok(())
}
#[test]
fn original_positive_c_variadic_requests_keep_their_actual_source() -> Result {
    let names = test_source::fixture("pa31/tests/spec/source-va-runtime/feature.tsv")?;
    let mut count = 0;
    for name in names.lines() {
        let text = test_source::fixture(&format!("pa31/tests/spec/source-va-list/initial/{name}"))?;
        let report = test_source::source_provider(&text, "original_vararg_feature")?;
        pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
        count += 1;
    }
    assert_eq!(count, 16);
    Ok(())
}
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn actual_source_cursor_reads_clones_and_scope_cleanup_execute_natively() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (name, observer, expected) in [
        ("run", "native.c", "17 11 17.5 23 23 29 31\n"),
        ("rzro", "zero2.c", "37\n"),
        ("cvloc2", "cvloc.c", "1\n"),
        ("cvoid", "cvoid.c", "1\n"),
        ("rorder", "drop.c", "41 41 77 47\n"),
        ("rfixed", "fixed.c", "31.5\n"),
        ("rfrom", "fdcall.c", "17\n"),
    ] {
        let folder = root.join(format!(
            "obj/parallel/source-transport/va-run/native-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let ir = folder.join("source.lowir");
        rustgm::lowir::writer::write(&program(name)?, &mut std::fs::File::create(&ir)?)?;
        let object = folder.join("source.o");
        checked(
            "backend",
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(&ir)
                .arg("-o")
                .arg(&object),
            &folder,
        )?;
        let executable = folder.join("observer");
        checked(
            "link",
            Command::new("cc")
                .arg(
                    root.join("pa31/tests/spec/source-va-runtime")
                        .join(observer),
                )
                .arg(object)
                .arg("-o")
                .arg(&executable),
            &folder,
        )?;
        let actual = checked("native", &mut Command::new(executable), &folder)?;
        std::fs::write(folder.join("native.stdout"), &actual)?;
        assert_eq!(actual, expected.as_bytes(), "{name}");
    }
    Ok(())
}
fn checked(label: &str, command: &mut Command, folder: &Path) -> Result<Vec<u8>> {
    std::fs::write(
        folder.join(format!("{label}.command")),
        format!("{command:?}\n"),
    )?;
    let output = command.output()?;
    std::fs::write(folder.join(format!("{label}.stderr")), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("{label}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(output.stdout)
}
#[test]
fn shared_opening_law_preserves_actual_discriminant_consumers() -> Result {
    for name in ["actual-calls", "opened-scope"] {
        let text = test_source::fixture(&format!(
            "pa31/tests/spec/source-discriminants/consumers/{name}.rs"
        ))?;
        let report = test_source::source_provider(&text, "source_discriminant_opening")?;
        pipeline::analyze_source(&report).map_err(|error| format!("{name}: {error}"))?;
    }
    let text =
        test_source::fixture("pa31/tests/spec/source-discriminants/consumers/reference-fault.rs")?;
    let report = test_source::source_provider(&text, "source_discriminant_fault")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("invalid actual reference was erased")?;
    assert!(
        error
            .to_string()
            .contains("constant pointer has no allocation provenance"),
        "{error}"
    );
    Ok(())
}
#[test]
fn cursor_borrows_effects_and_const_drop_keep_their_real_causes() -> Result {
    program("source")?;
    let mut failures = Vec::new();
    for (name, words) in [
        ("rt_readsafe", "unsafe"),
        ("rt_readbool", "trait|implementation|VaArgSafe"),
        ("rt_alias", "borrow|loan"),
        ("rt_escape", "lifetime|outlive|borrow|escape"),
        ("rt_mutability", "mutable|immutable"),
        ("rt_endunsafe", "unsafe"),
        ("dc_trait2", "const trait|constant trait|not const"),
        ("dc_unsafe", "positive safe impl"),
        ("dc_method", "non-const"),
    ] {
        let error = program(name).err().map(|error| error.to_string());
        eprintln!("{name}: {error:?}");
        if !error.as_deref().is_some_and(|message| {
            !message.contains("not implemented")
                && words.split('|').any(|word| message.contains(word))
        }) {
            failures.push(format!("{name}: {error:?}; expected {words}"));
        }
    }
    for name in ["rt_copyok", "rt_ended", "rt_cloneborrow", "dc_ok"] {
        if let Err(error) = program(name) {
            failures.push(format!("{name}: {error}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn actual_cursor_demands_have_bounded_canonical_work() -> Result {
    let (_, initial) = compiled("work0")?;
    let mut growth = Vec::new();
    for width in [8, 32, 128] {
        let (output, sample) = compiled(&format!("work{width}"))?;
        let mut text = Vec::new();
        rustgm::lowir::writer::write(&output, &mut text)?;
        let text = String::from_utf8(text)?;
        assert_eq!(text.matches("va_start ").count(), width);
        growth.push(
            sample
                .checked_sub(initial)
                .ok_or("canonical work decreased")?,
        );
    }
    eprintln!("actual cursor canonical work: base={initial}, growth={growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1].saturating_mul(5), "{growth:?}");
    Ok(())
}

#[test]
fn const_cursor_definition_compiles_and_reached_cleanup_remains_required() -> Result {
    program("cldef")?;
    let error = program("clcall")
        .err()
        .ok_or("interpreter cleanup boundary changed; qualify it")?;
    assert!(
        error
            .to_string()
            .contains("constant interpreter cleanup scheduling is not implemented"),
        "{error}"
    );
    Ok(())
}

#[test]
fn ellipsis_stays_outside_the_public_fixed_signature() -> Result {
    for (name, function, fixed) in [
        ("source", "va_int", 1),
        ("srczero", "va_zero", 0),
        ("unused2", "probe", 1),
    ] {
        let (report, analysis) = analyze(name)?;
        let function = analysis
            .facts
            .functions
            .iter()
            .find(|f| {
                let definition = &report.resolution.definitions[f.definition.0 as usize];
                report.package.names.get(definition.name) == function
            })
            .ok_or("actual source function")?;
        let Kind::Function {
            parameters,
            bound_regions,
            convention,
            ..
        } = analysis
            .facts
            .types
            .kind(function.signature)
            .ok_or("actual signature")?
        else {
            return Err("actual function signature".into());
        };
        assert!(convention.variadic);
        assert_eq!(bound_regions, 0);
        assert_eq!(
            analysis
                .facts
                .types
                .list(parameters)
                .ok_or("fixed parameters")?
                .len(),
            fixed
        );
        assert_eq!(
            report
                .package
                .unit(function.node.file)
                .configured
                .ast
                .list(function.parameters)
                .len(),
            fixed + 1
        );
    }
    Ok(())
}
