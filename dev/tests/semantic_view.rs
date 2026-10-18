//! The PA19 inspection view reports selection while preserving later proof boundaries.
#[path = "harness/native_references.rs"]
mod native;
#[path = "semantic_view/relationships.rs"]
mod relationships;
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;
use rustgm::sema::semantic_view;
use std::io::{self, Write};
use std::process::{Command, Output, Stdio};

const SOURCE: &str = "trait A<U=u8>{fn get(&self,x:U)->U;fn default(&self,x:U)->U{Self::get(self,x)}}impl A for u8{fn get(&self,x:u8)->u8{x+*self}}impl A<u16>for u8{fn get(&self,x:u16)->u16{x}}struct S(u8);impl S{fn f(&self)->u8{self.0}}fn generic<T:A<u8>>(v:&T)->u8{v.get(2)}fn main(){let a=S(7);let _=S::f(&a);let b=7u8;let _=u8::get(&b,3u8);let _=<u8 as A<u16>>::get(&b,4u16);let _=generic(&b);let _=b.get(5u8);}";

fn cli(mode: &str, source: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .args([mode, "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing child input")?
        .write_all(source.as_bytes())?;
    Ok(child.wait_with_output()?)
}

#[test]
fn selected_items_have_complete_stable_records() -> Result<(), Box<dyn std::error::Error>> {
    rustc::qualify("semantic-view-selections", SOURCE, true)?;
    let (report, facts) = semantic::analyze(SOURCE)?;
    let mut bytes = Vec::new();
    semantic_view::write(&report, &facts, &mut bytes)?;
    let text = std::str::from_utf8(&bytes)?;
    assert!(text.starts_with("semantic-crate 1\n"));
    let rows: Vec<_> = text.lines().collect();
    for (prefix, count) in [
        ("associated-owner ", 4),
        ("associated-member ", 5),
        ("impl ", 2),
        ("impl-method ", 4),
        ("method-call ", 2),
        ("inherent-path ", 1),
        ("trait-path ", 3),
    ] {
        assert_eq!(
            rows.iter().filter(|r| r.starts_with(prefix)).count(),
            count,
            "{prefix}: {text}"
        );
    }
    assert_eq!(
        rows.iter()
            .filter(|r| r.starts_with("impl-method ") && r.ends_with("default=true"))
            .count(),
        2
    );
    assert!(
        rows.iter()
            .any(|r| r.starts_with("method-call ") && r.ends_with("origin=bound"))
    );
    assert!(
        rows.iter()
            .any(|r| r.starts_with("method-call ") && r.ends_with("origin=lexical"))
    );
    assert!(
        rows.iter()
            .any(|r| r.starts_with("trait-path ")
                && r.contains("qualifier=explicit origin=ordinary"))
    );
    assert!(
        rows.iter()
            .any(|r| r.starts_with("trait-path ") && r.contains("qualifier=implicit origin=bound"))
    );
    assert!(
        rows.iter()
            .any(|r| r.starts_with("implementation-use ") && r.contains(" arguments=["))
    );
    let mut count = Vec::new();
    semantic_view::count(&facts, &mut count)?;
    assert!(bytes.ends_with(&[b"eof ".as_slice(), &count].concat()));
    let cli_count = cli("--count-semantics", SOURCE)?;
    assert!(cli_count.status.success());
    assert!(cli_count.stderr.is_empty());
    assert_eq!(cli_count.stdout, count);
    let first = cli("--emit-semantics", SOURCE)?;
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, bytes);
    for _ in 0..6 {
        let repeated = cli("--emit-semantics", SOURCE)?;
        assert!(repeated.status.success());
        assert!(repeated.stderr.is_empty());
        assert_eq!(first.stdout, repeated.stdout);
    }
    Ok(())
}

#[test]
fn inspection_does_not_claim_region_and_loan_proof() -> Result<(), Box<dyn std::error::Error>> {
    let rejected = "trait A{fn f(&self)->&u8;}struct S(u8);impl A for S{fn f(&self)->&u8{&self.0}}fn g()->&'static u8{let s=S(7);s.f()}fn main(){}";
    rustc::qualify("semantic-view-later-region-rejection", rejected, false)?;
    let early = cli("--emit-semantics", rejected)?;
    assert!(
        early.status.success(),
        "{}",
        String::from_utf8_lossy(&early.stderr)
    );
    assert!(early.stderr.is_empty());
    assert!(std::str::from_utf8(&early.stdout)?.contains(" pending\n"));
    let later = cli("--emit-lowir", rejected)?;
    assert_eq!(later.status.code(), Some(1));
    assert!(later.stdout.is_empty());
    let ambiguous = "trait A{fn f(self);}trait B{fn f(self);}impl A for u8{fn f(self){}}impl B for u8{fn f(self){}}fn main(){3u8.f();}";
    rustc::qualify("semantic-view-name-rejection", ambiguous, false)?;
    for mode in ["--emit-semantics", "--count-semantics"] {
        let output = cli(mode, ambiguous)?;
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
    Ok(())
}

#[test]
fn inherited_drop_and_generic_arguments_remain_inspectable()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "trait A<'a,const N:usize>{fn f(&self,x:&'a [u8;N]);}struct S;impl Drop for S{fn drop(&mut self){}}impl<'a>A<'a,2>for S{fn f(&self,x:&'a [u8;2]){}}fn main(){let s=S;s.f(&[7,9]);}";
    rustc::qualify("semantic-view-drop-and-const", source, true)?;
    let output = cli("--emit-semantics", source)?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = std::str::from_utf8(&output.stdout)?;
    assert!(text.contains("const-parameter:"));
    assert!(text.contains("region:bound:"));
    assert!(text.contains("kind=trait-impl"));
    Ok(())
}

struct Fails;
impl Write for Fails {
    fn write(&mut self, _: &[u8]) -> io::Result<usize> {
        Err(io::Error::other("injected output failure"))
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
#[test]
fn output_failure_is_not_silently_accepted() -> Result<(), Box<dyn std::error::Error>> {
    let (report, facts) = semantic::analyze("fn main(){}")?;
    assert!(semantic_view::write(&report, &facts, &mut Fails).is_err());
    assert!(semantic_view::count(&facts, &mut Fails).is_err());
    Ok(())
}
