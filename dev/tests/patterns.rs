use rustgm::{
    body,
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{self, model::Declarations, patterns::Kind},
    support::source::SourceFile,
};
use std::{
    io::Write,
    path::Path,
    process::{Command, Output, Stdio},
};

struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn analyze(source: &str) -> Result<(Report, Declarations), String> {
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
    let facts = sema::declarations::analyze(&report)
        .and_then(|f| sema::bodies::analyze(&report, f))
        .map_err(|e| e.to_string())?;
    Ok((report, facts))
}
fn compile(source: &str, mode: &str) -> Result<Output, Box<dyn std::error::Error>> {
    let mut child = Command::new(env!("CARGO_BIN_EXE_rustgm"))
        .arg(mode)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .ok_or("missing compiler input")?
        .write_all(source.as_bytes())?;
    Ok(child.wait_with_output()?)
}
const CHECK: &str = "fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}\n";
const EXECUTABLE: &[(&str, &str)] = &[
    (
        "tuple",
        "fn main(){let (x,y)=(10u64,7u8);check(x==10&&y==7);}",
    ),
    (
        "tuple-rest",
        "fn main(){let p=(1u8,2u16,(3u32,4u64));let (a,..,(b,c))=p;check(a==1&&b==3&&c==4);}",
    ),
    (
        "uninitialized",
        "fn main(){let (x,mut y):(u8,u8);x=1;y=2;y+=3;check(x==1&&y==5);}",
    ),
    (
        "partial-move",
        "struct N(u8);fn main(){let p=(N(3),N(7));let(a,_)=p;let b=p.1;check(a.0==3&&b.0==7);}",
    ),
    (
        "at-copy-fields",
        "struct N(u8);fn main(){let n=N(3);let whole@N(x)=n;check(whole.0==3&&x==3);}",
    ),
    (
        "at-tuple",
        "fn main(){let whole@(x,(y,z))=(1u8,(2u16,3u32));check(whole.0==x&&whole.1.0==y&&z==3);}",
    ),
    (
        "record-rest",
        "mod m{pub struct S{a:u8,pub b:u64}pub fn make()->S{S{a:1,b:7}}}fn main(){let m::S{b,..}=m::make();check(b==7);}",
    ),
    (
        "tuple-struct-rest",
        "struct S(u8,u16,u32,u64);fn main(){let S(a,..,d)=S(1,2,3,4);check(a==1&&d==4);}",
    ),
    (
        "parameter",
        "fn f((a,mut b):(u8,u64))->u64{b+=1;a as u64+b}fn main(){check(f((3,5))==9);}",
    ),
    (
        "record-parameter",
        "struct N(u64);struct S{a:N,b:N}fn f(S{a:N(x),b}:S)->u64{x+b.0}fn main(){check(f(S{a:N(7),b:N(9)})==16);}",
    ),
    (
        "wildcard-uninitialized",
        "struct S{a:u8,b:u64}struct Z;fn main(){let p:(u8,u8);let(_, _)=p;let s:S;let S{..}=s;let z:Z;let Z=z;let u:();let ()=u;check(true);}",
    ),
    (
        "single-variant",
        "#[repr(C,u8)]enum E{A(u64,u8)}fn main(){let E::A(a,b)=E::A(13,2);check(a==13&&b==2);}",
    ),
    (
        "alternatives",
        "fn main(){let ((x,y)|(y,x))=(1u8,2u8);check(x==1&&y==2);}",
    ),
    (
        "macro-pattern",
        "macro_rules! take{($p:pat,$e:expr)=>{let $p=$e;}}fn main(){take!((x,y),(1u8,2u16));check(x==1&&y==2);}",
    ),
    (
        "place-effects",
        "fn main(){let a=[(3u8,4u16)];let mut i=0;let (x,y)=a[{i+=1;0}];check(i==1&&x==3&&y==4);}",
    ),
    (
        "divergence",
        "fn f()->u8{let (a,b):(u8,u8)=return 3;a+b}fn main(){check(f()==3);}",
    ),
    (
        "zst-effects",
        "fn main(){let mut n=0;let ((),())={n+=1;((),())};check(n==1);}",
    ),
    (
        "record-numeric",
        "struct S(u8,u64);fn main(){let S{1:b,0:a}=S(3,7);check(a==3&&b==7);}",
    ),
];
const REJECTED: &[&str] = &[
    "fn main(){let (x,y)=(1u8,);}",
    "fn main(){let (x,)=1u8;}",
    "fn main(){let (x,..,..)=(1u8,2u8);}",
    "struct S{a:u8}fn main(){let S{b}=S{a:1};}",
    "struct S{a:u8}fn main(){let S{a:x,a:y}=S{a:1};}",
    "struct S{a:u8,b:u8}fn main(){let S{a}=S{a:1,b:2};}",
    "mod m{pub struct S{a:u8}pub fn make()->S{S{a:1}}}fn main(){let m::S{a}=m::make();}",
    "mod m{pub struct S(u8);pub fn make()->S{S(1)}}fn main(){let m::S(_)=m::make();}",
    "struct Z;fn main(){let Z()=Z;}",
    "fn main(){let ((x,y)|(y,x))=(1u8,2u16);}",
    "struct N(u8);fn main(){let p=(N(3),N(7));let(a,_)=p;let _whole=p;}",
    "struct N(u8);fn main(){let whole@(x,_)=(N(1),N(2));}",
    "fn main(){let (x,y)=(1u8,2u8);x=3;}",
    "fn main(){let p:(u8,u8);let(x,_)=p;}",
    "enum E{A,B}fn main(){let E::A=E::A;}",
];

