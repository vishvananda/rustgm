//! A function item whose generic inputs are inferred only through a callable
//! bound meets an abstract projection input such as `I::Item` by blocked
//! projection identity, as core's `Iterator::max` passes `Ord::cmp`.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const ORDINARY: &str = "pa24/tests/spec/abstract-callable-inputs";
const PACKET: &str = "pa31/tests/spec/abstract-callable-inputs";
const REJECTED: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

fn cases(folder: &str, count: usize) -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{folder}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("abstract callable input outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid abstract callable input outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), count);
    Ok(cases)
}

#[test]
fn ordinary_abstract_callable_inputs_lower_like_rustc() -> Result {
    let mut failures = Vec::new();
    for (path, accepted) in cases(ORDINARY, 9)? {
        let source = crate::resolve::fixture(&path)?;
        let name = path.rsplit('/').next().ok_or("fixture name")?;
        crate::qualify::qualify(&format!("abstract-callable-{name}"), &source, accepted)?;
        let result = crate::semantic::analyze(&source).and_then(|(mut report, mut facts)| {
            let entry =
                rustgm::sema::entry::select(&mut report, &facts).map_err(|e| e.to_string())?;
            rustgm::lower::instances::build(&report, &mut facts, entry)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if result.is_ok() != accepted
            || result
                .as_ref()
                .err()
                .is_some_and(|error| !error.contains(REJECTED))
        {
            failures.push(format!("{path}: {:?}", result.err()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn source_abstract_callable_inputs_meet_callable_bounds_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    for (path, accepted) in cases(PACKET, 11)? {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@abstract_inputs",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "abstract_inputs",
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

const PREFIX: &str = "trait Num{fn num(&self)->u16;}impl Num for u16{fn num(&self)->u16{*self}}\
fn by<T,F:FnMut(&T)->u16>(x:T,mut f:F)->u16{f(&x)}fn get<S:Num>(x:&S)->u16{x.num()}trait It{type Item;}\n";

#[derive(Clone, Copy, Debug)]
enum Form {
    /// The trait member path `Num::num`, as core passes `Ord::cmp`.
    Member,
    /// A generic free fn item.
    Generic,
}

/// `uses` functions passing a function item for an `I::Item` callable input,
/// then unrelated types with their own `Num` impls.
fn width(form: Form, uses: usize, noise: bool) -> std::result::Result<String, std::fmt::Error> {
    let mut text = PREFIX.to_owned();
    let item = match form {
        Form::Member => "Num::num",
        Form::Generic => "get",
    };
    for index in 0..uses {
        writeln!(
            text,
            "fn p{index}<I:It>(x:I::Item)->u16 where I::Item:Num{{by(x,{item})}}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "struct N{index};impl Num for N{index}{{fn num(&self)->u16{{{index}}}}}"
        )?;
    }
    Ok(text)
}

#[test]
fn abstract_input_items_track_uses_without_scanning_unrelated_impls() -> Result {
    for form in [Form::Member, Form::Generic] {
        let mut previous = None;
        let mut unrelated = None;
        for uses in [8, 32, 128] {
            let mut ordinary = [0; 3];
            for noise in [false, true] {
                let (_, facts) = crate::semantic::analyze(&width(form, uses, noise)?)?;
                let work = [
                    facts.body_work.inference_steps,
                    facts.body_work.canonical_steps,
                    facts.body_work.trait_selections,
                ];
                println!("abstract inputs {form:?} uses={uses} noise={noise} work={work:?}");
                if noise {
                    // The unrelated impls check their own bodies once; the
                    // function item proofs never select them.
                    let added: Vec<_> = std::iter::zip(ordinary, work)
                        .map(|(old, actual)| actual.checked_sub(old))
                        .collect();
                    assert_eq!(*unrelated.get_or_insert_with(|| added.clone()), added);
                } else {
                    if let Some(before) = previous {
                        for (old, actual) in std::iter::zip(before, work) {
                            assert!(
                                actual <= 4 * old + 512,
                                "{form:?} uses={uses}: {before:?}->{work:?}"
                            );
                        }
                    }
                    previous = Some(work);
                    ordinary = work;
                }
            }
        }
    }
    Ok(())
}
