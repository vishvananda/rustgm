//! A projection's symbolic result discards each compatible impl whose trait
//! condition has no candidate, as rustc's winnowing does. Core's
//! `(&[]).into_iter()` in `slice/iter/macros.rs`'s `Default` needs it: the
//! blanket `impl<I: Iterator> IntoIterator for I` matches `&[?E; 0]`'s header.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/blanket-candidate-winnowing";
const SUFFIX: &str = "impl<T> Default for Iter<'_, T> {
    fn default() -> Self {
        (& []).into_iter()
    }
}
";

fn program(source: &str) -> Result<rustgm::lowir::program::Program> {
    let report = crate::resolve::source_provider(source, "winnowed_projections")?;
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
            "minicore@winnowed_projections",
            "--crate-name",
            "minicore",
            "--crate-disambiguator",
            "winnowed_projections",
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
fn blanket_candidates_are_winnowed_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut count = 0;
    let mut failures = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let mut columns = line.split('\t');
        let path = columns.next().ok_or("winnowing case path")?;
        let expected = columns.next().ok_or("winnowing case outcome")?;
        let (types, stderr) = typed(&root.join(path))?;
        let source = crate::resolve::fixture(path)?;
        let matches = if expected == "0" {
            let lowered = program(&source).map(|_| ());
            if let Err(error) = &lowered {
                failures.push(format!("{path}: lowering {error}"));
            }
            types && lowered.is_ok()
        } else {
            let site = columns.next().ok_or("winnowing rejection site")?;
            let offset = source.rfind(site).ok_or("winnowing site text")?;
            !types && stderr.starts_with(&format!("ERROR: file 0: {offset}: {expected}\n"))
        };
        if !matches {
            failures.push(format!("{path}: types={types} {stderr}"));
        }
        count += 1;
    }
    assert_eq!(count, 11);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// `functions` functions, each returning `(&[]).into_iter()` as an `Iter`.
fn width(prefix: &str, functions: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..functions {
        writeln!(
            text,
            "pub fn empty{index}<'a, T>() -> Iter<'a, T> {{ (&[]).into_iter() }}"
        )?;
    }
    Ok(text)
}

/// Typing work stays linear in the winnowed projections, and every width lowers.
#[test]
fn winnowed_projection_work_tracks_uses() -> Result {
    let source = crate::resolve::fixture(&format!("{PACKET}/core-form.rs"))?;
    let prefix = source.strip_suffix(SUFFIX).ok_or("winnowing suffix")?;
    let mut previous: Option<[usize; 3]> = None;
    for functions in [8, 32, 128] {
        let text = width(prefix, functions)?;
        let report = crate::resolve::source_provider(&text, "winnowed_projections")?;
        let facts = rustgm::pipeline::analyze_source(&report)?.facts;
        let work = [
            facts.body_work.inference_steps,
            facts.body_work.canonical_steps,
            facts.body_work.trait_selections,
        ];
        println!("winnowed projections functions={functions} work={work:?}");
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

/// The integer types, for a family of unconditional `Shl` impls.
const INTEGERS: [&str; 12] = [
    "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
];

/// `uses` functions shaped like core's `_mm512_movm_epi16` argument, over the
/// first `impls` integer `Shl` impls, `i16 << i32` first.
fn family(impls: usize, uses: usize) -> std::result::Result<String, std::fmt::Error> {
    let mut text = String::from(
        "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]\n\
         #[lang=\"pointee_sized\"]pub trait PointeeSized{}\n\
         #[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}\n\
         #[lang=\"sized\"]pub trait Sized:MetaSized{}\n\
         #[lang=\"legacy_receiver\"]pub trait LegacyReceiver:PointeeSized{}\n\
         impl<T:PointeeSized> LegacyReceiver for &T{}\n\
         #[lang=\"clone\"]pub trait Clone:Sized{#[lang=\"clone_fn\"]fn clone(&self)->Self;}\n\
         #[lang=\"copy\"]pub trait Copy:Clone{}\n\
         #[lang=\"shl\"]pub trait Shl<Rhs=Self>{type Output;fn shl(self,rhs:Rhs)->Self::Output;}\n\
         #[lang=\"bitor\"]pub trait BitOr<Rhs=Self>{type Output;fn bitor(self,rhs:Rhs)->Self::Output;}\n",
    );
    for integer in INTEGERS {
        writeln!(
            text,
            "impl Clone for {integer}{{fn clone(&self)->Self{{*self}}}}impl Copy for {integer}{{}}\
             impl BitOr for {integer}{{type Output={integer};fn bitor(self,rhs:{integer})->{integer}{{loop{{}}}}}}"
        )?;
    }
    let pairs = INTEGERS
        .iter()
        .flat_map(|subject| INTEGERS.iter().map(move |rhs| (*subject, *rhs)));
    let first = std::iter::once(("i16", "i32"));
    for (subject, rhs) in first
        .chain(pairs.filter(|pair| *pair != ("i16", "i32")))
        .take(impls)
    {
        writeln!(
            text,
            "impl Shl<{rhs}> for {subject}{{type Output={subject};fn shl(self,rhs:{rhs})->{subject}{{loop{{}}}}}}"
        )?;
    }
    for index in 0..uses {
        writeln!(
            text,
            "pub fn mask{index}()->i16{{1<<15|1<<14|1<<13|1<<12|1<<11|1<<10|1<<9|1<<8}}"
        )?;
    }
    Ok(text)
}

fn family_work(impls: usize, uses: usize) -> Result<[usize; 3]> {
    let text = family(impls, uses)?;
    let report = crate::resolve::source_provider(&text, "winnowed_projections")?;
    let work = rustgm::pipeline::analyze_source(&report)?.facts.body_work;
    Ok([
        work.inference_steps,
        work.canonical_steps,
        work.trait_selections,
    ])
}

/// Winnowing discards only conditional candidates, so a second unconditional
/// compatible header ends the scan before the rest are instantiated. Canonical
/// work and selections per `?int << ?int` use stay flat as the integer `Shl`
/// family grows. Inference still grows, by about 23 steps per impl and
/// operator, against about 197 without the bound, where 128 impls cost 5.4
/// times 8.
#[test]
fn unconditional_family_scan_stops_at_the_second_header() -> Result {
    let mut base: Option<[usize; 3]> = None;
    for impls in [8, 32, 128] {
        let empty = family_work(impls, 0)?;
        let used = family_work(impls, 16)?;
        let per_uses: [usize; 3] =
            std::array::from_fn(|index| used[index].saturating_sub(empty[index]));
        println!("unconditional family impls={impls} empty={empty:?} uses={per_uses:?}");
        if let Some([inference, canonical, selections]) = base {
            let bounds = [3 * inference, canonical + 512, selections + 512];
            assert!(
                std::iter::zip(per_uses, bounds).all(|(actual, bound)| actual <= bound),
                "impls={impls}: {base:?}->{per_uses:?}"
            );
        }
        base.get_or_insert(per_uses);
    }
    Ok(())
}
