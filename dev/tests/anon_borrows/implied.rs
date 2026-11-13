//! Anonymous originals inherit only their enclosing declaration's implied bounds.
use super::{Result, bootstrap};
use std::fmt::Write as _;
const ORDINARY: &str = "pa25/tests/spec/anonymous-implied-bounds";
const SOURCE: &str = "pa31/tests/spec/anonymous-implied-bounds";

#[test]
fn implied_inputs_preserve_complete_outcomes() -> Result {
    for packet in [ORDINARY, SOURCE] {
        let mut count = 0;
        let mut failures = Vec::new();
        for line in crate::resolve::fixture(&format!("{packet}/cases.tsv"))?.lines() {
            let columns: Vec<_> = line.split('\t').collect();
            let [path, expected, _] = columns.as_slice() else {
                return Err("invalid implied-bound case".into());
            };
            let source = crate::resolve::fixture(path)?;
            let result = if packet == SOURCE {
                crate::provider_traits::lower(&source)
            } else {
                bootstrap(&source)
            };
            println!("{path}: {:?}", result.as_ref().map(|_| ()));
            if result.is_ok() != (*expected == "0") {
                failures.push(format!("{path}: {:?}", result.err()));
            }
            count += 1;
        }
        assert_eq!(count, 13);
        if !failures.is_empty() {
            return Err(failures.join("\n").into());
        }
    }
    Ok(())
}

#[test]
fn bulk_contracts_keep_implied_inputs_and_anonymous_ownership() -> Result {
    for packet in [ORDINARY, SOURCE] {
        for name in [
            "direct-inherent",
            "macro-trait",
            "free-input",
            "nested-input",
        ] {
            let text = crate::resolve::fixture(&format!("{packet}/{name}.rs"))?;
            let report = if packet == SOURCE {
                crate::provider_traits::report(&text)?
            } else {
                crate::resolve::resolve(&text, true)?
            };
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let mut facts = analysis.facts;
            let mut templates = rustgm::body::templates::build(&report, &facts)?;
            rustgm::body::promote::analyze_templates(&report, &mut facts, &mut templates)?;
            let contracts = rustgm::body::regions::collect(&facts)?;
            rustgm::borrow::regions::check_templates(
                &facts.types,
                &facts.lifetimes.variances,
                &contracts,
                &templates,
            )?;
        }
    }
    Ok(())
}

#[test]
fn implied_bound_work_tracks_anonymous_owners() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut text = "enum Slot<T>{Empty,Full(T)}\n".to_owned();
        for index in 0..width {
            writeln!(
                text,
                "pub fn p{index}<'a,T>(input:&'a T)->Slot<&'a T>{{const{{Slot::<&'a T>::Empty}}}}"
            )?;
        }
        let report = crate::resolve::resolve(&text, true)?;
        let analysis = rustgm::pipeline::analyze_source(&report)?;
        let work = [
            analysis.facts.body_work.inference_steps,
            analysis.facts.body_work.canonical_steps,
            analysis.facts.constant_work.anonymous_templates,
        ];
        assert_eq!(work[2], width);
        println!("implied bounds width={width} work={work:?}");
        if let Some(old) = previous {
            for (before, after) in std::iter::zip(old, work) {
                assert!(after <= 5 * before + 512, "{old:?}->{work:?}");
            }
        }
        previous = Some(work);
    }
    Ok(())
}

#[test]
#[ignore = "requires the pinned CPPGM backend and host C linker"]
fn inherited_input_instances_execute() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let folder = root
        .join("obj/parallel/inline-region-arrays/native")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, packet) in [("ordinary", ORDINARY), ("source", SOURCE)] {
        let source = crate::resolve::fixture(&format!("{packet}/scalar-native.rs"))?;
        let program = if packet == SOURCE {
            crate::provider_traits::lower(&source)?
        } else {
            bootstrap(&source)?
        };
        crate::ffi_program::check(
            root,
            &folder,
            name,
            &program,
            &crate::resolve::fixture(&format!("{packet}/scalar-observer.c"))?,
            "7\n",
        )?;
    }
    Ok(())
}
