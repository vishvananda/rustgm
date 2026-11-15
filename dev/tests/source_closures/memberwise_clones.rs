//! A source core has no `Clone` implementation for tuples, closures, function
//! items or function pointers: the language provides it. rustc builds a shim
//! that clones each element or capture in order and copies a function.
use super::implemented_copies::run;
use std::{
    path::{Path, PathBuf},
    process::Command,
};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Every case replaces this fixture's last two lines.
const PREFIX: &str = "pa31/tests/spec/source-copy-repeats/const-length.rs";
/// `note` reports each user `clone`; `Mark` is Copy with an observable one.
const SHARED: &str = "unsafe extern \"C\"{fn note(v:u8);}\n\
impl<T:PointeeSized> Clone for &T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for &T{}\n\
pub struct Step(u8);impl Clone for Step{fn clone(&self)->Self{unsafe{note(self.0)};Step(self.0)}}\n\
pub struct Mark(u8);impl Clone for Mark{fn clone(&self)->Self{unsafe{note(self.0)};Mark(self.0)}}impl Copy for Mark{}\n\
pub struct Owned(u8);\n\
fn seven()->u8{7}\n\
fn dup<T:Clone>(t:&T)->T{t.clone()}\n\
fn main(){let _=entry();}\n";
const ENTRY: &str = "#[unsafe(no_mangle)]pub extern \"C\" fn entry()->u8";
const OBSERVER: &str = "#include <stdio.h>\nvoid note(unsigned char v){printf(\"%u \",(unsigned)v);}\nextern unsigned char entry(void);\nint main(void){printf(\"= %u\\n\",(unsigned)entry());return 0;}\n";
const WRAPPED: &str = "pub trait Tag{fn tag(&self)->u8;}\n\
pub struct Wrap<F>(F);impl<F:Clone> Clone for Wrap<F>{fn clone(&self)->Self{Wrap(self.0.clone())}}\n\
impl<F:Fn(u8)->u8> Tag for Wrap<F>{fn tag(&self)->u8{(self.0)(1)}}\n\
fn make(n:Step)->impl Tag+Clone{Wrap(move|x:u8|{let _u=x;let _p=&n;n.0})}\n";
const REFILL: &str = "fn refill<T:Clone>(d:&mut T,s:&T){d.clone_from(s)}\n";
/// `Loud` swaps its fields when cloned and reports its first field when
/// cloned and when dropped, so a leaked or doubly dropped member is visible.
const LOUD: &str = "#[lang=\"drop\"]#[rustc_const_unstable(feature=\"const_destruct\",issue=\"133214\")]pub const trait Drop{fn drop(&mut self);}\n\
pub struct Loud(u8,u8);impl Clone for Loud{fn clone(&self)->Self{unsafe{note(self.0)};Loud(self.1,self.0)}}\n\
impl Drop for Loud{fn drop(&mut self){unsafe{note(self.0)}}}\n";

