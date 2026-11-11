//! Actual source indexing keeps actual returned places, signature identity and ordinary proof.
#[path = "idxops/indexed_lengths.rs"]
mod indexed_lengths;
use crate::resolve as test_source;
use rustgm::{
    lowir::program::Program,
    pipeline,
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
    },
};
use std::{path::Path, process::Command};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn analyze(name: &str) -> Result<(Report, pipeline::Analysis)> {
    let text = test_source::fixture(&format!("pa31/tests/spec/source-index-places/{name}.rs"))?;
    let mut config = rustgm::expand::config::Config::default();
    if name == "raw-cap-allow" {
        config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    }
    let report = test_source::source_provider_config(&text, "source_index_places", &config)?;
    let facts = pipeline::analyze_source(&report)?;
    Ok((report, facts))
}
fn lower(report: &Report, analysis: pipeline::Analysis) -> Result<Program> {
    let mut roots = Vec::new();
    for (index, d) in report.resolution.definitions.iter().enumerate() {
        let name = report.package.names.get(d.name);
        if let Some(origin) = d.node {
            let ast = &report.package.unit(origin.file).configured.ast;
            if matches!(ast.node(origin.node).kind,rustgm::syntax::ast::Kind::Item(index) if !ast.list(ast.items[index as usize].generics).is_empty())
            {
                continue;
            }
        }
        if d.kind == DefKind::Function
            && (matches!(
                name,
                "probe"
                    | "calc"
                    | "index_read"
                    | "index_write"
                    | "index_borrow"
                    | "index_explicit"
                    | "index_explicit_mut"
                    | "index_generic"
                    | "index_reference"
                    | "index_length"
                    | "index_unsized"
                    | "index_constant"
                    | "index_builtin"
                    | "index_provided"
                    | "index_array_key"
                    | "index_array_write"
                    | "index_array_u8"
                    | "index_order_value"
                    | "index_order_state"
                    | "index_changed"
            ) || name.starts_with("index_bound_"))
        {
            roots.push(DefId(u32::try_from(index)?));
        }
    }
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = analysis;
    Ok(rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &roots,
        },
        prepared,
    )?)
}
fn program(name: &str) -> Result<Program> {
    let (report, facts) = analyze(name)?;
    lower(&report, facts)
}
fn cause_matches(kind: &str, error: &str) -> bool {
    let message = error.to_lowercase();
    let words: &[&str] = match kind {
        "role" => &["actual language declaration", "actual trait"],
        "implementation" => &[
            "implementation",
            "trait",
            "cannot be indexed",
            "cannot index",
            "incompatible",
        ],
        "signature" | "output" | "lifetime" => &[
            "signature",
            "incompatible",
            "method",
            "receiver",
            "lifetime",
            "projection",
            "output",
            "member",
        ],
        "unsafe" => &["unsafe", "safe"],
        "mutable" => &["mutable", "immutable", "shared"],
        "alias" => &["borrow", "loan", "conflict"],
        "escape" => &["lifetime", "outlive", "borrow", "escape"],
        "move" => &["move", "initialized", "copy"],
        "const" => &["const", "indexed", "implementation", "trait"],
        "alignment" => &["align", "packed"],
        "lint" => &["expect", "lint", "unfulfilled", "implicit autoref"],
        "inference" => &["infer", "ambiguous", "trait", "implementation"],
        _ => &[],
    };
    !message.contains("semantic contract is not implemented")
        && !message.contains("expected a token")
        && words.iter().any(|w| message.contains(w))
}
#[test]
fn actual_index_places_keep_signature_effect_and_borrow_causes() -> Result {
    let rows = test_source::fixture("pa31/tests/spec/source-index-places/cases.tsv")?;
    let mut failures = Vec::new();
    let mut count = 0;
    for line in rows.lines() {
        let mut fields = line.splitn(4, '\t');
        let file = fields.next().ok_or("case")?;
        let expected = fields.next().ok_or("status")?;
        let cause = fields.next().ok_or("cause")?;
        let detail = fields.next().ok_or("detail")?;
        let name = file.strip_suffix(".rs").ok_or("suffix")?;
        let result = program(name);
        match (expected, result) {
            ("0", Ok(_)) => {}
            ("1", Err(error)) if cause_matches(cause, &error.to_string()) => {}
            (_, result) => failures.push(format!(
                "{file}: {cause} ({detail}); {}",
                result
                    .err()
                    .map_or_else(|| "accepted".to_owned(), |e| e.to_string())
            )),
        }
        count += 1;
    }
    assert_eq!(count, 73);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}
