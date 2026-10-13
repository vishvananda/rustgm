use rustgm::{
    body,
    borrow::ownership,
    expand::config::Config,
    resolve::{
        self,
        load::{InputFile, Sources},
    },
    sema::{bodies, declarations},
    support::source::SourceFile,
    types::{
        Kind, RegionId, Store, TypeId,
        aggregate::Definitions,
        copy::{Copies, Properties},
    },
};
use std::{collections::HashMap, path::Path, process::Command};

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn check(source: &str) -> Result<String, String> {
    let report = resolve::expansion::analyze(
        InputFile {
            identity: "lib.rs".into(),
            path: "lib.rs".into(),
            source: SourceFile::new(source.as_bytes().to_vec())?,
        },
        &mut NoFiles,
        &Config::default(),
    )
    .and_then(resolve::analyze_expanded)
    .map_err(|e| e.to_string())?;
    let mut facts = declarations::analyze(&report)
        .and_then(|f| bodies::analyze(&report, f))
        .map_err(|e| e.to_string())?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let checked = ownership::check(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        &program,
    )
    .map_err(|e| e.message.to_string())?;
    let mut text = Vec::new();
    ownership::view::write(&checked, &mut text).map_err(|e| e.to_string())?;
    String::from_utf8(text).map_err(|e| e.to_string())
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "One declared acceptance matrix is checked by both frontends"
)]
fn moves_and_derive_obligations_agree_with_pinned_rustc() -> Result<(), Box<dyn std::error::Error>>
{
    let cases = [
        ("struct N; fn f(){let x=N;let _a=x;let _b=x;}", false),
        ("struct N; fn f(){let x=N;let _=x;let _b=x;}", true),
        ("struct N; fn f(){let x=N;let _a=x;x=N;}", false),
        ("struct N; fn f(){let mut x=N;let _a=x;x=N;let _b=x;}", true),
        (
            "struct N; fn f(){let mut x=(N,N);let _a=x.0;let _b=x.1;}",
            true,
        ),
        (
            "struct N; fn f(){let mut x=(N,N);let _a=x.0;let _b=x;}",
            false,
        ),
        (
            "struct N; fn f(){let mut x=(N,N);let _a=x.0;x.0=N;let _b=x;}",
            true,
        ),
        (
            "struct N; fn f(){let mut x=(N,N);let _a=x.0;let _b=x.1;x.0=N;x.1=N;let _c=x;}",
            true,
        ),
        ("struct N; fn f(){let mut x=(N,N);let _a=x;x.0=N;}", false),
        ("struct N; fn f(){let mut x:(N,N);x.0=N;}", false),
        ("struct N; fn f(){let x=(N,N);x.0=N;}", false),
        ("struct N; fn f(){let x:(N,N);let _=x.0;}", true),
        (
            "struct N; fn f(){let mut x=((N,N),N);let _a=x.0;x.0.0=N;}",
            false,
        ),
        (
            "struct N; fn f(){let mut x=((N,N),N);let _a=x.0.0;x.0.0=N;let _b=x;}",
            true,
        ),
        (
            "struct N; fn f(){let mut x=((N,N),N);let _a=x.0.0;let _b=x.0;}",
            false,
        ),
        (
            "struct N; fn f(){let x=(N,1u8);let _a=x.0;let _b=x.1;}",
            true,
        ),
        (
            "struct N; fn f(c:bool){let x=N;if c{let _a=x;}let _b=x;}",
            false,
        ),
        (
            "struct N; fn f(c:bool){let mut x=N;if c{let _a=x;}x=N;let _b=x;}",
            true,
        ),
        (
            "struct N; fn f(c:bool){let mut x:(N,N);if c{x=(N,N);}x.0=N;}",
            false,
        ),
        (
            "struct N; fn f(c:bool){let x;if c{x=N;}else{x=N;}let _b=x;}",
            true,
        ),
        ("struct N; fn f(c:bool){let x;if c{x=N;}let _b=x;}", false),
        ("struct N; fn f(c:bool){let x=N;while c{let _a=x;}}", false),
        (
            "struct N; fn f(c:bool){let mut x=N;while c{let _a=x;x=N;}let _b=x;}",
            true,
        ),
        ("struct N; fn f(c:bool){while c{let x=N;let _a=x;}}", true),
        ("struct N; fn f(){let x=N;loop{let _a=x;break;}}", true),
        (
            "struct N; fn f(){let x=N;loop{let _a=x;break;}let _b=x;}",
            false,
        ),
        ("struct N; fn f(a:[N;2],i:usize){let _a=a[i];}", false),
        ("struct N; fn f(a:[N;0],i:usize){let _a=a[i];}", false),
        ("struct N; fn f(a:[N;2],i:usize){let _=a[i];}", true),
        (
            "struct N; fn f(mut a:[N;2],i:usize){a[i]=N;let _b=a;}",
            true,
        ),
        ("struct N; fn f(a:[N;2],i:usize){a[i]=N;}", false),
        ("struct N; fn f(i:usize){let a:[N;2];let _=a[i];}", false),
        ("struct N; fn f(a:[(N,u8);2],i:usize)->u8{a[i].1}", true),
        ("struct N; fn f(a:[(N,u8);2],i:usize)->N{a[i].0}", false),
        ("struct N; fn f(mut a:[(N,u8);2],i:usize){a[i].0=N;}", true),
        ("struct N; fn f(x:N){let _a=[x;0];}", true),
        ("struct N; fn f(x:N){let _a=[x;1];}", true),
        ("struct N; fn f(x:N){let _a=[x;2];}", false),
        ("struct N; fn f(x:N){let _a=[x;0];let _b=x;}", false),
        ("struct N; fn f(){let _a=[N;2];}", false),
        ("struct N; const C:N=N; fn f(){let _a=[C;2];}", true),
        ("struct N; fn f(a:[N;0]){let _a=a;let _b=a;}", false),
        ("struct N; fn f(a:[N;1]){let _a=a;let _b=a;}", false),
        ("fn f(a:[u8;3]){let _a=a;let _b=a;}", true),
        (
            "struct N; struct S{a:N,b:N} fn f(mut x:S){let _a=x.a;x.a=N;let _b=x;}",
            true,
        ),
        (
            "struct N; struct S{a:N,b:N} fn f(x:S){let _a=x.a;let _b=S{a:N,..x};}",
            true,
        ),
        (
            "struct N; struct S{a:N,b:N} fn f(x:S){let _a=x.b;let _b=S{a:N,..x};}",
            false,
        ),
        (
            "struct N; struct S{a:N,b:N} fn f(x:S){let _a=S{a:N,..x};let _b=x.a;}",
            true,
        ),
        (
            "struct N; struct S{a:N,b:N} fn f(x:S){let _a=S{a:N,..x};let _b=x;}",
            false,
        ),
        (
            "#[derive(Clone,Copy)] struct N; fn f(x:N){let _a=[x;2];let _b=x;}",
            true,
        ),
        (
            "#[derive(Clone,Copy)] struct N(u8); fn f(x:N){let _a=x;let _b=x;}",
            true,
        ),
        (
            "#[derive(Clone,Copy)] enum N{A(u8),B} fn f(x:N){let _a=x;let _b=x;}",
            true,
        ),
        ("#[derive(Clone,Copy)] enum N{}", true),
        (
            "#[derive(Clone)] struct N; fn f(x:N){let _a=x;let _b=x;}",
            false,
        ),
        ("#[derive(Copy)] struct N;", false),
        ("#[derive(Clone,Copy,Copy)] struct N;", false),
        ("#[derive(Clone,Clone)] struct N;", false),
        ("struct N; #[derive(Clone,Copy)] struct S(N);", false),
        (
            "#[derive(Clone)] struct N; #[derive(Clone,Copy)] struct S(N);",
            false,
        ),
        (
            "#[derive(Clone)] struct N; #[derive(Clone)] struct S(N);",
            true,
        ),
        (
            "#[derive(Clone)] struct N; #[derive(Clone)] #[repr(packed)] struct S(N);",
            false,
        ),
        (
            "#[derive(Clone)] struct N(u8); #[derive(Clone)] #[repr(packed(2))] struct S(N);",
            false,
        ),
        (
            "#[derive(Clone,Copy)] struct N; #[derive(Clone)] #[repr(packed)] struct S(N);",
            true,
        ),
        ("#[derive(Clone,Copy)] struct S(&'static mut u8);", false),
        (
            "struct N; #[derive(Clone,Copy)] struct S(&'static N);",
            true,
        ),
        ("#[derive(Clone,Copy)] struct S(*const S);", true),
        ("struct N; #[derive(Clone,Copy)] struct S([N;0]);", false),
        (
            "#[cfg_attr(all(),derive(Clone,Copy))] struct N; fn f(x:N){let _a=x;let _b=x;}",
            true,
        ),
        (
            "macro_rules! m{()=>{#[derive(Clone,Copy)] struct N;}} m!(); fn f(x:N){let _a=x;let _b=x;}",
            true,
        ),
        ("fn f(x:&'static mut u8){let _a=x;let _b=x;}", false),
        ("fn f(x:&'static u8){let _a=x;let _b=x;}", true),
        ("fn a(){} fn f(){let x=a;let _a=x;let _b=x;}", true),
        ("struct N; fn f()->N{(N,N).0}", true),
        ("struct N; fn f(){let _x={let a=N;a};}", true),
    ];
    let version = Command::new("rustc").arg("--version").output()?;
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("rustc 1.98.0 "));
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa12-work/ownership-rustc")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory)?;
    for (index, (source, expected)) in cases.iter().enumerate() {
        let prefix = directory.join(format!("case_{index}"));
        std::fs::write(prefix.with_extension("rs"), source)?;
        let result = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--emit=metadata",
                "-o",
            ])
            .arg(prefix.with_extension("rmeta"))
            .arg(prefix.with_extension("rs"))
            .output()?;
        std::fs::write(prefix.with_extension("stderr"), &result.stderr)?;
        assert_eq!(
            result.status.success(),
            *expected,
            "rustc {index}: {source}\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let ours = check(source);
        assert_eq!(
            ours.is_ok(),
            *expected,
            "frontend {index}: {source}\n{:?}",
            ours.err()
        );
    }
    Ok(())
}

