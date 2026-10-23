//! The attribute name, not its wrapper or item, owns edition-sensitive safety.
use super::{Attribute, FileId, Forest, Report, safety};
use crate::{
    expand::config::Config,
    resolve::{
        expansion,
        load::{InputFile, Sources},
        unit::Unit,
    },
    support::{edition::Edition, source::SourceFile},
    syntax::token_tree::Span,
};
use std::path::Path;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _path: &Path) -> std::result::Result<Option<InputFile>, String> {
        Err("unexpected source read in linkage provenance control".into())
    }
}

fn generated(report: &Report) -> Result<(Attribute, &Forest)> {
    let unit = &report.package.units[0];
    for owner in &unit.configured.owners {
        for (attribute, forest) in unit.retained_attributes(owner.attributes.clone()) {
            if attribute.span.start as usize >= unit.source.physical_len()
                && attribute.name(forest, unit.source.text()) == Some("no_mangle")
            {
                return Ok((attribute, forest));
            }
        }
    }
    Err("missing generated linkage attribute".into())
}

fn prepared(wrapper: Edition, name: Edition) -> Result<Report> {
    let source =
        b"macro_rules! make { ($a:ident) => { #[$a] pub fn value() {} } } make!(no_mangle);";
    let input = InputFile {
        path: "/fixture/main.rs".into(),
        identity: "/fixture/main.rs".into(),
        source: SourceFile::new(source.to_vec())?,
    };
    let mut report = expansion::analyze(input, &mut NoFiles, &Config::in_edition(wrapper))?;
    let (attribute, forest) = generated(&report)?;
    let token = forest.token(attribute.path.start).ok_or("missing name")?;
    let donor = Unit::parse(
        SourceFile::new(b"#[no_mangle] fn donor() {}".to_vec())?,
        &Config::in_edition(name),
        &mut report.package.names,
    )
    .map_err(|error| format!("{error:?}"))?;
    let origin = report.package.units[0]
        .origins
        .iter_mut()
        .find(|origin| {
            origin.generated
                == Span {
                    start: token.start,
                    end: token.end,
                }
        })
        .ok_or("name token must have its own exact origin")?;
    origin.original.file = FileId(1);
    origin.original.span = Span { start: 2, end: 11 };
    report.package.units.push(donor);
    Ok(report)
}

#[test]
fn generated_linkage_name_selects_edition_independently_of_the_wrapper() -> Result {
    for wrapper in [Edition::Rust2021, Edition::Rust2024] {
        for name in [Edition::Rust2021, Edition::Rust2024] {
            let report = prepared(wrapper, name)?;
            let (attribute, forest) = generated(&report)?;
            assert_eq!(
                safety(&report, FileId(0), forest, attribute, "no_mangle").is_ok(),
                name == Edition::Rust2021,
                "wrapper {wrapper:?}, name {name:?}"
            );
        }
    }
    Ok(())
}

#[test]
fn missing_demanded_name_origin_cannot_fall_back_to_the_package_edition() -> Result {
    let mut report = prepared(Edition::Rust2021, Edition::Rust2021)?;
    report.package.units[0].origins.clear();
    let (mut attribute, forest) = generated(&report)?;
    let error = safety(&report, FileId(0), forest, attribute, "no_mangle")
        .err()
        .ok_or("missing origin must reject")?;
    assert!(
        error
            .to_string()
            .contains("missing exact token edition origin")
    );
    // The explicit wrapper is edition-independent and needs no origin query.
    attribute.unsafe_wrapper = true;
    safety(&report, FileId(0), forest, attribute, "no_mangle")?;
    Ok(())
}
