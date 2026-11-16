//! Coherence separates alloc's `Box` implementations from core's blanket
//! implementations as rustc 1.98.0 does under `with_negative_coherence`.
//!
//! Every outcome was declared before either compiler ran. rustc 1.98.0 gave
//! each one with the sources as one crate, and again with the core part as
//! crate `core` and the rest as crate `alloc` built with `--extern`; `rustc:`
//! names its code. A downstream crate can implement a trait only for a type
//! it can make local, and `&`, `&mut` and `#[fundamental]` `Box` are the only
//! types it can see through: `Box<[T; N]>: From<Box<[T]>>` and
//! `Box<dyn Error>: Error` are knowably absent, `Box<dyn Error, A>` with an
//! open allocator is not. A negative `Iterator` impl, negated under the other
//! header's own clauses, separates `IntoIterator for &Box<[I], A>` from
//! `impl<I: Iterator> IntoIterator for I`.
use std::{
    path::PathBuf,
    process::{Command, Output},
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

const OVERLAP: &str = "overlapping trait implementation headers require coherence proof";

/// A mini core with alloc's `Box`, `Global` and iterators in the same crate.
const CORE: &str = r#"#![feature(no_core,lang_items,negative_impls,rustc_attrs,auto_traits,with_negative_coherence,fundamental)]
#![no_core]
#![allow(internal_features,dead_code)]
#[lang="pointee_sized"]#[fundamental]#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[rustc_coinductive]pub trait PointeeSized{}
#[lang="meta_sized"]#[fundamental]#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[rustc_coinductive]pub trait MetaSized:PointeeSized{}
#[lang="sized"]#[fundamental]#[rustc_specialization_trait]#[rustc_deny_explicit_impl]#[rustc_coinductive]#[rustc_dyn_incompatible_trait]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
pub trait Iterator{type Item;}
impl<T> !Iterator for [T]{}
impl<I:Iterator+?Sized> Iterator for &mut I{type Item=I::Item;}
pub trait IntoIterator{type Item;type IntoIter:Iterator<Item=Self::Item>;}
impl<I:Iterator> IntoIterator for I{type Item=I::Item;type IntoIter=I;}
pub trait Allocator{}
pub struct Global;
impl Allocator for Global{}
pub trait Extra{}
#[fundamental]pub struct Box<T:?Sized,A:Allocator=Global>(*const T,A);
impl<I:Iterator+?Sized,A:Allocator> Iterator for Box<I,A>{type Item=I::Item;}
pub struct Iter<'a,T>(&'a T);
impl<'a,T> Iterator for Iter<'a,T>{type Item=&'a T;}
pub struct IntoIter<T,A:Allocator>(*const T,A);
impl<T,A:Allocator> Iterator for IntoIter<T,A>{type Item=T;}
"#;

/// Core's conversion traits and blankets, and alloc's other `Box` sources.
const CONVERT: &str = "pub trait From<T>:Sized{}
pub trait Into<T>:Sized{}
impl<T,U:From<T>> Into<U> for T{}
impl<T> From<T> for T{}
pub trait TryFrom<T>:Sized{}
impl<T,U:Into<T>> TryFrom<U> for T{}
pub struct Vec<T,A:Allocator=Global>(*const T,A);
impl<T> From<T> for Box<T>{}
impl<T,const N:usize> From<[T;N]> for Box<[T]>{}
impl<T,A:Allocator> From<Vec<T,A>> for Box<[T],A>{}
pub trait Error{}
";

/// One declared case: its name, its source after the prelude, and for a
/// rejection the implementation header the overlap is reported at.
type Case = (&'static str, &'static str, Option<&'static str>);

const REF: &str = "impl<'a,I,A:Allocator> IntoIterator for &'a Box<[I],A>";

/// Items 13 and 14 of the alloc list.
const ITERATORS: [Case; 10] = [
    // rustc: accepts.
    (
        "ref-negative",
        "impl<'a,I,A:Allocator> !Iterator for &'a Box<[I],A>{}impl<'a,I,A:Allocator> IntoIterator for &'a Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        None,
    ),
    (
        "mut-negative",
        "impl<I,A:Allocator> !Iterator for Box<[I],A>{}impl<'a,I,A:Allocator> !Iterator for &'a mut Box<[I],A>{}impl<'a,I,A:Allocator> IntoIterator for &'a mut Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        None,
    ),
    (
        "owned-negative",
        "impl<I,A:Allocator> !Iterator for Box<[I],A>{}impl<I,A:Allocator> IntoIterator for Box<[I],A>{type Item=I;type IntoIter=IntoIter<I,A>;}",
        None,
    ),
    // The negative impl's `A: Extra` is the other header's own premise.
    (
        "premise-pays-condition",
        "impl<'a,I,A:Allocator+Extra> !Iterator for &'a Box<[I],A>{}impl<'a,I,A:Allocator+Extra> IntoIterator for &'a Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        None,
    ),
    // No downstream crate can name `&Box<[I], Global>` for itself.
    (
        "closed-global",
        "impl<'a,I> IntoIterator for &'a Box<[I],Global>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        None,
    ),
    // rustc: E0119, a downstream allocator may make `&Box<[I], A>` an iterator.
    (
        "no-negative",
        "impl<'a,I,A:Allocator> IntoIterator for &'a Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        Some(REF),
    ),
    // rustc: E0119, the negative impl covers only the global allocator.
    (
        "narrow-negative",
        "impl<'a,I> !Iterator for &'a Box<[I],Global>{}impl<'a,I,A:Allocator> IntoIterator for &'a Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        Some(REF),
    ),
    // rustc: E0119, nothing proves the negative impl's `A: Extra`.
    (
        "uncovered-condition",
        "impl<'a,I,A:Allocator+Extra> !Iterator for &'a Box<[I],A>{}impl<'a,I,A:Allocator> IntoIterator for &'a Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        Some(REF),
    ),
    // rustc: E0119, the negative impls name the other references.
    (
        "other-reference",
        "impl<I,A:Allocator> !Iterator for Box<[I],A>{}impl<'a,I,A:Allocator> !Iterator for &'a mut Box<[I],A>{}impl<'a,I,A:Allocator> IntoIterator for &'a Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        Some(REF),
    ),
    // rustc: E0751, `impl Iterator for &mut I` holds unless `Box<[I], A>`
    // is known not to be an iterator.
    (
        "mut-without-owned",
        "impl<'a,I,A:Allocator> !Iterator for &'a mut Box<[I],A>{}impl<'a,I,A:Allocator> IntoIterator for &'a mut Box<[I],A>{type Item=&'a I;type IntoIter=Iter<'a,I>;}",
        Some("impl<'a,I,A:Allocator> !Iterator for &'a mut Box<[I],A>"),
    ),
];

/// Items 15 and 16 of the alloc list.
const CONVERSIONS: [Case; 9] = [
    // rustc: accepts; `Box<[T]>: Into<Box<[T; N]>>` is knowably absent.
    (
        "tryfrom-slice",
        "impl<T,const N:usize> TryFrom<Box<[T]>> for Box<[T;N]>{}",
        None,
    ),
    (
        "tryfrom-vec",
        "impl<T,const N:usize> TryFrom<Vec<T>> for Box<[T;N]>{}",
        None,
    ),
    // rustc: accepts; `Box<dyn Error>: Error` is knowably absent, and
    // `Error for Box<E>` needs a Sized `E`.
    (
        "from-error",
        "impl<E:Error> Error for Box<E>{}impl<'a,E:Error+'a> From<E> for Box<dyn Error+'a>{}",
        None,
    ),
    (
        "from-error-none",
        "impl<'a,E:Error+'a> From<E> for Box<dyn Error+'a>{}",
        None,
    ),
    // rustc: E0119, a downstream allocator may make the slice box convertible.
    (
        "tryfrom-open-allocator",
        "impl<T,A:Allocator,const N:usize> TryFrom<Box<[T],A>> for Box<[T;N],A>{}",
        Some("impl<T,A:Allocator,const N:usize> TryFrom"),
    ),
    // rustc: E0119, `[T; N]: Into<Box<[T]>>` holds.
    (
        "tryfrom-held",
        "impl<T,const N:usize> TryFrom<[T;N]> for Box<[T]>{}",
        Some("impl<T,const N:usize> TryFrom"),
    ),
    // rustc: E0119, `T: Into<Box<T>>` holds.
    (
        "tryfrom-genuine",
        "impl<T> TryFrom<T> for Box<T>{}",
        Some("impl<T> TryFrom"),
    ),
    // rustc: E0119, `Error for Box<E: ?Sized>` covers `Box<dyn Error>`.
    (
        "from-error-unsized",
        "impl<E:Error+?Sized> Error for Box<E>{}impl<'a,E:Error+'a> From<E> for Box<dyn Error+'a>{}",
        Some("impl<'a,E:Error+'a> From<E>"),
    ),
    // rustc: E0119, a downstream allocator may implement `Error` for its box.
    (
        "from-error-open",
        "impl<E:Error> Error for Box<E>{}impl<'a,E:Error+'a,A:Allocator> From<E> for Box<dyn Error+'a,A>{}",
        Some("impl<'a,E:Error+'a,A:Allocator> From<E>"),
    ),
];

fn folder() -> Result<PathBuf> {
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/fundamental-coherence")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}

/// How `output` differs from accepting the `file`th source of its session or
/// rejecting it at the last `rejected` header with the overlap error.
fn disagreement(
    output: &Output,
    (file, source): (usize, &str),
    rejected: Option<&str>,
) -> Result<Option<String>> {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let agrees = match rejected {
        None => output.status.success(),
        Some(construct) => {
            let offset = source
                .rfind(construct)
                .ok_or_else(|| format!("the source has no `{construct}`"))?;
            output.status.code() == Some(1)
                && stderr.contains(&format!("ERROR: file {file}: {offset}: {OVERLAP}"))
        }
    };
    Ok((!agrees).then(|| format!("{:?} {stderr}", output.status)))
}

/// Compile each case after `prelude` as one source-core crate; return every
/// disagreement with its declared outcome.
fn disagreements(label: &str, prelude: &str, cases: &[Case]) -> Result<Vec<String>> {
    let folder = folder()?;
    let mut failures = Vec::new();
    for &(name, suffix, rejected) in cases {
        let source = format!("{prelude}{suffix}\n");
        let path = folder.join(format!("{label}-{name}.rs"));
        std::fs::write(&path, &source)?;
        let output = Command::new(env!("CARGO_BIN_EXE_rustgm"))
            .args(["--emit-types", "--edition=2024", "--cap-lints=allow"])
            .args(["--source-core", "minicore@fundamental_coherence"])
            .args(["--crate-name", "minicore"])
            .args(["--crate-disambiguator", "fundamental_coherence"])
            .arg(&path)
            .output()?;
        if let Some(failure) = disagreement(&output, (0, &source), rejected)? {
            failures.push(format!("{label}/{name}: {failure}"));
        }
    }
    Ok(failures)
}

#[test]
fn negative_iterator_impls_separate_box_slice_iteration_from_the_blanket() -> Result {
    let failures = disagreements("plain", CORE, &ITERATORS)?;
    assert!(failures.is_empty(), "{failures:#?}");
    Ok(())
}

#[test]
fn a_fundamental_box_with_closed_arguments_has_knowable_conversions() -> Result {
    let prelude = format!("{CORE}{CONVERT}");
    let failures = disagreements("plain", &prelude, &CONVERSIONS)?;
    assert!(failures.is_empty(), "{failures:#?}");
    Ok(())
}
