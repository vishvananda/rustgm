//! Contextual shared reborrows keep the ordinary source Copy and loan boundary.
use crate::{body, borrow, pipeline, test_source};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn analyze(
    input: &str,
    source: bool,
) -> Result<(crate::resolve::expansion::Report, pipeline::Analysis)> {
    let report = if source {
        test_source::source_provider(input, "shared_call_reborrow")?
    } else {
        test_source::resolve(input, true)?
    };
    let analysis = pipeline::analyze_source(&report)?;
    Ok((report, analysis))
}
fn check(input: &str, source: bool) -> Result<(usize, usize)> {
    let (report, pipeline::Analysis { mut facts, .. }) = analyze(input, source)?;
    let mut builder = body::templates::Prepared::default();
    let mut bodies = Vec::new();
    let mut promotions = body::promote::Scratch::default();
    for function in facts.functions.clone() {
        if function.body.is_none() {
            continue;
        }
        let mut template = builder.function(&report, &facts, &function)?;
        body::promote::instance_reusing(
            &report,
            &mut facts,
            &mut template.body,
            &template.destruction,
            &mut promotions,
        )?;
        bodies.push(template);
    }
    let templates = body::templates::Templates::source(bodies);
    let mut contracts = body::regions::collect(&facts)?.bodies;
    for template in &templates.bodies {
        let contract = contracts
            .iter_mut()
            .find(|contract| contract.key == template.body.key())
            .ok_or("actual source body contract")?;
        body::prepare_source_copy(&mut facts, &template.body, contract)?;
        contract.prepare_outlives(&mut facts.types, &mut facts.lifetimes.nominal_requirements)?;
    }
    let contracts = body::regions::Contracts {
        bodies: contracts,
        callables: &facts.lifetimes.callables,
        nominal_requirements: &facts.lifetimes.nominal_requirements,
    };
    let mut environment = borrow::Environment {
        types: &facts.types,
        definitions: &facts.aggregates,
        derives: &facts.derives,
        source_copy: facts.uses_source_copy(),
        copies: &mut facts.copies,
    };
    borrow::templates::verify(
        &mut environment,
        &facts.lifetimes.variances,
        &contracts,
        &templates,
        &[],
        |_, _| Err("unexpected destructor field"),
    )?;
    Ok((
        facts.reference_adjustments.len(),
        facts.body_work.canonical_steps,
    ))
}
#[test]
fn explicit_reference_contexts_preserve_reuse_without_source_copy() -> Result {
    for name in [
        "direct",
        "tuple",
        "field",
        "generic-referent",
        "function-pointer",
        "nested",
        "block",
        "branches",
        "match",
        "loop",
        "typed-let",
        "mutable-call",
        "shared-from-mutable",
    ] {
        for (assignment, source) in [
            (if name == "generic-referent" { 18 } else { 16 }, false),
            (31, true),
        ] {
            let input = test_source::fixture(&format!(
                "pa{assignment}/tests/spec/shared-call-reborrow/{name}.rs"
            ))?;
            check(&input, source).map_err(|error| format!("{assignment}/{name}: {error}"))?;
        }
    }
    for name in [
        "alias",
        "method-argument",
        "mutable-nested",
        "wide-text",
        "generic-turbofish",
        "unsize-complete",
    ] {
        let input =
            test_source::fixture(&format!("pa31/tests/spec/shared-call-reborrow/{name}.rs"))?;
        check(&input, true).map_err(|error| format!("source/{name}: {error}"))?;
    }
    Ok(())
}
#[test]
fn inferred_moves_copy_bounds_and_loans_remain_independent() -> Result {
    for name in [
        "generic-value",
        "inferred-let",
        "copy-bound",
        "late-inferred",
        "generic-after-reference",
        "mutable-generic-move",
        "shared-to-mut",
        "live-shared-write",
        "return-local",
        "owned-move",
    ] {
        let input =
            test_source::fixture(&format!("pa31/tests/spec/shared-call-reborrow/{name}.rs"))?;
        let error = check(&input, true)
            .err()
            .ok_or_else(|| format!("source/{name} was accepted"))?;
        let expected = match name {
            "copy-bound" => "trait obligation",
            "shared-to-mut" => "cannot coerce a shared reference to a mutable reference",
            "live-shared-write" => "access conflicts with a live reference loan",
            "return-local" => "required lifetime does not outlive its target",
            _ => "uninitialized or moved",
        };
        assert!(error.to_string().contains(expected), "{name}: {error}");
        println!("shared-call source opponent {name}: {error}");
    }
    for name in ["generic-value", "inferred-let", "copy-bound"] {
        let assignment = if name == "inferred-let" { 16 } else { 18 };
        let input = test_source::fixture(&format!(
            "pa{assignment}/tests/spec/shared-call-reborrow/{name}.rs"
        ))?;
        check(&input, false)?;
    }
    Ok(())
}
#[test]
fn repeated_reference_contexts_have_linear_sparse_adjustments() -> Result {
    let source = test_source::fixture("pa31/tests/spec/shared-call-reborrow/direct.rs")?;
    let prefix = source.split("fn read(").next().ok_or("source foundation")?;
    let mut previous = None;
    for count in [1, 8, 32, 128] {
        let references = format!(
            "{prefix}fn take(_: &u8){{}}fn f(r:&u8){{{}}}",
            "take(r);".repeat(count)
        );
        let (rows, steps) = check(&references, true)?;
        assert_eq!(rows, count);
        if let Some(previous) = previous {
            assert!(steps <= previous * 8);
        }
        previous = Some(steps);
        let scalar = format!(
            "{prefix}fn take(_: u8){{}}fn f(r:u8){{{}}}",
            "take(r);".repeat(count)
        );
        let (scalar_rows, scalar_steps) = check(&scalar, true)?;
        assert_eq!(scalar_rows, 0);
        println!(
            "shared-call scaling calls={count} reference_rows={count} scalar_rows=0 work={steps} scalar_work={scalar_steps} Body={} NodeFacts={}",
            size_of::<body::ir::Body>(),
            size_of::<crate::sema::model::NodeFacts>()
        );
    }
    Ok(())
}
fn write_native(input: &str, source: bool, name: &str) -> Result {
    let (
        report,
        pipeline::Analysis {
            mut facts,
            prepared,
        },
    ) = analyze(input, source)?;
    let program = crate::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        crate::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/pa16/shared-call-reborrow-v1/candidate-native/{name}"
    ));
    std::fs::create_dir_all(&folder)?;
    crate::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("provider.lowir"))?,
    )?;
    Ok(())
}
#[test]
fn ordinary_and_source_call_bodies_reach_native_producers() -> Result {
    for (assignment, source) in [(16, false), (31, true)] {
        for fixture in ["native", "native-returned"] {
            let input = test_source::fixture(&format!(
                "pa{assignment}/tests/spec/shared-call-reborrow/{fixture}.rs"
            ))?;
            check(&input, source)?;
            write_native(&input, source, &format!("pa{assignment}-{fixture}"))?;
        }
    }
    Ok(())
}
#[test]
fn actual_no_copy_formatting_source_reaches_original_native_input() -> Result {
    let input =
        test_source::fixture("pa31/tests/spec/source-format-macros/display/source-mixed.rs")?;
    write_native(&input, true, "original-mixed")
}

