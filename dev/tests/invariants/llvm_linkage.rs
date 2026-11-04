//! Source authority and reserved symbol identity survive ordinary checked demand.
use crate::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    support::{
        diagnostics::LintCap,
        edition::Edition,
        linkage::SymbolNames,
        llvm_symbols::{Intrinsic, Operation},
        source::SourceFile,
    },
};
use std::path::Path;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> std::result::Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn report(source: &str, edition: Edition) -> Result<Report> {
    let mut config = Config::in_edition(edition);
    config.set_lint_cap(LintCap::Allow);
    let input = InputFile {
        identity: "llvm.rs".into(),
        path: "llvm.rs".into(),
        source: SourceFile::new(source.as_bytes().to_vec())?,
    };
    Ok(resolve::analyze_expanded(resolve::expansion::analyze(
        input,
        &mut NoFiles,
        &config,
    )?)?)
}
fn program(source: &str) -> Result<crate::lowir::program::Program> {
    let mut report = report(source, Edition::Rust2024)?;
    let analysis = crate::pipeline::analyze_source(&report)?;
    let mut facts = analysis.facts;
    let entry = crate::sema::entry::select(&mut report, &facts)?;
    Ok(crate::lower::instances::build_prepared(
        &report,
        &mut facts,
        entry,
        analysis.prepared,
    )?)
}
#[test]
fn source_gates_match_independent_foreign_item_and_macro_authority() -> Result {
    let inventory = crate::test_source::fixture("pa28/tests/spec/llvm-linkage/cases-v2.tsv")?;
    let mut rows = 0;
    for (ordinal, row) in inventory.lines().enumerate() {
        let parts: Vec<_> = row.split('\t').collect();
        let [name, path, status] = parts.as_slice() else {
            return Err("invalid LLVM source inventory".into());
        };
        let source = crate::test_source::fixture(path)?;
        for edition in [Edition::Rust2021, Edition::Rust2024] {
            let actual = report(&source, edition).and_then(|report| {
                crate::pipeline::analyze_source(&report)
                    .map(|_| ())
                    .map_err(Into::into)
            });
            assert_eq!(
                actual.is_ok(),
                *status == "0",
                "case {ordinal} {name} {edition:?}: {:?}",
                actual.err()
            );
        }
        rows += 1;
    }
    assert_eq!(rows, 43);
    Ok(())
}
#[test]
fn fixed_unary_kernels_are_shared_per_callable_and_leave_no_llvm_imports() -> Result {
    for name in ["native", "native-signed", "native-mixed"] {
        let source =
            crate::test_source::fixture(&format!("pa28/tests/spec/llvm-linkage/{name}.rs"))?;
        let program = program(&source)?;
        assert!(
            program
                .symbols
                .iter()
                .all(|symbol| !symbol.name.starts_with("llvm."))
        );
        assert_eq!(
            program
                .symbols
                .iter()
                .filter(|symbol| symbol.name.contains("rg.llvm.unary.t"))
                .count(),
            14
        );
        let mut output = Vec::new();
        crate::lowir::writer::write(&program, &mut output)?;
        assert!(!output.is_empty());
    }
    Ok(())
}
#[test]
fn unsupported_operations_storage_and_wrong_signatures_cannot_fall_back_to_c() {
    for item in [
        "#[link_name=\"llvm.rustgm.unknown\"]fn operation(x:u32)->u32;",
        "#[link_name=\"llvm.ctpop.i32\"]fn operation(x:u64)->u32;",
        "#[link_name=\"llvm.ctpop.i64\"]fn operation(x:u32)->u32;",
    ] {
        let source = format!(
            "#![feature(link_llvm_intrinsics)]unsafe extern \"C\"{{{item}}}fn main(){{unsafe{{let _=operation(7);}}}}"
        );
        assert!(
            program(&source).is_err(),
            "invalid selected LLVM operation escaped: {item}"
        );
    }
    assert!(program("#![feature(link_llvm_intrinsics)]unsafe extern \"C\"{#[link_name=\"llvm.not_a_global\"]static VALUE:u32;fn observe(x:u32);}fn main(){unsafe{observe(VALUE);}}").is_err());
}
#[test]
fn reserved_symbol_catalog_is_cold_and_paid_without_alias_duplication() -> Result {
    let mut names = SymbolNames::default();
    let ordinary = names.intern("ordinary")?;
    assert_eq!(names.llvm(ordinary), None);
    let mut total = crate::support::retained::Growth::default();
    let id = names.intern_paid("llvm.ctpop.i32", &mut |growth| {
        total.include(growth);
        Ok(())
    })?;
    assert_eq!(
        names.llvm(id),
        Some(Intrinsic {
            operation: Operation::Population,
            bits: 32
        })
    );
    let bytes = total.bytes;
    for _ in 0..512 {
        assert_eq!(
            names.intern_paid("llvm.ctpop.i32", &mut |growth| {
                total.include(growth);
                Ok(())
            })?,
            id
        );
    }
    assert_eq!(total.bytes, bytes);
    assert_eq!(names.len(), 2);
    let unknown = names.intern("llvm.unknown.i32")?;
    assert_eq!(
        names.llvm(unknown).map(|intrinsic| intrinsic.operation),
        Some(Operation::Unsupported)
    );
    assert_eq!(Intrinsic::named("ordinary"), None);
    Ok(())
}
#[test]
fn extended_global_permissions_keep_bit_boundaries_distinct_and_macro_masks_small() {
    assert_eq!(size_of::<super::Features>(), 16);
    assert_eq!(align_of::<super::Features>(), 8);
    assert_eq!(size_of::<super::MacroFeatures>(), 1);
    let mut features = super::Features::default();
    features.insert(super::Feature::LinkCfg);
    assert!(!features.has(super::Feature::LinkLlvmIntrinsics));
    features.insert(super::Feature::LinkLlvmIntrinsics);
    assert!(features.has(super::Feature::LinkCfg));
    assert!(features.has(super::Feature::LinkLlvmIntrinsics));
    assert!(!features.has(super::Feature::NoCore));
    assert!(!features.has(super::Feature::AbiUnadjusted));
}
