//! Actual fieldless nominal tags share transport while ownership and validity stay nominal.
use crate::resolve as source;
use rustgm::{
    abi::classification::{Passing, Plans},
    lowir::program::Program,
    pipeline,
    resolve::{
        expansion::Report,
        load::{InputFile, Sources},
        model::DefKind,
    },
    support::source::SourceFile,
    types::{Kind, Primitive, TypeId},
};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct Resources;
impl Sources for Resources {
    fn read(&mut self, path: &Path) -> std::result::Result<Option<InputFile>, String> {
        if path.file_name().is_none_or(|name| name != "c_void.md") {
            return Ok(None);
        }
        let text = source::fixture("pa31/tests/spec/source-c-enum/c_void.md")
            .map_err(|e| e.to_string())?;
        Ok(Some(InputFile {
            identity: path.into(),
            path: path.into(),
            source: SourceFile::new(text.into_bytes())?,
        }))
    }
}
fn analyze(name: &str) -> Result<(Report, pipeline::Analysis)> {
    let text = source::fixture(&format!("pa31/tests/spec/source-c-enum/{name}.rs"))?;
    let report = source::source_provider_with_sources(
        &text,
        "source_c_enum",
        &rustgm::expand::config::Config::default(),
        &mut Resources,
    )?;
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn program(name: &str) -> Result<Program> {
    let (report, analysis) = analyze(name)?;
    let roots = analysis
        .facts
        .functions
        .iter()
        .filter(|function| {
            let definition = &report.resolution.definitions[function.definition.0 as usize];
            let name = report.package.names.get(definition.name);
            definition.kind == DefKind::Function
                && (name.starts_with("echo_")
                    || matches!(
                        name,
                        "pair" | "indirect" | "imported" | "generic" | "consume" | "echo"
                    ))
        })
        .map(|f| f.definition)
        .collect::<Vec<_>>();
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?)
}
#[test]
fn actual_tags_retain_exact_nominal_identity_and_scalar_plan() -> Result {
    let (report, analysis) = analyze("widths")?;
    let pipeline::Analysis { mut facts, .. } = analysis;
    let mut plans = Plans::default();
    let mut count = 0;
    for function in &facts.functions {
        let definition = &report.resolution.definitions[function.definition.0 as usize];
        let name = report.package.names.get(definition.name);
        let Some(tag) = name.strip_prefix("echo_") else {
            continue;
        };
        let primitive = match tag {
            "u8" => Primitive::U8,
            "i8" => Primitive::I8,
            "u16" => Primitive::U16,
            "i16" => Primitive::I16,
            "u32" => Primitive::U32,
            "i32" => Primitive::I32,
            "u64" => Primitive::U64,
            "i64" => Primitive::I64,
            "u128" => Primitive::U128,
            "i128" => Primitive::I128,
            "usize" => Primitive::Usize,
            "isize" => Primitive::Isize,
            _ => return Err("unknown fixture tag".into()),
        };
        let Kind::Function {
            parameters, result, ..
        } = facts
            .types
            .kind(function.signature)
            .ok_or("actual signature")?
        else {
            return Err("actual function".into());
        };
        let &[nominal] = facts.types.list(parameters).ok_or("actual inputs")? else {
            return Err("actual one input".into());
        };
        assert_eq!(result, nominal);
        assert!(matches!(facts.types.kind(nominal), Some(Kind::Adt(..))));
        let leaf = TypeId::primitive(primitive);
        assert_ne!(nominal, leaf);
        let plan = plans.demand(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            function.signature,
        )?;
        assert_eq!(plan.signature, function.signature);
        assert_eq!(&*plan.parameters, &[Passing::Scalar(leaf)]);
        assert_eq!(plan.result, Passing::Scalar(leaf));
        let before = facts.layouts.steps;
        let same = plans.demand(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            function.signature,
        )?;
        assert!(std::sync::Arc::ptr_eq(&plan, &same));
        assert_eq!(facts.layouts.steps, before);
        count += 1;
    }
    assert_eq!(count, 12);
    for name in ["widths", "calls", "drops2", "alias"] {
        let output = program(name).map_err(|e| format!("{name}: {e}"))?;
        rustgm::lowir::writer::write(&output, &mut Vec::new())?;
    }
    Ok(())
}
#[test]
fn enum_transport_keeps_move_identity_mutability_and_discriminant_causes() -> Result {
    for (name, words) in [
        ("move", "moved|initialized"),
        ("identity", "mismatch|incompatible|unify"),
        ("mutable", "immutable"),
        ("duplicate", "discriminant"),
        ("range", "range|overflow|discriminant"),
        ("packed", "enum|representation|packed"),
    ] {
        let error = program(name)
            .err()
            .ok_or_else(|| format!("{name}: invalid source accepted"))?
            .to_string();
        eprintln!("{name}: {error}");
        assert!(
            !error.contains("not implemented") && words.split('|').any(|word| error.contains(word)),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn wider_valid_enum_abi_profiles_remain_named_requirements() -> Result {
    for name in ["default", "payload", "aligned", "trans"] {
        // Independently Rust-positive; these finite-profile fences are required work.
        let error = program(name)
            .err()
            .ok_or_else(|| format!("{name}: boundary changed; qualify it"))?;
        assert!(
            error.to_string().contains("C aggregate ABI profile"),
            "{name}: {error}"
        );
    }
    Ok(())
}
#[test]
fn variant_walk_is_bounded_and_warm_plans_do_not_rescan() -> Result {
    let mut growth = Vec::new();
    for width in [8, 32, 128] {
        let (report, analysis) = analyze(&format!("work{width}"))?;
        let pipeline::Analysis { mut facts, .. } = analysis;
        let signature = facts
            .functions
            .iter()
            .find(|f| {
                let d = &report.resolution.definitions[f.definition.0 as usize];
                report.package.names.get(d.name) == "echo"
            })
            .ok_or("echo")?
            .signature;
        let mut plans = Plans::default();
        let before = facts.layouts.steps;
        plans.demand(
            &facts.types,
            &facts.aggregates,
            &mut facts.layouts,
            signature,
        )?;
        let sample = facts.layouts.steps - before;
        growth.push(sample);
        let before = facts.layouts.steps;
        for _ in 0..32 {
            plans.demand(
                &facts.types,
                &facts.aggregates,
                &mut facts.layouts,
                signature,
            )?;
        }
        assert_eq!(before, facts.layouts.steps);
    }
    eprintln!("enum scalar layout work {growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1] * 5, "{growth:?}");
    Ok(())
}
#[test]
#[ignore = "requires the locked object backend and a C observer compiler"]
fn actual_enum_tags_callbacks_nested_fields_and_drop_execute_natively() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    for (name, observer, expected) in [
        (
            "widths",
            "widths.c",
            "1 -3 65535 -3 4294967295 -3 18446744073709551615 -3 1 1 17 -3\n",
        ),
        ("calls", "calls.c", "1 73 1 1 513\n"),
        ("drops2", "drops.c", "9\n"),
    ] {
        let folder = root.join(format!(
            "obj/parallel/source-transport/cenum/native-{name}-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&folder)?;
        let input = folder.join("source.lowir");
        let object = folder.join("source.o");
        let executable = folder.join("observer");
        rustgm::lowir::writer::write(&program(name)?, &mut std::fs::File::create(&input)?)?;
        checked(
            "backend",
            Command::new("python3")
                .arg(root.join("scripts/backend.py"))
                .arg("--emit-object")
                .arg(input)
                .arg("-o")
                .arg(&object),
            &folder,
        )?;
        checked(
            "link",
            Command::new("cc")
                .arg(root.join("pa31/tests/spec/source-c-enum").join(observer))
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
