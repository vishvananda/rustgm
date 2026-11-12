//! Projected inherent implementation headers normalize before receiver selection.
use super::{native, prepared, semantic};
use crate::resolve as source;
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa20/tests/spec/projected-inherent-headers";
const HEADER: &str = "trait Try{type Output;fn from_output(o:Self::Output)->Self;}
enum Flow<B,C>{Continue(C),Break(B)}
impl<R:Try> Flow<R,R::Output>{fn into_try(self)->R{match self{Flow::Continue(v)=>R::from_output(v),Flow::Break(v)=>v}}}
";

fn inputs() -> Result<Vec<(String, String, bool)>> {
    let mut inputs = Vec::new();
    for line in source::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line.split_once('\t').ok_or("header outcome columns")?;
        let name = path
            .rsplit('/')
            .next()
            .and_then(|name| name.strip_suffix(".rs"))
            .ok_or("header case name")?;
        let expected = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid header outcome".into()),
        };
        inputs.push((name.to_owned(), source::fixture(path)?, expected));
    }
    assert_eq!(inputs.len(), 9);
    Ok(inputs)
}

#[test]
fn projected_inherent_headers_select_through_normalized_conditions() -> Result {
    let mut failures = Vec::new();
    for (name, text, accepted) in inputs()? {
        match (accepted, prepared::checked(&text)) {
            (true, Ok(_)) => {}
            (false, Err(error)) if error.contains("no applicable inherent method") => {}
            (_, result) => failures.push(format!("{name}: {:?}", result.err())),
        }
    }
    // Rust rejects an unrelated output (E0599). The outcome must stay a
    // rejection; its cause remains the pending candidate-refutation contract.
    let unbound =
        format!("{HEADER}fn code<A,R:Try>(c:Flow<R,A>)->R{{c.into_try()}}\nfn main(){{}}\n");
    if prepared::checked(&unbound).is_ok() {
        failures.push("unbound output accepted".to_owned());
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

/// Unrelated declarations: plain types, or inherent impls with their own methods.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Noise {
    None,
    Types,
    Methods,
}

fn width(calls: usize, noise: Noise) -> std::result::Result<String, std::fmt::Error> {
    let mut text = HEADER.to_owned();
    for index in 0..calls {
        writeln!(
            text,
            "fn c{index}<A,R:Try<Output=A>>(c:Flow<R,A>)->R{{c.into_try()}}"
        )?;
    }
    for index in 0..128 {
        match noise {
            Noise::None => break,
            Noise::Types => writeln!(text, "struct Unrelated{index};")?,
            Noise::Methods => writeln!(
                text,
                "struct N{index}(u8);impl N{index}{{fn into_try(&self)->u8{{self.0}}}}"
            )?,
        }
    }
    text.push_str("fn main(){}\n");
    Ok(text)
}

#[test]
fn projected_header_work_tracks_calls_without_scanning_unrelated_types() -> Result {
    let mut previous = None;
    for calls in [8, 32, 128] {
        let mut ordinary = [0; 4];
        for noise in [Noise::None, Noise::Types, Noise::Methods] {
            let (_, facts) = semantic::analyze(&width(calls, noise)?)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.methods.selection_steps,
                facts.methods.selection_probes,
            ];
            println!("projected header calls={calls} work={work:?}");
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
                Noise::Types => assert_eq!(ordinary, work, "calls={calls} unrelated types"),
                // Same-named unrelated methods check their own bodies, but
                // selection must not probe their owners for these receivers.
                Noise::Methods => {
                    assert_eq!(ordinary[2..], work[2..], "calls={calls} unrelated methods");
                }
            }
        }
    }
    Ok(())
}

#[test]
#[ignore = "requires the qualified CPPGM backend"]
fn projected_inherent_headers_execute_selected_values() -> Result {
    let inputs: Vec<_> = inputs()?.into_iter().filter(|(_, _, pass)| *pass).collect();
    assert_eq!(inputs.len(), 7);
    let cases: Vec<_> = inputs
        .iter()
        .map(|(name, text, _)| (name.as_str(), text.as_str(), false))
        .collect();
    native::check_literal_cases("pa20-projected-inherent-headers/native", &cases)
}
