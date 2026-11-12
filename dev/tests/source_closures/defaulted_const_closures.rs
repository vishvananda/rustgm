//! A const closure in a const trait's default body meets a `[const] FnMut`
//! bound through the trait's `Self: [const] Trait` premise, as in core's
//! `Iterator::count`.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/default-const-closures";
const REJECTED: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

fn cases() -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("default const closure outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid default const closure outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), 10);
    Ok(cases)
}

#[test]
fn const_closures_in_const_trait_defaults_meet_const_callable_bounds_like_rustc() -> Result {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("repository root")?;
    let mut failures = Vec::new();
    for (path, accepted) in cases()? {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args([
                "--emit-types",
                "--edition=2024",
                "--cap-lints=allow",
                "--source-core",
                "minicore@default_closures",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "default_closures",
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

const UNSTABLE: &str = "#[rustc_const_unstable(feature=\"const_probe\",issue=\"none\")]";

#[derive(Clone, Copy, Debug)]
enum Form {
    /// A const closure, through the defaulted owner's contract.
    Closure,
    /// The same bound met by a const fn item, as a declaration-cost control.
    Item,
}

/// `uses` const traits whose default method passes a const closure or a const
/// fn item to a `[const] FnMut` bound, then unrelated const traits and const
/// fns that define and call their own const closures. Each trait names its own
/// methods, so method lookup on `Self` has one candidate.
fn width(
    prefix: &str,
    form: Form,
    uses: usize,
    noise: bool,
) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..uses {
        let argument = match form {
            Form::Closure => "const |accum,_elem|accum+1".to_owned(),
            Form::Item => {
                write!(
                    text,
                    "{UNSTABLE}pub const fn s{index}(accum:usize,_elem:u8)->usize{{accum+1}}"
                )?;
                format!("s{index}")
            }
        };
        writeln!(
            text,
            "{UNSTABLE}pub const trait W{index}{{fn fold{index}<F>(self,f:F)->usize where Self:Sized+[const]Destruct,F:[const]FnMut(usize,u8)->usize+[const]Destruct;fn count{index}(self)->usize where Self:Sized+[const]Destruct{{self.fold{index}({argument})}}}}"
        )?;
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "{UNSTABLE}pub const trait N{index}{{fn n(&self)->usize;}}{UNSTABLE}pub const fn m{index}()->usize{{let c=const |x:usize|x;c(1)}}"
        )?;
    }
    Ok(text)
}

#[test]
fn default_closure_eligibility_tracks_closures_without_scanning_unrelated_items() -> Result {
    let prefix = crate::resolve::fixture(&format!("{PACKET}/count.rs"))?;
    for form in [Form::Closure, Form::Item] {
        let mut previous = None;
        let mut unrelated = None;
        for uses in [8, 32, 128] {
            let mut ordinary = [0; 3];
            for noise in [false, true] {
                let report = crate::resolve::source_provider(
                    &width(&prefix, form, uses, noise)?,
                    "default_closures",
                )?;
                let facts = rustgm::pipeline::analyze_source(&report)?.facts;
                let work = [
                    facts.body_work.inference_steps,
                    facts.body_work.canonical_steps,
                    facts.body_work.trait_selections,
                ];
                println!("default const {form:?} uses={uses} noise={noise} work={work:?}");
                if noise {
                    // The unrelated items check their own bodies once; these
                    // uses never select or prove them.
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