#[test]
fn product_patterns_select_sparse_fields_and_complete_their_types() -> Result<(), String> {
    let (report, facts) =
        analyze("fn f((a,..,(b,c)):(u8,bool,(u16,u32)))->u32{a as u32+b as u32+c}")?;
    assert_eq!(facts.patterns.roots.len(), 1);
    assert!(
        facts
            .patterns
            .nodes
            .iter()
            .all(|p| p.ty.is_some() && !matches!(p.kind, Kind::Pending))
    );
    let root = *facts
        .patterns
        .roots
        .values()
        .next()
        .ok_or("missing pattern root")?;
    let Kind::Product { fields, .. } = facts.patterns.nodes[root.0 as usize].kind else {
        return Err("missing tuple selection".into());
    };
    let fields = facts.patterns.fields(fields);
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].selection, sema::members::Selection::Tuple(0));
    assert_eq!(fields[1].selection, sema::members::Selection::Tuple(2));
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    assert_eq!(program.bodies[0].projections.len(), 4);
    Ok(())
}
#[test]
fn wildcard_products_do_not_create_value_reads() -> Result<(), String> {
    let (report, facts) = analyze("fn main(){let p:(u8,u16);let(_, _)=p;}")?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    assert!(
        !program.bodies[0]
            .values
            .iter()
            .any(|v| matches!(v.kind, body::ir::Rvalue::Read(_)))
    );
    Ok(())
}
#[test]
fn tuple_rest_tracks_only_selected_fields_and_alternatives_keep_shared_bindings()
-> Result<(), String> {
    let source = format!(
        "type T=({});fn f((a,..,b):T){{let _=(a,b);}}",
        vec!["()"; 16_000].join(",")
    );
    let (report, facts) = analyze(&source)?;
    assert_eq!(facts.patterns.nodes.len(), 3);
    assert_eq!(facts.patterns.fields.len(), 2);
    assert_eq!(
        facts.patterns.fields[1].selection,
        sema::members::Selection::Tuple(15_999)
    );
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    assert_eq!(program.bodies[0].projections.len(), 2);
    let (_, facts) = analyze("fn f(v:(u8,u8)){let ((x,y)|(y,x))=v;let _=(x,y);}")?;
    let bindings: Vec<_> = facts
        .patterns
        .nodes
        .iter()
        .filter_map(|pattern| {
            if let Kind::Binding { definition, .. } = pattern.kind {
                Some(definition)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(bindings.len(), 4);
    assert_eq!(bindings[0], bindings[3]);
    assert_eq!(bindings[1], bindings[2]);
    assert_eq!(std::mem::size_of::<sema::model::NodeFacts>(), 80);
    Ok(())
}
#[test]
fn owned_product_programs_emit_atomic_deterministic_lowir() -> Result<(), Box<dyn std::error::Error>>
{
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/product-vectors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (name, source) in EXECUTABLE {
        let source = format!("{CHECK}{source}");
        let output = compile(&source, "--emit-lowir")?;
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        assert_eq!(output.stdout, compile(&source, "--emit-lowir")?.stdout);
        let count = compile(&source, "--count-lowir")?;
        assert!(count.status.success());
        assert!(count.stderr.is_empty());
        std::fs::write(folder.join(format!("{name}.rs")), source)?;
        std::fs::write(folder.join(format!("{name}.lowir")), output.stdout)?;
        let executable = folder.join(name);
        let built = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "pattern_values",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
                "-C",
                "debug-assertions=off",
            ])
            .arg(folder.join(format!("{name}.rs")))
            .arg("-o")
            .arg(&executable)
            .output()?;
        assert!(
            built.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        let executed = Command::new(executable).output()?;
        assert!(
            executed.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&executed.stderr)
        );
        assert!(executed.stdout.is_empty() && executed.stderr.is_empty());
    }
    Ok(())
}
#[test]
fn product_errors_and_move_conflicts_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/product-errors")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    for (index, source) in REJECTED.iter().enumerate() {
        let output = compile(source, "--emit-lowir")?;
        assert_eq!(output.status.code(), Some(1), "{source}");
        assert!(output.stdout.is_empty(), "{source}");
        assert!(!output.stderr.is_empty(), "{source}");
        let file = folder.join(format!("case{index}.rs"));
        std::fs::write(&file, source)?;
        let result = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "pattern_check",
                "--emit=metadata",
            ])
            .arg(&file)
            .arg("-o")
            .arg(folder.join("metadata"))
            .output()?;
        assert_eq!(
            result.status.code(),
            Some(1),
            "{source}: {}",
            String::from_utf8_lossy(&result.stderr)
        );
    }
    Ok(())
}
