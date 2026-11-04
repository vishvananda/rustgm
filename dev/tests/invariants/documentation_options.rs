//! Documentation permissions never enable cfg filtering or other language gates.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{diagnostics::LintCap, edition::Edition, source::SourceFile},
};
use std::{fmt::Write as _, path::Path};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, config: &Config) -> Result<Report> {
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        InputFile {
            identity: "documentation.rs".into(),
            path: "documentation.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        config,
    )?)?)
}
#[test]
fn documentation_gates_match_independent_source_and_macro_outcomes() -> Result {
    assert_eq!(size_of::<super::Features>(), 8);
    let cases = crate::test_source::fixture("pa40/tests/spec/documentation-gates/cases-v2.tsv")?;
    let mut count = 0;
    for row in cases.lines() {
        let columns: Vec<_> = row.split('\t').collect();
        let [name, path, status] = columns.as_slice() else {
            return Err("malformed documentation gate case".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let mut config = Config::in_edition(edition);
            config.set_lint_cap(LintCap::Allow);
            let actual = (|| -> Result {
                let report = report(&source, &config)?;
                let facts = crate::sema::declarations::analyze(&report)?;
                crate::pipeline::analyze(&report, facts)?;
                Ok(())
            })();
            assert_eq!(
                actual.is_ok(),
                *status == "0",
                "{name} {edition:?}: {:?}",
                actual.err()
            );
        }
        count += 1;
    }
    assert_eq!(count, 73);
    Ok(())
}
#[test]
fn documentation_predicates_do_not_filter_program_owners_or_satisfy_const_proofs() -> Result {
    let mut config = Config::default();
    config.set_lint_cap(LintCap::Allow);
    for predicate in ["any()", "all()", "false", "true", "feature=\"missing\""] {
        let source =
            format!("#![feature(doc_cfg)]#[doc(cfg({predicate}))]fn answer()->u8{{missing}}\n");
        let actual = (|| -> Result {
            let report = report(&source, &config)?;
            let facts = crate::sema::declarations::analyze(&report)?;
            crate::pipeline::analyze(&report, facts)?;
            Ok(())
        })();
        assert!(
            actual.is_err(),
            "documentation predicate filtered a required body"
        );
    }
    for attribute in ["#[doc(cfg(any()))]", ""] {
        let source = format!(
            "#![feature(doc_cfg)]{attribute}fn ordinary()->u8{{7}}fn main(){{let _=const{{ordinary()}};}}"
        );
        let actual = (|| -> Result {
            let mut report = report(&source, &config)?;
            let crate::pipeline::Analysis {
                mut facts,
                prepared,
            } = crate::pipeline::analyze_source(&report)?;
            let entry = crate::sema::entry::select(&mut report, &facts)?;
            crate::lower::instances::build_prepared(&report, &mut facts, entry, prepared)?;
            Ok(())
        })();
        let Err(error) = actual else {
            return Err(
                "documentation metadata enabled a non-const call in a demanded constant".into(),
            );
        };
        assert!(error.to_string().contains("const"), "{error}");
    }
    Ok(())
}
#[test]
fn repeated_documentation_headers_share_paid_macro_allowance_work() -> Result {
    fn usage(width: usize, declarations: usize) -> Result<usize> {
        let list = std::iter::once("doc_cfg".to_owned())
            .chain((1..width).map(|i| format!("fixture_feature_{i}")))
            .collect::<Vec<_>>()
            .join(",");
        let mut source = format!(
            "#![feature(allow_internal_unstable)]#[allow_internal_unstable({list})]macro_rules! make{{($name:ident)=>{{#[doc(cfg(any()))]fn $name()->u8{{7}}}}}}"
        );
        for i in 0..declarations {
            write!(source, "make!(f{i});")?;
        }
        let mut config = Config::default();
        config.set_lint_cap(LintCap::Allow);
        let report = report(&source, &config)?;
        let context = report
            .package
            .macro_context()?
            .ok_or("actual macro context")?;
        let before = context.contracts().usage().0;
        crate::sema::declarations::analyze(&report)?;
        Ok(context
            .contracts()
            .usage()
            .0
            .checked_sub(before)
            .ok_or("work order")?)
    }
    let base = usage(8, 32)?.checked_sub(usage(8, 1)?).ok_or("base work")?;
    for width in [32, 128] {
        let growth = usage(width, 32)?
            .checked_sub(usage(width, 1)?)
            .ok_or("work growth")?;
        assert!(growth <= base + 256 * 31, "width{width}: {growth}/{base}");
    }
    Ok(())
}
#[test]
fn uncapped_documentation_diagnostics_remain_an_explicit_boundary() -> Result {
    for name in ["cfg-false", "cfg-auto-hide", "notable-trait"] {
        let source =
            crate::test_source::fixture(&format!("pa40/tests/spec/documentation-gates/{name}.rs"))?;
        let report = report(&source, &Config::default())?;
        let Err(error) = crate::sema::declarations::analyze(&report) else {
            return Err("uncapped documentation profile unexpectedly accepted".into());
        };
        assert!(
            error
                .to_string()
                .contains("uncapped documentation option diagnostics")
        );
    }
    Ok(())
}

#[test]
fn shared_cfg_inventory_validates_all_leaves_in_capture_order() -> Result {
    use crate::syntax::{configuration, token_tree::Forest};
    let source = "all(false,ub_checks,not(any(target_has_reliable_f16,sanitize=\"cfi\")),target_has_atomic_load_store=\"8\")";
    let forest = Forest::parse(source).map_err(|error| error.message)?;
    let uses = configuration::documentation(&forest, source, forest.roots(), &Edition::Rust2024)
        .map_err(|error| error.message)?;
    let names: Vec<_> = uses
        .iter()
        .map(|used| &source[used.span.start as usize..used.span.end as usize])
        .collect();
    assert_eq!(
        names,
        [
            "ub_checks",
            "target_has_reliable_f16",
            "sanitize",
            "target_has_atomic_load_store"
        ]
    );
    let malformed = "all(false,wrong())";
    let forest = Forest::parse(malformed).map_err(|error| error.message)?;
    assert!(
        configuration::documentation(&forest, malformed, forest.roots(), &Edition::Rust2024)
            .is_err()
    );
    Ok(())
}