/// Items, the body of `entry`, and the observer's output: each `note` in
/// call order, then the value `entry` returns.
const ACCEPTED: &[(&str, &str, &str, &str)] = &[
    (
        "generic-tuple",
        "",
        "let t=(Step(1),Step(2));let u=dup(&t);let _keep=t;u.1.0",
        "1 2 = 2\n",
    ),
    (
        "generic-closure",
        "",
        "let a=Step(3);let b=Step(4);let f=move|x:u8|{let _q=&b;let _p=&a;x};let g=dup(&f);let _k=f(1);g(9)",
        "4 3 = 9\n",
    ),
    (
        "generic-copy-tuple",
        "",
        "let t=(Mark(5),Mark(6));let u=dup(&t);let _keep=t;u.1.0",
        "5 6 = 6\n",
    ),
    (
        "generic-copy-closure",
        "",
        "let m=Mark(7);let f=move|x:u8|{let _p=&m;x};let g=dup(&f);let _k=f(1);g(9)",
        "7 = 9\n",
    ),
    ("generic-fn-item", "", "let g=dup(&seven);g()", "= 7\n"),
    (
        "generic-fn-pointer",
        "",
        "let p:fn()->u8=seven;let g=dup(&p);g()",
        "= 7\n",
    ),
    (
        "nested",
        "",
        "let s=Step(3);let t=((Step(1),Step(2)),move|x:u8|{let _p=&s;x},Step(4));let u=dup(&t);let _keep=t;(u.1)(8)",
        "1 2 3 4 = 8\n",
    ),
    (
        "reference-capture",
        "",
        "let s=Step(3);let f=|x:u8|{let _u=x;s.0};let g=dup(&f);let _k=f(1);g(1)",
        "= 3\n",
    ),
    (
        "wrapped-opaque",
        WRAPPED,
        "let a=make(Step(3));let b=a.clone();let _k=a.tag();b.tag()",
        "3 = 3\n",
    ),
    (
        "provided-clone-from",
        REFILL,
        "let mut t=(Step(1),Step(2));let s=(Step(8),Step(9));refill(&mut t,&s);t.1.0",
        "8 9 = 9\n",
    ),
    (
        "direct-tuple",
        "",
        "let t=(Step(1),Step(2));let u=t.clone();let _keep=t;u.1.0",
        "1 2 = 2\n",
    ),
    (
        "direct-closure",
        "",
        "let n=Step(3);let f=move|x:u8|{let _p=&n;x};let g=f.clone();let _k=f(1);g(9)",
        "3 = 9\n",
    ),
    ("direct-fn-item", "", "let g=seven.clone();g()", "= 7\n"),
    (
        "direct-fn-pointer",
        "",
        "let p:fn()->u8=seven;let g=p.clone();g()",
        "= 7\n",
    ),
    (
        "direct-copy-tuple",
        "",
        "let t=(Mark(5),Mark(6));let u=t.clone();u.0.0",
        "5 6 = 5\n",
    ),
    (
        "path-tuple",
        "",
        "let t=(Step(1),Step(2));let u=Clone::clone(&t);let _keep=t;u.0.0",
        "1 2 = 1\n",
    ),
    ("generic-unit", "", "let u=dup(&());let _:()=u;4", "= 4\n"),
    (
        "reference-member",
        "",
        "let n=4u8;let t=((Step(1),&n),Step(2));let u=dup(&t);*(u.0).1",
        "1 2 = 4\n",
    ),
    (
        "opaque-closure",
        "fn mk<T:Clone>(t:T)->impl Fn()->T+Clone{move||t.clone()}\n",
        "let f=mk(Step(3));let g=f.clone();let _k=f();g().0",
        "3 3 3 = 3\n",
    ),
    (
        "hr-fn-pointer",
        "fn first(x:&u8)->u8{*x}\n",
        "let p:fn(&u8)->u8=first;let g=dup(&p);g(&7)",
        "= 7\n",
    ),
    (
        "unsafe-fn-pointer",
        "",
        "let p:unsafe extern \"C\" fn(u8)=note;let g=dup(&p);unsafe{g(5)};1",
        "5 = 1\n",
    ),
    (
        "direct-clone-from",
        "",
        "let mut t=(Step(1),Step(2));let s=(Step(8),Step(9));t.clone_from(&s);t.0.0",
        "8 9 = 8\n",
    ),
    (
        "reference-to-tuple",
        "",
        "let t=(Step(1),Step(2));let r=(&t,Step(3));let u=dup(&r);(u.0).1.0",
        "3 = 2\n",
    ),
    (
        "twice-closure",
        "fn twice<T:Clone>(t:&T)->(T,T){(t.clone(),t.clone())}\n",
        "let a=Step(3);let f=move|x:u8|{let _p=&a;x};let (g,h)=twice(&f);h(g(4))",
        "3 3 = 4\n",
    ),
    (
        "opaque-member",
        WRAPPED,
        "let t=(make(Step(3)),Step(4));let u=dup(&t);(u.0).tag()",
        "3 4 = 3\n",
    ),
    (
        "empty-closure",
        "",
        "let f=|x:u8|x;let g=dup(&f);g(6)",
        "= 6\n",
    ),
    (
        "nested-closure",
        "",
        "let s=Step(3);let inner=move|x:u8|{let _p=&s;x};let outer=move|x:u8|inner(x);let g=dup(&outer);g(5)",
        "3 = 5\n",
    ),
    (
        "generic-fn-item-args",
        "fn id<T>(t:T)->T{t}\n",
        "let g=dup(&id::<u8>);g(9)",
        "= 9\n",
    ),
    (
        "parameter-tuple",
        "fn dup2<A:Clone,B:Clone>(t:&(A,B))->(A,B){t.clone()}\n",
        "let t=(Step(1),Step(2));let u=dup2(&t);u.1.0",
        "1 2 = 2\n",
    ),
    (
        "through-reference",
        "",
        "let t=(Step(1),Step(2));let r=&t;let u=r.clone();u.0.0",
        "1 2 = 1\n",
    ),
    (
        "fn-item-member",
        "",
        "let t=(seven,Step(2));let u=dup(&t);(u.0)()",
        "2 = 7\n",
    ),
    (
        "dropped-members",
        LOUD,
        "let t=(Loud(1,11),Loud(2,12));let u=dup(&t);u.1.1",
        "1 2 11 12 1 2 = 2\n",
    ),
    (
        "dropped-capture",
        LOUD,
        "let a=Loud(3,13);let f=move|x:u8|{let _p=&a;x};let g=dup(&f);let _k=f(1);g(9)",
        "3 13 3 = 9\n",
    ),
];
const UNPROVED: &str =
    "trait obligation requires an unambiguous matching assumption or implementation";