#[test]
fn copy_queries_are_iterative_cached_and_independent_of_pointer_referents()
-> Result<(), &'static str> {
    let mut types = Store::default();
    let mut ty = TypeId::UNIT;
    for _ in 0..20_000 {
        ty = types.intern(Kind::Array {
            element: ty,
            length: 0,
        })?;
    }
    let mut copies = Copies::default();
    let definitions = Definitions::default();
    let derives = HashMap::new();
    assert_eq!(
        copies.demand(&types, &definitions, &derives, ty)?,
        Properties {
            copy: true,
            clone_: true
        }
    );
    let steps = copies.steps;
    assert!(copies.demand(&types, &definitions, &derives, ty)?.copy);
    assert_eq!(copies.steps, steps);
    let mutable = types.intern(Kind::Reference {
        mutable: true,
        region: RegionId::STATIC,
        referent: ty,
    })?;
    assert!(!copies.demand(&types, &definitions, &derives, mutable)?.copy);
    let raw = types.intern(Kind::Pointer {
        mutable: true,
        pointee: mutable,
    })?;
    assert!(copies.demand(&types, &definitions, &derives, raw)?.copy);
    Ok(())
}

#[test]
fn move_paths_do_not_expand_array_elements_and_preserve_unmentioned_fields() -> Result<(), String> {
    let text = check("struct N; fn f(a:[N;18446744073709551615],i:usize){let _=a[i];}")?;
    assert!(text.contains("paths=2"), "{text}");
    assert!(check("struct N; fn f(mut a:(N,N,N)){let _x=a.1;a.1=N;let _y=a;}").is_ok());
    assert!(check("struct N; fn f(mut a:(N,N,N)){let _x=a.1;let _y=a;}").is_err());
    let text = check("struct N; fn f(a:(N,N)){let _x=a.0;let _=a.0;let _y=a.1;}")?;
    assert_eq!(text.matches(" nominal-field=").count(), 0);
    assert_eq!(text.matches(" tuple-field=").count(), 2);
    Ok(())
}

