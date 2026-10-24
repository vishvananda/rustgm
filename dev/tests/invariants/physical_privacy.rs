//! Real typed signatures share graph work instead of flattening repeated children.
use super::*;
use std::fmt::Write;
type TestResult = Result<(), Box<dyn std::error::Error>>;

#[test]
fn actual_shared_signatures_have_linear_privacy_work_and_constant_repeated_use() -> TestResult {
    for depth in [8_usize, 32, 128] {
        let mut source = String::from("pub struct Leaf; type T0=Leaf;");
        for index in 1..=depth {
            write!(
                source,
                "type T{index}=(*const T{},*const T{});",
                index - 1,
                index - 1
            )?;
        }
        write!(source, "pub fn read(_: &T{depth})->u32{{7u32}}")?;
        let report = crate::test_source::resolve(&source, true)?;
        let crate::pipeline::Analysis { facts, .. } = crate::pipeline::analyze_source(&report)?;
        let function = facts
            .functions
            .iter()
            .find(|function| {
                let item = &report.resolution.definitions[function.definition.0 as usize];
                report.package.names.get(item.name) == "read"
            })
            .ok_or("actual public signature missing")?;
        let mut privacy = Privacy::new(&report, &facts, ModuleId(0));
        privacy.check(function.signature)?;
        assert!(privacy.steps <= depth * 8 + 20, "{}", privacy.steps);
        assert!(privacy.visited.len() <= depth * 4 + 8);
        let initial = privacy.steps;
        for _ in 0..128 {
            privacy.check(function.signature)?;
        }
        assert_eq!(privacy.steps - initial, 128);
        assert!(privacy.work.is_empty());
        assert!(privacy.ancestors.is_none());
        println!(
            "privacy depth={depth} unique={} initial_steps={initial}",
            privacy.visited.len()
        );
    }
    Ok(())
}

#[test]
fn primitive_privacy_uses_allocate_no_graph_and_failures_still_consume_work() -> TestResult {
    let report = crate::test_source::resolve("pub fn answer()->u32{7}", true)?;
    let crate::pipeline::Analysis { facts, .. } = crate::pipeline::analyze_source(&report)?;
    let mut privacy = Privacy::new(&report, &facts, ModuleId(0));
    for _ in 0..128 {
        privacy.check(TypeId::primitive(crate::types::Primitive::U32))?;
    }
    assert_eq!(privacy.visited.capacity(), 0);
    assert_eq!(privacy.work.capacity(), 0);
    assert_eq!(privacy.opaques.capacity(), 0);
    assert!(privacy.ancestors.is_none());
    let initial = privacy.steps;
    assert!(privacy.check(TypeId(u32::MAX)).is_err());
    assert_eq!(privacy.steps, initial + 1);
    privacy.steps = 1_000_000;
    assert!(privacy.check(TypeId::UNIT).is_err());
    assert_eq!(privacy.steps, 1_000_001);
    Ok(())
}
