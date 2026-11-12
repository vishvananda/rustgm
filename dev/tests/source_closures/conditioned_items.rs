//! A conditional const fn item, or a const trait member's item, meets a
//! `[const]` callable bound through its substituted const conditions, as in
//! core's `Iterator::last` and `Option::as_deref`.
use std::fmt::Write as _;
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;
const PACKET: &str = "pa31/tests/spec/conditional-const-items";
const REJECTED: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";

fn cases() -> Result<Vec<(String, bool)>> {
    let mut cases = Vec::new();
    for line in crate::resolve::fixture(&format!("{PACKET}/cases.tsv"))?.lines() {
        let (path, expected) = line
            .split_once('\t')
            .ok_or("conditional const item outcome columns")?;
        let accepted = match expected {
            "0" => true,
            "1" => false,
            _ => return Err("invalid conditional const item outcome".into()),
        };
        cases.push((path.to_owned(), accepted));
    }
    assert_eq!(cases.len(), 11);
    Ok(cases)
}

#[test]
fn conditional_const_items_meet_const_callable_bounds_like_rustc() -> Result {
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
                "minicore@conditioned_items",
                "--crate-name",
                "minicore",
                "--crate-disambiguator",
                "conditioned_items",
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
    /// `T::default` under a `[const] Default` premise.
    Member,
    /// A nested const fn whose own `[const] Default` bound the caller meets.
    Nested,
}

/// `uses` const fns passing a conditional item to `make`'s `[const] FnOnce`
/// bound, then unrelated types with their own `const impl Default`.
fn width(
    prefix: &str,
    form: Form,
    uses: usize,
    noise: bool,
) -> std::result::Result<String, std::fmt::Error> {
    let mut text = prefix.to_owned();
    for index in 0..uses {
        match form {
            Form::Member => writeln!(
                text,
                "{UNSTABLE}pub const fn d{index}<T:[const]Default>()->T{{make(T::default)}}"
            )?,
            Form::Nested => writeln!(
                text,
                "{UNSTABLE}pub const fn n{index}<T:[const]Default>()->T{{{UNSTABLE}const fn inner<V:[const]Default>()->V{{V::default()}}make(inner::<T>)}}"
            )?,
        }
    }
    for index in 0..if noise { 64 } else { 0 } {
        writeln!(
            text,
            "pub struct N{index};#[rustc_const_unstable(feature=\"const_default\",issue=\"none\")]const impl Default for N{index}{{fn default()->N{index}{{N{index}}}}}"
        )?;
    }
    Ok(text)
}

#[test]
fn conditional_item_proofs_track_uses_without_scanning_unrelated_impls() -> Result {
    let prefix = crate::resolve::fixture(&format!("{PACKET}/default-item.rs"))?;
    for form in [Form::Member, Form::Nested] {
        let mut previous = None;
        let mut unrelated = None;
        for uses in [8, 32, 128] {
            let mut ordinary = [0; 3];
            for noise in [false, true] {
                let report = crate::resolve::source_provider(
                    &width(&prefix, form, uses, noise)?,
                    "conditioned_items",
                )?;
                let facts = rustgm::pipeline::analyze_source(&report)?.facts;
                let work = [
                    facts.body_work.inference_steps,
                    facts.body_work.canonical_steps,
                    facts.body_work.trait_selections,
                ];
                println!("conditional items {form:?} uses={uses} noise={noise} work={work:?}");
                if noise {
                    // The unrelated impls check their own bodies once; the
                    // premise proofs never select them.
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
