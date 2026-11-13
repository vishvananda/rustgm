//! Projections a body meets before any normalization are normalized like the
//! signature's. Core's portable-simd `first_set` (`masks.rs`) annotates
//! `let min_index: T::Unsigned`, which no signature names. Its `try_from`
//! (`vector.rs`) applies `?` to `slice.try_into()`, whose error type is still a
//! projection when the operand is coerced.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/body-projection-normalization";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "body_projection_normalization")?;
    let rustgm::pipeline::Analysis {
        mut facts,
        prepared,
    } = rustgm::pipeline::analyze_source(&report)?;
    Ok(rustgm::lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        rustgm::lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}

/// The CLI's typing status and its stderr.
fn typed(path: &std::path::Path) -> Result<(bool, String)> {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([
            "--emit-types",
            "--edition=2024",
            "--cap-lints=allow",
            "--source-core",
            "minicore@body_projection_normalization",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "body_projection_normalization",
        ])
        .arg(path)
        .output()?;
    Ok((
        output.status.success(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

/// Every accepted case types and lowers. Each rejection reports its message at
/// the last occurrence of its site.
#[test]
fn body_projections_normalize_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let mut columns = line.split('\t');
        let path = columns.next().ok_or("normalization case path")?;
        let expected = columns.next().ok_or("normalization case outcome")?;
        let (types, stderr) = typed(&root.join(path))?;
        let source = crate::resolve::fixture(path)?;
        let matches = if expected == "0" {
            let lowered = program(&source).map(|_| ());
            if let Err(error) = &lowered {
                failures.push(format!("{path}: lowering {error}"));
            }
            types && lowered.is_ok()
        } else {
            let site = columns.next().ok_or("normalization rejection site")?;
            let offset = source.rfind(site).ok_or("normalization site text")?;
            !types && stderr.starts_with(&format!("ERROR: file 0: {offset}: {expected}\n"))
        };
        if !matches {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
        count += 1;
    }
    assert_eq!(count, 17);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` copies of `masks-one-let-transmute`'s method, each annotating a
/// local `T::Unsigned` that its signature does not name.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    text.push_str("impl<T,const N:usize> Mask<T,N> where T:MaskElement{\n");
    for index in 0..functions {
        writeln!(
            text,
            "    pub fn least{index}(self)->T{{let r:T::Unsigned=convert(self.to_simd());unsafe{{transmute_copy(&r)}}}}"
        )?;
    }
    text.push_str("}\n");
    Ok(text)
}

/// Typing work stays linear in the annotated projections, and every width lowers.
#[test]
fn body_projection_work_tracks_annotations() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/masks-one-let-transmute.rs"))?;
    let prefix = &source[..source
        .find("impl<T,const N:usize> Mask<T,N> where T:MaskElement{\n    pub fn least")
        .ok_or("one-let method")?];
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "body_projection_normalization")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("body projections functions={functions} work={work:?}");
        if let Some(before) = previous {
            for (old, actual) in std::iter::zip(before, work) {
                assert!(
                    actual <= 4 * old + 512,
                    "functions={functions}: {before:?}->{work:?}"
                );
            }
        }
        previous = Some(work);
        program(&text)?;
    }
    Ok(())
}
