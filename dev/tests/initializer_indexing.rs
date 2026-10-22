//! One compact append-time initializer index serves all typed value consumers.
use crate::resolve as source;
use rustgm::{pipeline, resolve::model::DefId, sema::values::Evaluator};
#[test]
fn initializer_index_is_shared_compact_and_counts_cached_reads() -> Result<(), String> {
    for width in [8, 32, 128] {
        use std::fmt::Write;
        let mut text = String::new();
        for index in 0..width {
            write!(text, "const C{index}:u32={index};").map_err(|e| e.to_string())?;
        }
        let report = source::resolve(&text, true)?;
        let mut facts = pipeline::analyze_source(&report)
            .map_err(|e| e.to_string())?
            .facts;
        assert_eq!(facts.initializers.len(), width);
        assert_eq!(facts.constant_work.initializer_indexed, width);
        assert_eq!(facts.initializer_index_bytes(), 4 * facts.definitions.len());
        let order = facts
            .initializers
            .iter()
            .map(|value| value.definition)
            .collect::<Vec<_>>();
        for (index, &definition) in order.iter().enumerate() {
            assert_eq!(
                report
                    .package
                    .names
                    .get(report.resolution.definitions[definition.0 as usize].name),
                format!("C{index}")
            );
        }
        let before = facts.constant_work.initializer_lookups.get();
        {
            let mut evaluator = Evaluator::new(&report, &mut facts);
            for _ in 0..3 {
                for &definition in &order {
                    evaluator
                        .definition(definition)
                        .map_err(|e| e.to_string())?;
                }
            }
            assert!(evaluator.definition(DefId(u32::MAX)).is_err());
        }
        assert_eq!(
            facts.constant_work.initializer_lookups.get() - before,
            3 * width + 1
        );
        assert_eq!(facts.constant_work.initializer_indexed, width);
        assert_eq!(facts.initializer_index_bytes(), 4 * facts.definitions.len());
    }
    Ok(())
}
#[test]
fn ordinary_functions_and_missing_demands_allocate_no_initializer_index() -> Result<(), String> {
    let report = source::resolve("fn ordinary(n:u32)->u32{n+1}fn main(){}", true)?;
    let mut facts = pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    assert!(facts.initializers.is_empty());
    assert_eq!(facts.initializer_index_bytes(), 0);
    let before = facts.constant_work.initializer_lookups.get();
    assert!(
        Evaluator::new(&report, &mut facts)
            .definition(DefId(u32::MAX))
            .is_err()
    );
    assert_eq!(facts.constant_work.initializer_lookups.get(), before + 1);
    assert_eq!(facts.initializer_index_bytes(), 0);
    Ok(())
}
