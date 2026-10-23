//! Actual constant preparation shares bounded query work across source requests.
use crate::{pipeline, test_source};
use std::{collections::BTreeMap, path::Path};
#[test]
fn source_requests_share_reachable_query_work_across_width_repeats_and_noise() -> Result<(), String>
{
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../pa31/tests/spec/needs-drop/work");
    let mut widths = BTreeMap::new();
    for width in [8, 32, 128] {
        for calls in [8, 32, 128] {
            let name = format!("width{width}-calls{calls}");
            let source = std::fs::read_to_string(root.join(format!("{name}.rs")))
                .map_err(|e| e.to_string())?;
            let report = test_source::source_provider(&source, "needs_drop_work")?;
            let analysis = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
            let context = analysis
                .prepared
                .constants
                .as_ref()
                .ok_or("constant context")?;
            let engine = context.engine.try_borrow().map_err(|e| e.to_string())?;
            let stats = engine
                .catalog
                .needs_drop_statistics()
                .ok_or("query never executed")?;
            assert_eq!(stats.2, 1, "{name}");
            assert!(stats.0 <= 4 * width + 8, "{name}: {stats:?}");
            if let Some(expected) = widths.get(&width) {
                assert_eq!(stats, *expected, "{name}");
            } else {
                widths.insert(width, stats);
            }
            println!(
                "{name}: steps={} query_bytes={} answers={}",
                stats.0, stats.1, stats.2
            );
        }
    }
    for noise in [64, 1024] {
        let name = format!("width8-calls8-noise{noise}");
        let source =
            std::fs::read_to_string(root.join(format!("{name}.rs"))).map_err(|e| e.to_string())?;
        let report = test_source::source_provider(&source, "needs_drop_work")?;
        let analysis = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
        let context = analysis.prepared.constants.as_ref().ok_or("context")?;
        let engine = context.engine.try_borrow().map_err(|e| e.to_string())?;
        let stats = engine.catalog.needs_drop_statistics().ok_or("query")?;
        assert_eq!(Some(&stats), widths.get(&8), "{name}");
        println!("{name}: {stats:?}");
    }
    for (name, queried) in [
        ("ordinary-unused-intrinsic", false),
        ("primitive-calls128", true),
    ] {
        let source =
            std::fs::read_to_string(root.join(format!("{name}.rs"))).map_err(|e| e.to_string())?;
        let report = test_source::source_provider(&source, "needs_drop_work")?;
        let analysis = pipeline::analyze_source(&report).map_err(|e| e.to_string())?;
        let context = analysis.prepared.constants.as_ref().ok_or("context")?;
        let engine = context.engine.try_borrow().map_err(|e| e.to_string())?;
        let stats = engine.catalog.needs_drop_statistics();
        assert_eq!(stats.is_some(), queried, "{name}");
        if let Some(stats) = stats {
            assert_eq!(stats.2, 1);
        }
        println!("{name}: {stats:?}");
    }
    Ok(())
}
