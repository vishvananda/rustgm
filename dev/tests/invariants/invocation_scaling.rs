//! Actual generic caller instances expose the one charged descriptor preparation pass.
use super::*;
use crate::{
    body::ir::Owner, const_eval::use_inputs::Inputs, pipeline::execution::materialized_bodies,
    resolve::model::DefId, sema::values::execution::Target,
};
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn reference_caller_width_and_instances_keep_one_paid_pass_and_lazy_descriptor_storage()
-> TestResult {
    for width in [8, 32, 128] {
        let source = crate::test_source::fixture(&format!(
            "pa31/tests/spec/const-eval-select-ctfe/caller-scaling-{width}.rs"
        ))?;
        let report = crate::test_source::source_provider(&source, "actual_caller_width")?;
        let definition = |name: &str| -> Result<DefId, Box<dyn std::error::Error>> {
            let index = report
                .resolution
                .definitions
                .iter()
                .position(|definition| report.package.names.get(definition.name) == name)
                .ok_or("actual scaling definition")?;
            Ok(DefId(u32::try_from(index)?))
        };
        let targets = [
            "reference_one",
            "reference_many",
            "closed_one",
            "closed_many",
            "ordinary_one",
            "ordinary_many",
        ]
        .map(definition)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()?;
        let (analysis, bodies) = materialized_bodies::instances(&targets, 15, || {
            crate::pipeline::analyze_source(&report)
        })?;
        let crate::pipeline::Analysis {
            mut facts,
            prepared,
        } = analysis;
        let mut observed = [0; 6];
        for body in &bodies {
            let Owner::Function { definition, .. } = body.owner else {
                return Err("actual caller owner".into());
            };
            let profile = targets
                .iter()
                .position(|&target| target == definition)
                .ok_or("target")?;
            observed[profile] += 1;
            assert!(
                body.instance.is_some(),
                "full actual caller instance must be retained"
            );
            check_preparation(&mut facts, body, width, profile)?;
        }
        assert_eq!(observed, [1, 4, 1, 4, 1, 4]);
        let context = prepared.constants.as_ref().ok_or("provider")?;
        context.inspect_const_call(
            Target::Initializer(definition("N")?),
            |catalog, inputs, _| -> TestResult {
                let mut retained = [0; 6];
                for (&instance, &index) in &catalog.functions {
                    let Some(Kind::FunctionItem { item, .. }) = facts.types.kind(instance.ty())
                    else {
                        continue;
                    };
                    let Some(profile) = targets.iter().position(|target| target.0 == item.0) else {
                        continue;
                    };
                    if profile < 4 {
                        let row = inputs.rows.get(&index).ok_or("selected caller Row")?;
                        row.validate(index, catalog.code(index).ok_or("Code")?)?;
                        assert_eq!(row.selectors.is_some(), profile < 2);
                        retained[profile] += 1;
                    } else {
                        assert!(
                            !inputs.rows.contains_key(&index),
                            "ordinary caller retains no input Row"
                        );
                    }
                }
                assert_eq!(retained, [1, 4, 1, 4, 0, 0]);
                Ok(())
            },
        )??;
    }
    Ok(())
}

fn check_preparation(
    facts: &mut Declarations,
    body: &crate::body::ir::Body,
    width: usize,
    profile: usize,
) -> TestResult {
    let before = facts.body_work.canonical_steps;
    let uses = Inputs::prepare_uses(facts, body, 64 * 1024 * 1024)?;
    let work = facts.body_work.canonical_steps - before;
    let (per_call, filter_work) = match profile {
        0 | 1 => (14, 8),
        2 | 3 => (7, 2),
        _ => (0, 0),
    };
    assert_eq!(work, body.values.len() + width * per_call);
    assert_eq!(uses.descriptor_work, width * filter_work);
    assert_eq!(uses.needed, profile < 4);
    if profile < 2 {
        let rows = uses.selectors.as_deref().ok_or("reference descriptors")?;
        assert_eq!(rows.calls.len(), width);
        assert!(
            rows.bytes()
                <= size_of::<Rows>() + crate::support::retained::new_map::<ValueId, Opening>(width)
        );
    } else {
        assert!(
            uses.selectors.is_none(),
            "closed and ordinary paths allocate no descriptor table"
        );
    }
    println!(
        "width={width} profile={profile} instance={:?} values={} work={work} descriptor-work={} payload={}",
        body.instance,
        body.values.len(),
        uses.descriptor_work,
        uses.selectors.as_deref().map_or(0, Rows::bytes)
    );
    Ok(())
}
