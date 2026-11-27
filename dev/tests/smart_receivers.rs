//! Actual source receiver chains require both dereference and legacy permission.
use crate::resolve as test_source;
#[path = "smart_receivers/pinned_calls.rs"]
mod pinned_calls;
use rustgm::{lower, pipeline};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn program(source: &str) -> TestResult<rustgm::lowir::program::Program> {
    let report = test_source::source_provider(source, "receiver_source")?;
    let pipeline::Analysis {
        mut facts,
        prepared,
    } = pipeline::analyze_source(&report)?;
    Ok(lower::instances::build_roots_prepared(
        &report,
        &mut facts,
        lower::Roots {
            entry: None,
            exports: &[],
        },
        prepared,
    )?)
}
#[test]
fn genuine_receiver_chains_preserve_complete_self_arguments_and_source_traits() -> TestResult {
    let mut count = 0;
    let mut failures = Vec::new();
    for line in test_source::fixture("pa44/tests/spec/pin-receivers/cases.tsv")?.lines() {
        let (path, status) = line.split_once('\t').ok_or("case columns")?;
        let result = program(&test_source::fixture(path)?);
        let failure = match (status, result) {
            ("0", Err(e)) => Some(format!("{path}: {e}")),
            ("1", Ok(_)) => Some(format!("accepted {path}")),
            ("1", Err(e))
                if e.to_string()
                    .contains("semantic contract is not implemented") =>
            {
                Some(format!("{path}: {e}"))
            }
            ("0", Ok(_)) | ("1", Err(_)) => None,
            _ => return Err("invalid prediction".into()),
        };
        if let Some(failure) = failure {
            failures.push(failure);
        }
        count += 1;
    }
    assert_eq!(count, 12);
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn proven_source_pin_receivers_execute_the_actual_methods() -> TestResult {
    let source = test_source::fixture("pa44/tests/spec/pin-receivers/native-receiver.rs")?;
    let program = program(&source)?;
    let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/parallel/deref-source-v1/receiver-native");
    std::fs::create_dir_all(&folder)?;
    std::fs::write(folder.join("source.rs"), source)?;
    rustgm::lowir::writer::write(
        &program,
        &mut std::fs::File::create(folder.join("source.lowir"))?,
    )?;
    Ok(())
}

/// A source core whose `Owner` and `Held` stand for alloc's `Box<T, A>`:
/// each dereferences to its target and is a legacy receiver.
const OWNERS: &str = "#![feature(no_core,lang_items,rustc_attrs)]#![no_core]#![rustc_coherence_is_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang=\"pointee_sized\"]pub trait PointeeSized{}
#[lang=\"meta_sized\"]pub trait MetaSized:PointeeSized{}
#[lang=\"sized\"]pub trait Sized:MetaSized{}
#[lang=\"legacy_receiver\"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
#[lang=\"clone\"]pub trait Clone:Sized{#[lang=\"clone_fn\"]fn clone(&self)->Self;}
#[lang=\"copy\"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->u8{*self}}impl Copy for u8{}
#[lang=\"deref\"]pub trait Deref:PointeeSized{#[lang=\"deref_target\"]type Target:?Sized;fn deref(&self)->&Self::Target;}
pub struct Owner<T:?Sized>(*const T);
impl<T:?Sized> LegacyReceiver for Owner<T>{}
impl<T:?Sized> Deref for Owner<T>{type Target=T;fn deref(&self)->&T{unsafe{&*self.0}}}
pub struct Held<T:?Sized,A>(*const A,*const T);
impl<T:?Sized,A> LegacyReceiver for Held<T,A>{}
impl<T:?Sized,A> Deref for Held<T,A>{type Target=T;fn deref(&self)->&T{unsafe{&*self.1}}}
";
/// Declared before either compiler ran; rustc 1.98.0 accepts each case
/// marked `true` and rejects the other (E0599).
const OWNED_RECEIVERS: [(&str, bool, &str); 5] = [
    (
        "slice-owner",
        true,
        "impl<T> [T]{pub fn take(self:Owner<Self>)->u8{7}}pub fn f(b:Owner<[u8]>)->u8{b.take()}",
    ),
    (
        "slice-held",
        true,
        "impl<T> [T]{pub fn take<A>(self:Held<Self,A>)->u8{7}}pub fn f<A>(b:Held<[u8],A>)->u8{b.take()}",
    ),
    (
        "struct-owner",
        true,
        "pub struct S;impl S{pub fn take(self:Owner<Self>)->u8{7}}pub fn f(b:Owner<S>)->u8{b.take()}",
    ),
    (
        "struct-held",
        true,
        "pub struct S;impl S{pub fn take<A>(self:Held<Self,A>)->u8{7}}pub fn f(b:Held<S,u8>)->u8{b.take()}",
    ),
    (
        "slice-reference",
        false,
        "impl<T> [T]{pub fn take(self:Owner<Self>)->u8{7}}pub fn f(b:&[u8])->u8{b.take()}",
    ),
];
/// A method of a structural Self, as alloc's `<[T]>::into_vec(self: Box<Self,
/// A>)`, is found through its receiver's `Deref` target, and a smart
/// receiver is matched with the method's own parameters fresh. A reference
/// to the target is no such receiver.
#[test]
fn smart_receivers_reach_structural_owners_and_name_their_own_parameters() -> TestResult {
    for (name, accepted, body) in OWNED_RECEIVERS {
        match (accepted, program(&format!("{OWNERS}{body}"))) {
            (true, Err(error)) => return Err(format!("{name}: {error}").into()),
            (false, Ok(_)) => return Err(format!("accepted {name}").into()),
            (false, Err(error)) => assert!(
                error
                    .to_string()
                    .contains("no applicable inherent method for this receiver"),
                "{name}: {error}"
            ),
            (true, Ok(_)) => {}
        }
    }
    Ok(())
}
