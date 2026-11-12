//! A caller premise that selects an inferred trait input, such as the `Rhs`
//! of `X::Item::e(a, b)`, binds it to the premise's own projection. That
//! projection reaches its normalizer before an argument reads its head, as
//! core's `Pin` comparison forwarding needs.
use super::{prepared, rustc};
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ORDINARY: &str = "pa20/tests/spec/premise-inputs";
const PACKET: &str = "pa31/tests/spec/premise-inputs";
const ORDINARY_REJECTED: &str = "body types require a provable associated-result equality";
const SOURCE_REJECTED: &str = "no applicable trait method for this prefix";

fn cases(folder: &str, count: usize) -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{folder}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("premise input outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid premise input outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), count);
    Ok(cases)
}

#[test]
fn premise_selected_inputs_lower_like_rustc() -> Result {
    let mut failures = Vec::new();
    for (path, accepted) in cases(ORDINARY, 7)? {
        let source = crate::resolve::fixture(&path)?;
        let name = path.rsplit('/').next().ok_or("fixture name")?;
        rustc::qualify(&format!("premise-input-{name}"), &source, accepted)?;
        match (accepted, prepared::checked(&source)) {
            (true, Ok(_)) => {}
            (false, Err(error)) if error.contains(ORDINARY_REJECTED) => {}
            (_, result) => failures.push(format!("{path}: {:?}", result.err())),
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn source_premise_inputs_meet_deref_coercions_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    for (path, accepted) in cases(PACKET, 9)? {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@premise_inputs",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "premise_inputs",
            ])
            .arg(root.join(&path))
            .output()?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        let expected = if accepted { Some(0) } else { Some(1) };
        if output.status.code() != expected || !(accepted || stderr.contains(SOURCE_REJECTED)) {
            failures.push(format!("{path}: {:?} {stderr}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

const PREFIX: &str = "trait It{type Item;fn get(&self)->&Self::Item;}\
trait E<R:?Sized>{fn e(&self,o:&R)->bool;}\n";

/// `uses` functions whose premise selects the `Rhs` of a path call over a
/// coerced `&&Y::Item`, then unrelated types with their own `It` impls.
fn width(uses: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = PREFIX.to_owned();
    for index in 0..uses {
        writeln!(
            text,
            "fn p{index}<X:It,Y:It>(a:&X,b:&Y)->bool where X::Item:E<Y::Item>{{X::Item::e(a.get(),&b.get())}}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "struct N{index}(u8);impl It for N{index}{{type Item=u8;fn get(&self)->&u8{{&self.0}}}}"
        )?;
    }
    Ok(text)
}

#[test]
fn premise_input_normalization_tracks_uses_without_scanning_unrelated_impls() -> Result {
    let mut previous = None;
    let mut unrelated = None;
    for uses in [8, 32, 128] {
        let mut ordinary = [0; 3];
        for noise in [false, true] {
            let (_, facts) = crate::semantic::analyze(&width(uses, noise)?)?;
            let work = [
                facts.body_work.inference_steps,
                facts.body_work.canonical_steps,
                facts.body_work.trait_selections,
            ];
            println!("premise inputs uses={uses} noise={noise} work={work:?}");
            if noise {
                // The unrelated impls check their own bodies once; the
                // premise-selected goals never select them.
                let added: Vec<_> = std::iter::zip(ordinary, work)
                    .map(|(old, actual)| actual.checked_sub(old))
                    .collect();
                assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
            } else {
                if let Some(before) = previous {
                    for (old, actual) in std::iter::zip(before, work) {
                        assert!(actual <= 4 * old + 512, "uses={uses}: {before:?}->{work:?}");
                    }
                }
                previous = Some(work);
                ordinary = work;
            }
        }
    }
    Ok(())
}
