//! Root associated values reuse actual scoped Copy qualification before const cleanup.
use crate::resolve as source;
use rustgm::{
    pipeline,
    sema::values::{Reader, View},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
fn report(owner: &str, name: &str) -> Result<rustgm::resolve::expansion::Report> {
    let text = source::fixture(&format!(
        "{owner}/tests/spec/projected-copy-cleanup/{name}.rs"
    ))?;
    Ok(if owner == "pa31" {
        source::source_provider(&text, "projected_copy_cleanup")?
    } else {
        source::resolve(&text, true)?
    })
}
fn program(name: &str) -> Result<rustgm::lowir::program::Program> {
    let program = crate::provider_traits::lower_report(&report("pa31", name)?)?;
    assert!(
        program
            .symbols
            .iter()
            .any(|s| s.name == "observe" && s.body.is_some())
    );
    Ok(program)
}
#[test]
fn associated_copy_bounds_qualify_actual_owned_parameters() -> Result {
    for name in [
        "declared-copy",
        "where-copy",
        "moved-copy",
        "parameter-control",
        "reference-control",
    ] {
        program(name).map_err(|error| format!("{name}: {error}"))?;
    }
    for name in ["declared-copy", "where-copy"] {
        let report = report("pa25", name)?;
        let analysis = pipeline::analyze_source(&report)?;
        let initializer = analysis
            .facts
            .initializers
            .iter()
            .find(|i| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[i.definition.0 as usize].name)
                    == "ANSWER"
            })
            .ok_or("ordinary ANSWER")?;
        let reader = Reader::new(&analysis.facts);
        let value = reader
            .definition(initializer.definition)
            .ok_or("completed ordinary ANSWER")?;
        assert!(matches!(
            reader.view(value)?,
            View::Scalar(rustgm::sema::model::Scalar::Integer(7))
        ));
    }
    Ok(())
}
#[test]
fn unpaid_and_lookalike_bounds_reject_actual_reached_cleanup() -> Result {
    for (owner, name, reason) in [
        (
            "pa25",
            "unpaid-reached",
            "constant evaluation cannot run destructors",
        ),
        (
            "pa31",
            "unpaid-reached",
            "active cleanup requires const Destruct",
        ),
        (
            "pa31",
            "lookalike-reached",
            "active cleanup requires const Destruct",
        ),
    ] {
        let error = pipeline::analyze_source(&report(owner, name)?)
            .err()
            .ok_or_else(|| format!("{owner}/{name} accepted"))?
            .to_string();
        assert!(error.contains(reason), "{owner}/{name}: {error}");
    }
    Ok(())
}

#[test]
fn aggregate_destruct_premise_only_pays_its_owned_cleanup_fields() -> Result {
    let provider = source::fixture("pa31/tests/spec/projected-copy-cleanup/declared-copy.rs")?;
    let (prefix, _) = provider
        .split_once("pub trait Has")
        .ok_or("provider prefix")?;
    let common = r#"
pub enum Maybe<T> { Some(T), None }
pub enum Flow<B, C> { Continue(C), Break(B) }
impl<B, C> Flow<B, C> {
    pub const fn break_value(self ARGUMENT) -> Maybe<B> PREMISE {
        match self {
            Self::Continue(..) => Maybe::None,
            Self::Break(x) => Maybe::Some(x),
        }
    }
}
#[unsafe(no_mangle)] pub extern "C" fn observe() -> u8 { 7 }
"#;
    for (argument, premise, accepted) in [
        ("", "where Self: [const] Destruct", true),
        ("", "", false),
        (", _other: C", "where Self: [const] Destruct", false),
    ] {
        let body = common
            .replace("ARGUMENT", argument)
            .replace("PREMISE", premise);
        let input = format!("#![feature(const_precise_live_drops)]\n{prefix}{body}");
        let report = source::source_provider(&input, "aggregate_cleanup")?;
        let lowered = crate::provider_traits::lower_report(&report);
        if accepted {
            lowered?;
        } else {
            let error = lowered.err().ok_or("unpaid cleanup accepted")?;
            assert!(
                error
                    .to_string()
                    .contains("active cleanup requires const Destruct"),
                "{error}"
            );
        }
    }
    Ok(())
}

#[test]
fn repeated_projected_roots_have_bounded_qualification_work() -> Result {
    let mut previous: Option<[usize; 3]> = None;
    for count in [8, 32, 128] {
        let report = report("pa31", &format!("scaling-{count}"))?;
        let analysis = pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis.facts.constant_work.steps,
        ];
        if let Some(old) = previous {
            assert!(work.into_iter().zip(old).all(|(new, old)| new <= old * 5));
        }
        println!("projected roots count={count} work={work:?}");
        previous = Some(work);
    }
    Ok(())
}
#[test]
#[ignore = "requires pinned CPPGM and host C linker"]
fn projected_copy_parameters_execute_without_spurious_cleanup() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let folder = root
        .join("obj/parallel/projected-cleanup/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for name in [
        "declared-copy",
        "where-copy",
        "moved-copy",
        "parameter-control",
        "reference-control",
    ] {
        crate::ffi_program::check(
            &root,
            &folder,
            name,
            &program(name)?,
            &source::fixture("pa31/tests/spec/projected-copy-cleanup/observe.c")?,
            "7\n",
        )?;
    }
    Ok(())
}
