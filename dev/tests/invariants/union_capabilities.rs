//! Opaque union transport never demands a value from an inactive alternative.
use super::*;
use crate::{
    const_eval::Limits,
    sema::values::Evaluator,
    types::{GenericArgs, Kind, aggregate},
};
use std::fmt::Write as _;
#[test]
fn union_byte_capability_is_shared_without_reading_inactive_uninhabited_fields()
-> Result<(), Box<dyn std::error::Error>> {
    let mut cold = Vec::new();
    for width in [8, 32, 128] {
        let mut source =
            String::from("#[derive(Copy,Clone)]enum Void{}union U{empty:(),never:Void,");
        for index in 0..width {
            write!(&mut source, "f{index}:[u8;{}],", index + 1)?;
        }
        source.push('}');
        let report = crate::test_source::resolve(&source, true)?;
        let mut facts = crate::pipeline::analyze_source(&report)?.facts;
        let item = facts
            .aggregates
            .iter()
            .find(|d| d.kind == aggregate::Kind::Union)
            .ok_or("union owner")?
            .item;
        let ty = facts.types.intern(Kind::Adt(item, GenericArgs::EMPTY))?;
        let evaluator = Evaluator::new(&report, &mut facts);
        let mut machine = Machine::new(evaluator, std::iter::empty(), Limits::default())?;
        machine.footprint(ty)?;
        let before = machine.work.steps;
        assert!(machine.byte_family(ty)?);
        cold.push(machine.work.steps - before);
        assert_eq!(machine.memory.bytes.families.len(), 1);
        let retained = machine.memory.retained_cells();
        let before = machine.work.steps;
        for _ in 0..128 {
            assert!(machine.byte_family(ty)?);
        }
        assert_eq!(machine.work.steps - before, 128);
        assert_eq!(machine.memory.retained_cells(), retained);
        eprintln!(
            "union width={width} cold={} warm=128 retained={retained}",
            cold.last().ok_or("work")?
        );
    }
    assert!(cold.windows(2).all(|pair| pair[0] == pair[1]), "{cold:?}");
    Ok(())
}