#[test]
fn actual_index_demand_has_bounded_canonical_work() -> Result {
    let (_, base) = analyze("runtime")?;
    let initial = base.facts.body_work.canonical_steps;
    let mut growth = Vec::new();
    for n in [8, 32, 128] {
        let (_, sample) = analyze(&format!("bound-{n}"))?;
        growth.push(
            sample
                .facts
                .body_work
                .canonical_steps
                .checked_sub(initial)
                .ok_or("work decreased")?,
        );
    }
    eprintln!("source index work: base={initial}, growth={growth:?}");
    assert!(growth[0] > 0 && growth[1] >= growth[0]);
    assert!(growth[2] <= growth[1].saturating_mul(5), "{growth:?}");
    Ok(())
}
#[test]
fn source_fixture_index_foundation_retains_real_constant_slice_access() -> Result {
    let mut source = String::from("#![feature(const_index)]\n");
    source.push_str(&test_source::fixture(
        "pa31/tests/spec/pointer-composition/source-provider.rs",
    )?);
    source.push_str(&test_source::fixture(
        "pa31/tests/spec/pointer-composition/index-foundation.rs",
    )?);
    source.push_str(&test_source::fixture(
        "pa31/tests/spec/pointer-composition/slice-indices.rs",
    )?);
    source.push_str("const READ:usize={let a=[7u8,8u8,9u8];let p=ptr::slice_from_raw_parts(&a as *const [u8;3] as *const u8,3);let s=unsafe{&*p};s[2] as usize};const WRITE:usize={let mut a=[7u8,8u8,9u8];let p=ptr::slice_from_raw_parts_mut(&mut a as *mut [u8;3] as *mut u8,3);let s=unsafe{&mut *p};s[1]=91;s[1] as usize};");
    let report = test_source::source_provider(&source, "fixture_index_foundation")?;
    let analysis = pipeline::analyze_source(&report)?;
    let reader = rustgm::sema::values::Reader::new(&analysis.facts);
    for (name, expected) in [("READ", 9), ("WRITE", 91)] {
        let id = report
            .resolution
            .definitions
            .iter()
            .position(|d| report.package.names.get(d.name) == name)
            .ok_or("constant")?;
        let constant = reader
            .definition(DefId(u32::try_from(id)?))
            .ok_or("constant value")?;
        assert!(
            matches!(reader.view(constant)?,rustgm::sema::values::View::Scalar(rustgm::sema::model::Scalar::Integer(n))if n==expected)
        );
    }
    Ok(())
}
#[test]
fn ordinary_index_foundation_keeps_unstaged_runtime_original_proof() -> Result {
    let mut source = test_source::fixture(
        "pa31/tests/spec/source-header-transport/semantic-provider-qualified.rs",
    )?;
    source.push_str("impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}");
    source.push_str(&test_source::fixture(
        "pa31/tests/spec/pointer-composition/runtime-index-foundation.rs",
    )?);
    source.push_str(
        "#[unsafe(no_mangle)]pub extern \"C\" fn probe()->u8{let values=[7u8,8u8,9u8];values[2]}",
    );
    let report = test_source::source_provider(&source, "runtime_index_foundation")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    let program = rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    assert!(!program.symbols.is_empty());
    source.push_str("const BAD:u8={let values=[7u8,8u8,9u8];values[2]};");
    let report = test_source::source_provider(&source, "runtime_index_const_opponent")?;
    let error = pipeline::analyze_source(&report)
        .err()
        .ok_or("ordinary runtime impl acquired const indexing permission")?;
    assert!(
        error
            .to_string()
            .contains("trait obligation requires an unambiguous matching"),
        "{error}"
    );
    Ok(())
}
fn checked(label: &str, command: &mut Command, folder: &Path) -> Result<Vec<u8>> {
    let output = command.output()?;
    std::fs::write(folder.join(format!("{label}.stderr")), &output.stderr)?;
    if !output.status.success() {
        return Err(format!("{label}: {}", String::from_utf8_lossy(&output.stderr)).into());
    }
    Ok(output.stdout)
}
#[test]
#[ignore = "requires current locked CPPGM object backend and host C compiler"]
fn actual_index_reference_places_and_custom_array_effects_execute_natively() -> Result {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("root")?;
    let cases = [
        ("runtime", "observer.c", "11 17 23 23 23 17 23 11 17\n"),
        ("unsized-output", "unsized-observer.c", "2 23\n"),
        ("const-valid", "constant-observer.c", "23\n"),
        ("provided", "provided-observer.c", "17\n"),
        ("array-custom-runtime", "array-observer.c", "11 23 11\n"),
        ("order", "order-observer.c", "23 4\n"),
        ("primitive-changed-output", "changed-observer.c", "11\n"),
    ];
    for (name, observer, expected) in cases {
        let folder = root.join(format!(
            "obj/parallel/source-transport/indexing/native-{name}-{}",
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
                    root.join("pa31/tests/spec/source-index-places")
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
