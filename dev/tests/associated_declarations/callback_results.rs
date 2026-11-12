//! Callback output equations precede inherent lookup on an inferred result.
use crate::{resolve as source, semantic};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

#[test]
fn callable_output_shapes_reach_consumers_without_waiving_projection_proof() -> Result {
    let mut failures = Vec::new();
    for owner in ["pa24", "pa31"] {
        for (name, accept) in [
            ("chain", true),
            ("split", true),
            ("annotated", true),
            ("qualified", true),
            ("explicit-constructor", true),
            ("closure", true),
            ("known-item", true),
            ("equal-projections", true),
            ("array-item", true),
            ("literal-call-input", true),
            ("parameter-call-input", true),
            ("wrong-output-reject", false),
            ("wrong-item-reject", false),
            ("different-projection-reject", false),
            ("missing-parent-reject", false),
        ] {
            let text = source::fixture(&format!(
                "{owner}/tests/spec/callable-result-heads/{name}.rs"
            ))?;
            let report = if owner == "pa31" {
                source::source_provider(&text, "callable_output_shapes")?
            } else {
                semantic::resolve(&text)?
            };
            let analysis = rustgm::pipeline::analyze_source(&report);
            if !accept {
                let cause = match name {
                    "wrong-item-reject" => {
                        "body types require a provable associated-result equality"
                    }
                    "missing-parent-reject" => {
                        "associated type requires an unambiguous implementation"
                    }
                    _ => "incompatible types",
                };
                if !analysis
                    .as_ref()
                    .is_err_and(|error| error.to_string().contains(cause))
                {
                    failures.push(format!(
                        "{owner}/{name}: expected {cause}, got {:?}",
                        analysis.err().map(|error| error.to_string())
                    ));
                }
                continue;
            }
            let outcome = analysis.and_then(|analysis| complete(&report, analysis));
            if let Err(error) = outcome {
                failures.push(format!("{owner}/{name}: {error}"));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

fn complete(
    report: &rustgm::resolve::expansion::Report,
    rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    }: rustgm::pipeline::Analysis,
) -> std::result::Result<rustgm::lowir::program::Program, rustgm::resolve::package::Error> {
    rustgm::lower::instances::build_roots_prepared(
        report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )
}

#[test]
fn nonempty_callable_binders_keep_their_actual_lifetime_requirements() -> Result {
    let mut failures = Vec::new();
    for owner in ["pa24", "pa31"] {
        for (name, reject) in [
            ("higher-ranked-input", false),
            ("static-input-reject", true),
        ] {
            let text = source::fixture(&format!(
                "{owner}/tests/spec/callable-result-heads/{name}.rs"
            ))?;
            let report = if owner == "pa31" {
                source::source_provider(&text, "nonempty_callable_binder")?
            } else {
                semantic::resolve(&text)?
            };
            let result = rustgm::pipeline::analyze_source(&report)
                .and_then(|analysis| complete(&report, analysis))
                .map(|_| ());
            match result {
                Ok(()) if !reject => {}
                Err(error)
                    if reject
                        && error
                            .to_string()
                            .contains("required lifetime does not outlive its target") => {}
                result => failures.push(format!(
                    "{owner}/{name}: expected reject={reject}, got {result:?}"
                )),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn projected_callback_results_scale_without_unrelated_type_work() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!(
                "pa31/tests/spec/callable-result-heads/width-{width}-noise-{noise}.rs"
            ))?;
            let report = source::source_provider(&text, "callback_result_scaling")?;
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let work = [
                analysis.facts.body_work.inference_steps,
                analysis.facts.body_work.trait_selections,
                analysis.facts.methods.selection_steps,
                analysis.facts.methods.selection_probes,
            ];
            complete(&report, analysis)?;
            println!("callback width={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(old) = previous {
                    for (before, actual) in std::iter::zip(old, work) {
                        assert!(actual <= 4 * before + 2048, "{old:?}->{work:?}");
                    }
                }
                previous = Some(work);
                baseline = Some(work);
            } else {
                assert_eq!(baseline, Some(work));
            }
        }
    }
    Ok(())
}

#[test]
fn input_reconciliation_defers_partial_projections_without_false_cycles() -> Result {
    use rustgm::types::{
        ItemId, Primitive, Store, TypeId,
        copy::Properties,
        infer::{Context, Domain, Error, arguments::Argument, refinement::Refinement},
        projection::Schema,
        substitute::Bound,
    };
    let mut store = Store::default();
    let item = ItemId(804);
    store.declare_projection(
        item,
        Schema {
            trait_item: ItemId(803),
            sized: true,
            copies: Properties::default(),
        },
    )?;
    let mut source = Context::default();
    let a = source.variable(Domain::Any)?;
    let b = source.variable(Domain::Any)?;
    let (mut query, roots) =
        Refinement::new(&store, &mut source, &[Argument::Type(a), Argument::Type(b)])?;
    let [Argument::Type(a), Argument::Type(b)] = roots.as_slice() else {
        return Err("missing imported input roots".into());
    };
    let unknown = query.context.variable(Domain::Any)?;
    let known = query
        .context
        .known(&store, TypeId::primitive(Primitive::U8))?;
    let partial = query.context.projected(&store, item, unknown, &[], &[])?;
    let complete = query.context.projected(&store, item, known, &[], &[])?;
    query.context.equate(&store, *a, partial)?;
    query.context.equate(&store, *b, complete)?;
    query
        .publish(&mut store, &mut source, &mut Bound::new(Vec::new()))
        .map_err(|error| format!("input publication: {error}"))?;
    assert_eq!(
        query.refresh(&mut store, &mut source),
        Err(Error::Normalization)
    );
    Ok(())
}

#[test]
fn generic_method_values_keep_input_correlations_and_actual_parent_proof() -> Result {
    let mut failures = Vec::new();
    for owner in ["pa24", "pa31"] {
        for (name, cause) in [
            ("chain-last", None),
            ("method-value-one", None),
            ("method-value-qualified", None),
            ("method-value-explicit-output", None),
            ("flatten-last", None),
            ("flatten-last-unfixed-accumulator", None),
            ("flatten-method-local", None),
            ("flatten-method-free", None),
            ("flatten-parameter-first", None),
            ("map-member", None),
            ("map-member-projected", None),
            ("map-associated", None),
            ("method-value-paid-parent", None),
            ("method-value-concrete-owner", None),
            ("method-value-wrong-result", Some("incompatible types")),
            (
                "method-value-unpaid-parent",
                Some(
                    "trait obligation requires an unambiguous matching assumption or implementation",
                ),
            ),
            (
                "method-value-wrong-owner",
                Some("associated type requires an unambiguous validated implementation"),
            ),
            (
                "method-value-concrete-owner-reject",
                Some("associated type requires an unambiguous validated implementation"),
            ),
        ] {
            let text = source::fixture(&format!(
                "{owner}/tests/spec/callable-result-heads/{name}.rs"
            ))?;
            let report = if owner == "pa31" {
                source::source_provider(&text, "method_value_inputs")?
            } else {
                semantic::resolve(&text)?
            };
            let result = rustgm::pipeline::analyze_source(&report)
                .and_then(|analysis| complete(&report, analysis))
                .map(|_| ());
            match (cause, result) {
                (None, Ok(())) => {}
                (Some(cause), Err(error)) if error.to_string().contains(cause) => {}
                (cause, result) => failures.push(format!("{owner}/{name}: {cause:?}: {result:?}")),
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn shared_projected_callback_inputs_scale_through_the_independent_anchor() -> Result {
    let mut previous = None;
    for width in [8, 32, 128] {
        let mut baseline = None;
        for noise in [0, 128] {
            let text = source::fixture(&format!(
                "pa31/tests/spec/callable-result-heads/multi-input-width-{width}-noise-{noise}.rs"
            ))?;
            let report = source::source_provider(&text, "shared_callback_inputs")?;
            let analysis = rustgm::pipeline::analyze_source(&report)?;
            let work = [
                analysis.facts.body_work.inference_steps,
                analysis.facts.body_work.trait_selections,
                analysis.facts.methods.selection_steps,
                analysis.facts.methods.selection_probes,
            ];
            complete(&report, analysis)?;
            println!("callback inputs={width} noise={noise} work={work:?}");
            if noise == 0 {
                if let Some(old) = previous {
                    for (before, actual) in std::iter::zip(old, work) {
                        assert!(actual <= 4 * before + 4096, "{old:?}->{work:?}");
                    }
                }
                previous = Some(work);
                baseline = Some(work);
            } else {
                assert_eq!(baseline, Some(work));
            }
        }
    }
    Ok(())
}