/// Items, the body of `entry`, rustc's error code and the typing rejection.
const REJECTED: &[(&str, &str, &str, &str, &str)] = &[
    (
        "reject-tuple-element",
        "",
        "let t=(Step(1),Owned(2));let u=dup(&t);0",
        "E0277",
        UNPROVED,
    ),
    (
        "reject-closure-capture",
        "",
        "let o=Owned(1);let f=move|x:u8|{let _p=&o;x};let g=dup(&f);0",
        "E0277",
        UNPROVED,
    ),
    (
        "reject-unique-capture",
        "",
        "let mut n=1u8;let f=||{n=2;};let g=dup(&f);0",
        "E0277",
        UNPROVED,
    ),
    (
        "reject-mutable-member",
        "",
        "let mut n=1u8;let t=(&mut n,Step(1));let u=dup(&t);0",
        "E0277",
        UNPROVED,
    ),
    (
        "reject-direct-tuple",
        "",
        "let t=(Step(1),Owned(2));let u=t.clone();0",
        "E0599",
        "no applicable method for this receiver",
    ),
    (
        "reject-parameter-tuple",
        "fn bad<A:Clone,B>(t:&(A,B))->(A,B){t.clone()}\n",
        "0",
        "E0308",
        "incompatible types",
    ),
];

fn evidence(kind: &str) -> Result<PathBuf> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/parallel/language-clones/{kind}/{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&folder)?;
    Ok(folder)
}

fn source(items: &str, body: &str) -> Result<String> {
    let fixture = crate::resolve::fixture(PREFIX)?;
    let lines: Vec<&str> = fixture.lines().collect();
    let kept = lines.len().checked_sub(2).ok_or("clone prefix lines")?;
    let mut text = lines[..kept].join("\n");
    text.push('\n');
    text.push_str(SHARED);
    text.push_str(items);
    text.push_str(ENTRY);
    text.push('{');
    text.push_str(body);
    text.push_str("}\n");
    Ok(text)
}

/// The independent oracle: pinned rustc compiles the same library to an object.
pub(super) fn rustc(path: &Path) -> Result<std::process::Output> {
    Ok(Command::new("rustc")
        .env("RUSTC_BOOTSTRAP", "1")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--crate-type=lib",
            "--crate-name",
            "minicore",
            "--emit=obj",
            "--cap-lints=allow",
            "-Cpanic=abort",
            "-Copt-level=0",
            "-o",
        ])
        .arg(path.with_extension("o"))
        .arg(path)
        .output()?)
}