#[test]
fn deep_projection_paths_and_many_independent_locals_stay_linear() -> Result<(), String> {
    use std::fmt::Write as _;
    let mut source = String::from("type T0=u8;");
    for n in 1..=10_000 {
        write!(source, "type T{n}=(T{},);", n - 1).map_err(|e| e.to_string())?;
    }
    source.push_str("fn f(x:T10000)->u8{x");
    for _ in 0..10_000 {
        source.push_str(".0");
    }
    source.push('}');
    let text = check(&source)?;
    assert!(text.contains("paths=10001"));
    assert_eq!(text.matches(" copy\n").count(), 1);
    source.clear();
    source.push_str("fn f(){");
    for n in 0..20_000 {
        write!(source, "let x{n}=1u8;let _a{n}=x{n};").map_err(|e| e.to_string())?;
    }
    source.push('}');
    let text = check(&source)?;
    assert!(text.contains("paths=40000"));
    Ok(())
}

#[test]
fn failed_copy_dependencies_are_cached_without_poisoning_independent_queries()
-> Result<(), &'static str> {
    use rustgm::types::{ItemId, aggregate};
    let mut types = Store::default();
    let item = ItemId(0);
    let recursive = types.intern(Kind::Adt(item, rustgm::types::RegionArgs::EMPTY))?;
    let fields = types.intern_list(&[recursive])?;
    let mut definitions = Definitions::default();
    definitions.insert(
        item,
        aggregate::Kind::Struct,
        aggregate::Repr::default(),
        &[aggregate::Variant {
            fields,
            discriminant: 0,
        }],
    )?;
    let derives = HashMap::from([(
        item,
        Properties {
            copy: true,
            clone_: true,
        },
    )]);
    let mut copies = Copies::default();
    assert!(
        copies
            .demand(&types, &definitions, &derives, recursive)
            .is_err()
    );
    let steps = copies.steps;
    assert!(
        copies
            .demand(&types, &definitions, &derives, recursive)
            .is_err()
    );
    assert_eq!(copies.steps, steps);
    assert!(
        copies
            .demand(&types, &definitions, &derives, TypeId::UNIT)?
            .copy
    );
    Ok(())
}

#[test]
fn ownership_cli_counts_the_same_proof_and_rejects_moved_uses()
-> Result<(), Box<dyn std::error::Error>> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa12-work/ownership-cli")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory)?;
    let path = directory.join("input.rs");
    std::fs::write(&path, "struct N; fn f(a:(N,N)){let _x=a.0;let _y=a.1;}")?;
    let full = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-ownership")
        .arg(&path)
        .output()?;
    let count = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--count-ownership")
        .arg(&path)
        .output()?;
    assert!(
        full.status.success(),
        "{}",
        String::from_utf8_lossy(&full.stderr)
    );
    assert!(count.status.success());
    assert!(full.stdout.ends_with(&count.stdout));
    std::fs::write(&path, "struct N; fn f(a:N){let _x=a;let _y=a;}")?;
    let result = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg("--emit-ownership")
        .arg(&path)
        .output()?;
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
    assert!(String::from_utf8_lossy(&result.stderr).contains("uninitialized or moved"));
    Ok(())
}
