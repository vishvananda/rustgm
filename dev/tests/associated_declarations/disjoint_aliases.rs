//! A trait candidate header cannot equal a goal's rigid alias with another head.
use super::{native, prepared, semantic};
use crate::resolve as source;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKETS: [&str; 2] = [
    "pa20/tests/spec/rigid-candidate-headers",
    "pa24/tests/spec/rigid-candidate-headers",
];
const SOURCE_CORE: &str = "pa31/tests/spec/rigid-candidate-headers";
const REJECTED: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
const HEADER: &str = "trait It{type Item;}
trait Pick<Rhs>{fn pick(&self)->u8;}
struct W<X>(X);
impl<A:Pick<B>,B> Pick<W<B>> for W<A>{fn pick(&self)->u8{self.0.pick()+1}}
trait Marker{}impl<X:Marker> Pick<X> for X{fn pick(&self)->u8{50}}
";

fn inputs(packets: &[&str], count: usize) -> Result<Vec<(String, String, bool)>> {
    let mut inputs = Vec::new();
    let mut rows = String::new();
    for packet in packets {
        rows.push_str(&source::fixture(&format!("{packet}/cases.tsv"))?);
    }
    for line in rows.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("rigid header outcome columns")?;
        let expected = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid rigid header outcome".into()),
        };
        inputs.push((path.to_owned(), source::fixture(path)?, expected));
    }
    assert_eq!(inputs.len(), count);
    Ok(inputs)
}

#[test]
fn rigid_aliases_refute_candidate_headers_with_other_heads() -> Result {
    let mut failures = Vec::new();
    for (path, text, accepted) in inputs(&PACKETS, 6)? {
        match (accepted, prepared::checked(&text)) {
            (true, Ok(_)) => {}
            (false, Err(error)) if error.contains(REJECTED) => {}
            (_, result) => failures.push(format!("{path}: {:?}", result.err())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Core's comparison family, including the `FnPtr` blanket, through the CLI.
#[test]
fn source_core_reference_equality_refutes_the_fn_pointer_blanket() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    for (path, _, accepted) in inputs(&[SOURCE_CORE], 7)? {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@rigid_headers",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "rigid_headers",
            ])
            .arg(root.join(&path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let expected = if accepted { Some(0) } else { Some(1) };
        if output.status.code() != expected || !(accepted || stderr.contains(REJECTED)) {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Unrelated declarations: marker implementations, or unrelated `Pick` owners.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Noise {
    None,
    Markers,
    Owners,
}

fn width(calls: usize, noise: Noise) -> std::result::Result<String, std::fmt::Error> {
    let mut text = HEADER.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "fn c{index}<I:It,T>(w:W<I::Item>)->u8 where I::Item:Pick<T>{{<W<I::Item> as Pick<W<T>>>::pick(&w)}}"
        )?;
    }
    for index in 0..128 {
        match noise {
            Noise::None => break,
            Noise::Markers => writeln!(text, "struct M{index};impl Marker for M{index}{{}}")?,
            Noise::Owners => writeln!(
                text,
                "struct N{index}(u8);impl Pick<u8> for N{index}{{fn pick(&self)->u8{{self.0}}}}"
            )?,
        }
    }
    text.push_str("fn main(){}\n");
    Ok(text)
}

#[test]
fn rigid_header_refutation_tracks_calls_without_scanning_unrelated_impls() -> Result {
    let mut previous = None;
    for calls in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [Noise::None, Noise::Markers, Noise::Owners] {
            let (_, facts) = semantic::analyze(&width(calls, noise)?)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("rigid header calls={calls} work={work:?}");
            match noise {
                Noise::None => {
                    if let Some(before) = previous {
                        for (old, actual) in std::iter::zip(before, work) {
                            assert!(
                                actual <= 4 * old + 512,
                                "calls={calls}: {before:?}->{work:?}"
                            );
                        }
                    }
                    previous = Some(work);
                    ordinary = work;
                }
                Noise::Markers => assert_eq!(ordinary, work, "calls={calls} unrelated markers"),
                // Unrelated owners check their own bodies, but these goals
                // never select or refute them.
                Noise::Owners => {
                    assert_eq!(ordinary[2], work[2], "calls={calls} unrelated owners");
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn rigid_candidate_headers_execute_selected_values() -> Result {
    let inputs: Vec<_> = inputs(&PACKETS, 6)?
        .into_iter()
        .filter(|(_, _, pass)| *pass)
        .collect();
    assert_eq!(inputs.len(), 4);
    let cases: Vec<_> = inputs
        .iter()
        .map(|(path, text, _)| {
            let name = path
                .rsplit('/')
                .next()
                .and_then(|name| name.strip_suffix(".rs"));
            (name.unwrap_or(path.as_str()), text.as_str(), false)
        })
        .collect();
    native::check_literal_cases("pa20-rigid-candidate-headers/native", &cases)
}
