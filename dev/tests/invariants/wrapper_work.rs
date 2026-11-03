//! Result traversal bounds shared graphs, compressed arrays and failed temporary storage.
use super::*;

#[test]
fn terminal_results_need_no_graph_storage_and_still_pay_validation_work() -> Result<(), String> {
    with_storage(
        0,
        |_| Ok(TypeId::primitive(Primitive::Usize)),
        |machine, ty| {
            machine.validate_result(scalar(7), ty)?;
            assert_eq!(machine.slots, 0);
            assert_eq!(machine.work.peak_slots, 0);
            assert_eq!(machine.work.steps, 1);
            Ok(())
        },
    )
}

#[test]
fn typed_result_validation_scales_with_shared_nodes_and_preserves_compressed_repeats()
-> Result<(), String> {
    let mut observations = Vec::new();
    for depth in [8, 32, 128] {
        with_storage(
            1 << 20,
            |types| {
                let mut ty = TypeId::primitive(Primitive::U8);
                for _ in 0..depth {
                    let fields = types.intern_list(&[ty, ty])?;
                    ty = types.intern(Kind::Tuple(fields))?;
                }
                Ok(ty)
            },
            |machine, ty| {
                let mut value = scalar(7);
                for _ in 0..depth {
                    value = machine.evaluator.fields(&[value, value], None)?;
                }
                machine.validate_result(value, ty)?;
                assert_eq!(machine.slots, 0);
                observations.push((machine.work.steps, machine.work.peak_slots));
                Ok(())
            },
        )?;
    }
    for pair in observations.windows(2) {
        assert!(pair[1].0 < pair[0].0 * 6, "{observations:?}");
        assert!(pair[1].1 < pair[0].1 * 6, "{observations:?}");
    }
    let mut repeats = Vec::new();
    for length in [1, 1_000_000] {
        with_array(length, 4096, |machine, ty| {
            let value = machine.evaluator.repeat(scalar(7), length)?;
            machine.validate_result(value, ty)?;
            assert_eq!(machine.slots, 0);
            repeats.push((machine.work.steps, machine.work.peak_slots));
            Ok(())
        })?;
    }
    assert_eq!(repeats[0], repeats[1]);
    eprintln!(
        "typed result graph depths8/32/128 steps/slots={observations:?}; repeat lengths1/1000000={repeats:?}"
    );
    Ok(())
}

#[test]
fn failed_result_work_restores_temporary_slots_without_refunding_attempted_work()
-> Result<(), String> {
    with_array(4, 1, |machine, ty| {
        let value = machine.evaluator.repeat(scalar(7), 4)?;
        let before = machine.work.steps;
        assert!(machine.validate_result(value, ty).is_err());
        assert_eq!(machine.slots, 0);
        assert!(machine.work.steps > before);
        machine.limits.steps = machine.work.steps;
        assert!(machine.validate_result(value, ty).is_err());
        assert_eq!(machine.slots, 0);
        assert!(machine.memory.bytes.roots.is_empty());
        Ok(())
    })
}
