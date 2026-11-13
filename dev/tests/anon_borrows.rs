//! Original anonymous constants defer associated values until their exact instance.
use crate::provider_traits::lower;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/anonymous-associated-borrows";
const ORDINARY: &str = "pa25/tests/spec/anonymous-associated-borrows";

#[test]
fn source_inline_borrows_keep_deferred_associated_values() -> Result {
    outcomes(PACKET, 16, lower)
}

#[test]
fn bootstrap_inline_borrows_keep_ordinary_value_and_const_rules() -> Result {
    outcomes(ORDINARY, 5, bootstrap)
}

fn bootstrap(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::resolve(source, true)?;
    crate::provider_traits::lower_report(&report)
}

fn outcomes(
    packet: &str,
    expected_count: usize,
    prepare: impl Fn(&str) -> Result<rustgm::lowir::program::Program>,
) -> Result {
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{packet}/cases.tsv"))?.lines() {
        let columns: Vec<_> = line.split('\t').collect();
        let [path, expected, reason] = columns.as_slice() else {
            return Err("invalid anonymous borrow case".into());
        };
        let result = prepare(&crate::resolve::fixture(path)?);
        if *expected == "0" {
            if let Err(error) = result {
                failures.push(format!("{path}: {error}"));
            }
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("{path}: anonymous borrow opponent accepted"))?;
            assert!(error.to_string().contains(reason), "{path}: {error}");
        }
        count += 1;
    }
    assert_eq!(count, expected_count);
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("\n").into())
    }
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn bootstrap_inline_borrows_execute_selected_values() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/initializer-completion/ordinary-native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let observer = crate::resolve::fixture(&format!("{ORDINARY}/observer.c"))?;
    for name in ["generic-inline", "const-call", "concrete-inline"] {
        let source = crate::resolve::fixture(&format!("{ORDINARY}/{name}.rs"))?;
        crate::ffi_program::check(root, &folder, name, &bootstrap(&source)?, &observer, "7\n")?;
    }
    Ok(())
}

#[test]
fn projected_inline_original_retains_its_complete_source_contract() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/projected-inline.rs"))?;
    let report = crate::provider_traits::report(&source)?;
    rustgm::pipeline::analyze_source(&report)?;
    Ok(())
}

#[test]
fn anonymous_promotion_work_tracks_distinct_owners() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/borrowed-inline-size.rs"))?;
    let (prefix, _) = source
        .rsplit_once("pub fn generic<T>")
        .ok_or("anonymous borrow prefix")?;
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut text = prefix.to_owned();
        for index in 0..width {
            writeln!(
                text,
                "pub fn probe{index}<T>()->usize{{const{{let p=&<T as SizedTypeProperties>::SIZE;*p}}}}"
            )?;
        }
        let report = crate::provider_traits::report(&text)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis.facts.constant_work.anonymous_templates,
        ];
        println!("anonymous borrows width={width} work={work:?}");
        if let Some(old) = previous {
            for (before, after) in std::iter::zip(old, work) {
                assert!(
                    after <= 5 * before + 512,
                    "width={width}: {old:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn anonymous_borrow_instances_execute_actual_associated_initializers() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/initializer-completion/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    crate::ffi_program::check(
        root,
        &folder,
        "anonymous",
        &lower(&crate::resolve::fixture(&format!("{PACKET}/native.rs"))?)?,
        &crate::resolve::fixture(&format!("{PACKET}/observer.c"))?,
        "4 8 4 4 8\n",
    )
}