#[test]
fn source_unsize_conversions_keep_actual_trait_prerequisites() -> Result {
    for name in [
        "unsize-missing-roles",
        "only-unsize",
        "both-without-impl",
        "without-shared-impl",
    ] {
        let input =
            test_source::fixture(&format!("pa31/tests/spec/shared-call-reborrow/{name}.rs"))?;
        let error = check(&input, true)
            .err()
            .ok_or_else(|| format!("source unsizing prerequisite {name} was accepted"))?;
        let expected = if matches!(name, "unsize-missing-roles" | "only-unsize") {
            "actual source CoerceUnsized declaration"
        } else {
            "trait obligation"
        };
        assert!(error.to_string().contains(expected), "{name}: {error}");
        println!("source unsizing prerequisite {name}: {error}");
    }
    Ok(())
}

#[test]
fn explicit_reference_unsizing_casts_keep_the_same_source_prerequisites() -> Result {
    for name in [
        "missing",
        "only-unsize",
        "both-noimpl",
        "mut-only",
        "complete",
    ] {
        let input = test_source::fixture(&format!(
            "pa31/tests/spec/shared-call-reborrow/cast-{name}.rs"
        ))?;
        let result = check(&input, true);
        if name == "complete" {
            result?;
        } else {
            let error = result
                .err()
                .ok_or_else(|| format!("invalid cast foundation {name} was accepted"))?;
            assert!(
                error
                    .to_string()
                    .contains(if matches!(name, "missing" | "only-unsize") {
                        "actual source CoerceUnsized declaration"
                    } else {
                        "trait obligation"
                    }),
                "{name}: {error}"
            );
        }
    }
    Ok(())
}