#[test]
fn language_clones_type_and_lower_as_rustc_accepts() -> Result {
    let folder = evidence("source")?;
    let mut failures = Vec::new();
    for &(name, items, body, _) in ACCEPTED {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source(items, body)?)?;
        let oracle = rustc(&path)?;
        if !oracle.status.success() {
            failures.push(format!(
                "{name} rustc: {}",
                String::from_utf8_lossy(&oracle.stderr)
            ));
        }
        for mode in ["--emit-types", "--emit-lowir"] {
            let output = run(mode, &path.to_string_lossy())?;
            if !output.status.success() {
                failures.push(format!(
                    "{name} {mode}: {:?} {}",
                    output.status.code(),
                    String::from_utf8_lossy(&output.stderr)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

#[test]
fn language_clones_keep_rejecting_members_that_are_not_clone() -> Result {
    let folder = evidence("rejected")?;
    let mut failures = Vec::new();
    for &(name, items, body, code, reason) in REJECTED {
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, source(items, body)?)?;
        let oracle = rustc(&path)?;
        let stderr = String::from_utf8_lossy(&oracle.stderr);
        if oracle.status.code() != Some(1) || !stderr.contains(code) {
            failures.push(format!("{name} rustc: {:?} {stderr}", oracle.status.code()));
        }
        let output = run("--emit-types", &path.to_string_lossy())?;
        let stderr = String::from_utf8_lossy(&output.stderr);
        if output.status.code() != Some(1) || !stderr.contains(reason) {
            failures.push(format!("{name}: {:?} {stderr}", output.status.code()));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    Ok(())
}

pub(super) fn program(text: &str) -> Result<rustgm::lowir::program::Program> {
    let mut config =
        rustgm::expand::config::Config::in_edition(rustgm::support::edition::Edition::Rust2024);
    config.set_lint_cap(rustgm::support::diagnostics::LintCap::Allow);
    let report = crate::resolve::source_provider_config(text, "copy_captures", &config)?;
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

/// Link rustc's object with the observer and return what the program prints.
pub(super) fn independent(path: &Path, observer: &Path) -> Result<Vec<u8>> {
    let oracle = rustc(path)?;
    if !oracle.status.success() {
        return Err(String::from_utf8_lossy(&oracle.stderr).into_owned().into());
    }
    let binary = path.with_extension("rustc");
    let linked = Command::new("cc")
        .arg(observer)
        .arg(path.with_extension("o"))
        .arg("-o")
        .arg(&binary)
        .output()?;
    if !linked.status.success() {
        return Err(String::from_utf8_lossy(&linked.stderr).into_owned().into());
    }
    let output = Command::new(&binary).output()?;
    if !output.status.success() || !output.stderr.is_empty() {
        return Err(format!("independent program: {output:?}").into());
    }
    Ok(output.stdout)
}

/// rustc and the native backend print the same clone calls in the same order.
#[test]
#[ignore = "requires the locked CPPGM object backend and a C compiler"]
fn language_clones_run_each_member_clone_in_order() -> Result {
    let folder = evidence("native")?;
    let observer = folder.join("observer.c");
    std::fs::write(&observer, OBSERVER)?;
    for &(name, items, body, expected) in ACCEPTED {
        let text = source(items, body)?;
        let path = folder.join(format!("{name}.rs"));
        std::fs::write(&path, &text)?;
        let printed = independent(&path, &observer)?;
        assert_eq!(
            String::from_utf8_lossy(&printed),
            expected,
            "{name}: rustc differs from the declared output"
        );
        let program = program(&text).map_err(|error| format!("{name}: {error}"))?;
        crate::observed_program::execute(
            &folder.join(name),
            &program,
            &observer,
            expected.as_bytes(),
        )
        .map_err(|error| format!("{name}: {error}"))?;
    }
    Ok(())
}
