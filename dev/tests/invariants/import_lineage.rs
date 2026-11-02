//! Alias-root sharing, authentic missing-edge rejection and pre-mutation payments.
use super::*;
use crate::resolve::{
    expansion::Report,
    load::{InputFile, Sources},
};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &std::path::Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn expand(source: &str) -> TestResult<Report> {
    Ok(crate::resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: crate::support::source::SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &crate::expand::config::Config::default(),
    )?)
}
#[test]
fn shared_export_chains_have_linear_paid_work_and_sparse_storage() -> TestResult {
    for width in [1, 8, 32, 128] {
        let mut source = String::from(
            "mod base { pub mod hidden {} pub mod visible {} } mod first { pub use super::base::*; mod hidden {} } mod second { pub use super::first::visible; }",
        );
        for index in 0..width {
            use std::fmt::Write;
            write!(source, "pub use second::visible as alias{index};")?;
        }
        let mut report = expand(&source)?;
        let mut work = report.imports.take().ok_or("completed expansion imports")?;
        let owner = report.resolution.imports[0].node;
        let before = work.lineage_budget;
        let exported = work.exported_globs(&report.resolution, owner)?;
        let steps = work.lineage_budget.steps - before.steps;
        let bytes = work.lineage_budget.bytes - before.bytes;
        assert_eq!(exported.len(), 1);
        assert!(steps < 80 * (width + 1));
        assert!(bytes < 2000 * (width + 1));
        let edges = work
            .lineage
            .as_deref()
            .ok_or("retained selections")?
            .edges
            .len();
        assert_eq!(edges, width + 3);
        println!(
            "import lineage width={width} edges={edges} trace_steps={steps} trace_paid_bytes={bytes} import_work={} import_paid_bytes={} Edge={} Source={} Lineage={} Work={} Level={} denseID={}",
            before.steps,
            before.bytes,
            size_of::<Edge>(),
            size_of::<Source>(),
            size_of::<Lineage>(),
            size_of::<Work>(),
            size_of::<crate::resolve::lint_policy::Level>(),
            size_of::<u32>()
        );
        assert_eq!(size_of::<crate::resolve::lint_policy::Level>(), 1);
        assert_eq!(size_of::<u32>(), 4);
    }
    Ok(())
}
#[test]
fn imported_missing_edges_never_become_direct_and_failed_work_remains_paid() -> TestResult {
    let mut report = expand("mod base { pub mod visible {} } pub use base::*;")?;
    let mut work = report.imports.take().ok_or("completed expansion imports")?;
    let owner = report.resolution.imports[0].node;
    let lineage = work.lineage.as_mut().ok_or("retained source selections")?;
    let (&edge, _) = lineage
        .edges
        .iter()
        .next()
        .ok_or("actual glob source edge")?;
    let binding = lineage
        .source(edge)
        .ok_or("actual source selection")?
        .binding;
    lineage.edges.remove(&edge);
    let before = work.lineage_budget.steps;
    let error = work
        .exported_globs(&report.resolution, owner)
        .err()
        .ok_or("accepted missing imported edge")?;
    assert!(error.to_string().contains("source lineage is required"));
    assert!(work.lineage_budget.steps > before);
    work.lineage_budget.steps = 10_000_000;
    let error = work
        .record_source(owner, edge.0, edge.1 as usize, binding, None)
        .err()
        .ok_or("accepted unpaid lineage publication")?;
    assert!(error.to_string().contains("work budget exceeded"));
    assert!(work.lineage_budget.steps > 10_000_000);
    assert!(
        !work
            .lineage
            .as_deref()
            .ok_or("retained selections")?
            .edges
            .contains_key(&edge)
    );
    Ok(())
}

#[test]
fn glob_fanout_shares_exact_source_snapshots_with_linear_paid_storage() -> TestResult {
    use std::fmt::Write;
    for width in [1, 8, 32, 128] {
        let mut source = String::from("mod base {");
        for index in 0..width {
            write!(source, "pub mod item{index} {{}}")?;
        }
        source.push('}');
        for index in 0..width {
            write!(
                source,
                "pub mod fanout{index} {{ pub use super::base::*; }}"
            )?;
        }
        let mut report = expand(&source)?;
        let work = report.imports.take().ok_or("completed expansion imports")?;
        let lineage = work.lineage.as_deref().ok_or("retained glob selections")?;
        let edges = width * width;
        assert_eq!(lineage.edges.len(), edges);
        assert_eq!(lineage.sources.len(), width);
        assert_eq!(lineage.selections.len(), width);
        assert!(work.lineage_budget.bytes < 300 * edges + 4000 * width);
        assert!(work.lineage_budget.steps < 32 * edges + 100 * width);
        for (&edge, &index) in &lineage.edges {
            let selected = lineage.source(edge).ok_or("actual selected source")?;
            assert_eq!(lineage.selections.get(selected), Some(&index));
            assert_eq!(selected.key.map(|key| key.2), Some(edge.0.2));
        }
        println!(
            "import fanout width={width} edges={edges} sources={} paid_bytes={} work={} edge_slot={} source_slot={}",
            lineage.sources.len(),
            work.lineage_budget.bytes,
            work.lineage_budget.steps,
            size_of::<(Edge, u32)>(),
            size_of::<Source>(),
        );
    }
    Ok(())
}

#[test]
fn sharing_keeps_old_selections_and_failed_payments_do_not_publish() -> TestResult {
    let mut report = expand(
        "mod base { pub mod visible {} } pub use base::*; pub mod other { pub use super::base::*; }",
    )?;
    let mut work = report.imports.take().ok_or("completed expansion imports")?;
    let owner = report.resolution.imports[0].node;
    let lineage = work.lineage.as_mut().ok_or("retained source selections")?;
    assert_eq!(lineage.sources.len(), 1);
    let edges: Vec<_> = lineage.edges.keys().copied().collect();
    assert_eq!(edges.len(), 2);
    let original = *lineage.source(edges[0]).ok_or("original selection")?;
    let changed = Source {
        binding: Binding {
            ambiguous: true,
            ..original.binding
        },
        ..original
    };
    let before = work.lineage_budget;
    let mut exhausted = super::super::super::lint_policy::producer::Work {
        bytes: 64 * 1024 * 1024,
        ..Default::default()
    };
    let mut empty = Lineage::default();
    let error = empty
        .record(owner, edges[0], original, &mut exhausted)
        .err()
        .ok_or("accepted unpaid source intern")?;
    assert!(error.to_string().contains("storage budget exceeded"));
    assert!(exhausted.bytes > 64 * 1024 * 1024);
    assert!(empty.sources.is_empty() && empty.selections.is_empty() && empty.edges.is_empty());
    lineage.record(owner, edges[0], changed, &mut work.lineage_budget)?;
    assert_eq!(lineage.sources.len(), 2);
    assert!(
        lineage
            .source(edges[0])
            .ok_or("changed selection")?
            .binding
            .ambiguous
    );
    assert!(
        !lineage
            .source(edges[1])
            .ok_or("retained selection")?
            .binding
            .ambiguous
    );
    assert!(work.lineage_budget.steps > before.steps);
    let error = work
        .exported_globs(&report.resolution, owner)
        .err()
        .ok_or("accepted changed source snapshot")?;
    assert!(
        error
            .to_string()
            .contains("completed import source selection changed")
    );
    Ok(())
}
