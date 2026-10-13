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
        "array-elements",
        "fn main(){let [a,b,c]=[2u64,3,5];check(a+b+c==10);}",
    ),
    (
        "array-rest",
        "fn main(){let [a,..,b]=[2u8,3,5,7];check(a==2&&b==7);}",
    ),
    (
        "array-rest-binding",
        "fn main(){let [a,middle@..,b]=[2u64,3,5,7];check(a==2&&middle[0]==3&&middle[1]==5&&b==7);}",
    ),
    (
        "array-prefix",
        "fn main(){let [prefix@..,z]=[2u8,3,5];check(prefix[0]==2&&prefix[1]==3&&z==5);}",
    ),
    (
        "array-suffix",
        "fn main(){let [a,tail@..]=[2u8,3,5];check(a==2&&tail[0]==3&&tail[1]==5);}",
    ),
    (
        "array-parameter",
        "fn f([a,tail@..]:[u64;3])->u64{a+tail[0]*2+tail[1]*3}fn main(){check(f([2,3,5])==23);}",
    ),
    (
        "array-return-rest",
        "fn f([_,tail@..]:[u64;3])->[u64;2]{tail}fn main(){let a=f([2,3,5]);check(a[0]==3&&a[1]==5);}",
    ),
    (
        "array-uninitialized-bindings",
        "fn main(){let [a,mut b]:[u8;2];a=3;b=7;b+=2;check(a==3&&b==9);}",
    ),
    (
        "array-disjoint",
        "struct N(u8);fn main(){let xs=[N(2),N(3)];let[a,_]=xs;let[_,b]=xs;check(a.0==2&&b.0==3);}",
    ),
    (
        "array-disjoint-rest",
        "struct N(u8);fn main(){let xs=[N(2),N(3),N(5)];let[a,..]=xs;let[_,tail@..]=xs;check(a.0==2&&tail[0].0==3&&tail[1].0==5);}",
    ),
    (
        "array-disjoint-prefix",
        "struct N(u8);fn main(){let xs=[N(2),N(3),N(5)];let[_,tail@..]=xs;let[a,..]=xs;check(a.0==2&&tail[0].0==3&&tail[1].0==5);}",
    ),
    (
        "array-disjoint-middle",
        "struct N(u8);fn main(){let xs=[N(2),N(3),N(5),N(7)];let[a,..,b]=xs;let[_,middle@..,_]=xs;check(a.0==2&&middle[0].0==3&&middle[1].0==5&&b.0==7);}",
    ),
    (
        "array-move-rest-and-ends",
        "struct N(u8);fn main(){let xs=[N(2),N(3),N(5),N(7)];let[a,middle@..,b]=xs;check(a.0==2&&middle[0].0==3&&middle[1].0==5&&b.0==7);}",
    ),
    (
        "array-disjoint-subfields",
        "struct N(u8);struct S{a:N,b:N}fn main(){let xs=[S{a:N(2),b:N(3)}];let[S{a,..}]=xs;let[S{b,..}]=xs;check(a.0==2&&b.0==3);}",
    ),
    (
        "array-copy-subfields",
        "struct N(u8);fn main(){let xs=[N(2),N(3)];let[N(a),_]=xs;let ys=xs;check(a==2&&ys[1].0==3);}",
    ),
    (
        "array-at-copy-fields",
        "struct N(u8);fn main(){let xs=[N(2),N(3)];let whole@[N(a),N(b)]=xs;check(a==2&&b==3&&whole[0].0==2);}",
    ),
    (
        "array-wildcard-uninitialized",
        "struct N(u8);fn main(){let a:[N;2];let[_,_]=a;let b:[N;3];let[..]=b;let z:[N;0];let[]=z;check(true);}",
    ),
    (
        "array-empty-rest",
        "struct N(u8);fn main(){let xs=[N(2)];let[_,a@..]=xs;let[_,b@..]=xs;let ys=xs;let _=(a,b);check(ys[0].0==2);}",
    ),
    (
        "array-empty-rest-after-move",
        "struct N(u8);fn main(){let xs=[N(2)];let[a]=xs;let[_,tail@..]=xs;let _=tail;check(a.0==2);}",
    ),
    (
        "array-zero-rest",
        "struct N(u8);fn main(){let xs:[N;0]=[];let[a@..]=xs;let[b@..]=xs;let _=(a,b);let ys=xs;let _=ys;check(true);}",
    ),
    (
        "array-nested",
        "fn main(){let [[a,..,b],rest@..]=[[1u64,2,3],[4,5,6],[7,8,9]];check(a==1&&b==3&&rest[0][2]==6&&rest[1][0]==7);}",
    ),
    (
        "array-nested-moves",
        "struct N(u8);fn main(){let xs=([N(2),N(3)],N(5));let([a,_],_)=xs;let([_,b],c)=xs;check(a.0==2&&b.0==3&&c.0==5);}",
    ),
    (
        "array-alternatives",
        "fn main(){let ([x,y]|[y,x])=[2u8,3];check(x==2&&y==3);}",
    ),
    (
        "array-rest-alternatives",
        "fn main(){let ([a@..,_]|[_,a@..])=[2u8,3];check(a[0]==2);}",
    ),
    (
        "array-place-effects",
        "fn main(){let xs=[[1u8,2]];let mut n=0;let[a,b]=xs[{n+=1;0}];check(a==1&&b==2&&n==1);}",
    ),
    (
        "array-macro",
        "macro_rules! take{($p:pat,$e:expr)=>{let $p=$e;}}fn main(){take!([a,tail@..],[2u8,3,5]);check(a==2&&tail[1]==5);}",
    ),
    (
        "array-reinitialize",
        "struct N(u8);fn main(){let mut xs=[N(2),N(3)];let[a,_]=xs;xs=[N(5),N(7)];let[b,c]=xs;check(a.0==2&&b.0==5&&c.0==7);}",
    ),
    (
        "array-conditional-disjoint",
        "struct N(u8);fn f(c:bool)->u8{let xs=[N(2),N(3),N(5)];if c{let[a,..]=xs;let _=a;}let[_,tail@..]=xs;tail[0].0+tail[1].0}fn main(){check(f(true)==8&&f(false)==8);}",
    ),
    (
        "array-loop-storage",
        "struct N(u8);fn main(){let mut n=0u8;let mut sum=0;while n<3{let xs=[N(n),N(5)];let[a,b]=xs;sum+=a.0+b.0;n+=1;}check(sum==18);}",
    ),
    (
        "array-huge-zst",
        "fn main(){let xs=[();18446744073709551615];let[a,tail@..,b]=xs;let _=(a,tail,b);check(true);}",
    ),
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
    "fn main(){let[a@b@..]=[1u8,2];let _=(a,b);}",
    "struct N(u8);fn main(){let xs:[N;0]=[];let[a@b@..]=xs;let _=(a,b);}",
    "struct N(u8);fn main(){let xs:[[N;0];1]=[[]];let[a@..]=xs[0];let ys=xs;let _=(a,ys);}",
    "fn main(){let[a,b]=[1u8];}",
    "fn main(){let[a,..,b]=[1u8];}",
    "fn main(){let[a]=1u8;}",
    "fn main(){let[a@..,..]=[1u8,2];}",
    "fn main(){let[a@..,b@..]=[1u8,2];}",
    "fn main(){let(a@..,)=(1u8,);}",
    "fn main(){let([a@..,_]|[_,_,a@..])=[1u8,2,3];}",
    "struct N(u8);fn main(){let xs=[N(1),N(2)];let[a,_]=xs;let _=xs[1].0;}",
    "struct N(u8);fn main(){let xs=[N(1),N(2)];let[a,_]=xs;let[b,_]=xs;}",
    "struct N(u8);fn main(){let xs=[N(1),N(2),N(3)];let[_,tail@..]=xs;let[_,b,_]=xs;}",
    "struct N(u8);fn main(){let xs=[N(1),N(2),N(3)];let[a,..]=xs;let[all@..]=xs;}",
    "struct N(u8);struct S{a:N,b:N}fn main(){let xs=[S{a:N(1),b:N(2)}];let[S{a,..}]=xs;let[rest@..]=xs;}",
    "struct N(u8);fn main(){let xs:[N;2];let[a,_]=xs;}",
    "struct N(u8);fn main(){let xs:[N;2];let[a@..]=xs;}",
    "struct N(u8);fn main(){let xs:[N;0]=[];let a@[..]=xs;let b=xs;}",
    "struct N(u8);fn main(){let mut xs=[N(1),N(2)];let[a,_]=xs;xs[0]=N(3);}",
    "struct N(u8);fn main(){let xs=[[N(1),N(2)]];let[a,b]=xs[0];}",
    "struct N(u8);fn main(){let xs=[N(1),N(2)];let all@[a,_]=xs;}",
    "struct N(u8);fn main(){let xs=[N(1),N(2)];let[_,a@..]=xs;let[b@..,_]=xs;let all=xs;}",
    "struct N(u8);fn f(c:bool){let xs=[N(1),N(2)];if c{let[a,_]=xs;}let[b,_]=xs;}fn main(){}",
    "struct N(u8);fn f(c:bool){let xs=[N(1),N(2)];while c{let[_,tail@..]=xs;}}fn main(){}",
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
    assert_eq!(
        fields[0].selection,
        sema::patterns::Projection::Field(sema::members::Selection::Tuple(0))
    );
    assert_eq!(
        fields[1].selection,
        sema::patterns::Projection::Field(sema::members::Selection::Tuple(2))
    );
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
        sema::patterns::Projection::Field(sema::members::Selection::Tuple(15_999))
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
fn array_patterns_keep_huge_rest_storage_sparse() -> Result<(), String> {
    let (report, mut facts) = analyze("fn f([a,..,b]:[();18446744073709551615]){let _=(a,b);}")?;
    assert_eq!(facts.patterns.nodes.len(), 3);
    assert_eq!(facts.patterns.fields.len(), 2);
    assert_eq!(
        facts.patterns.fields[1].selection,
        sema::patterns::Projection::Index(u64::MAX - 1)
    );
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let checked = rustgm::borrow::ownership::check(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        &program,
    )
    .map_err(|e| e.message.to_owned())?;
    let paths = &checked.bodies[0].paths;
    assert_eq!(paths.entries.len(), 5);
    assert_eq!(paths.bits, 6);
    Ok(())
}
#[test]
fn array_subarray_move_ranges_cover_nested_fields_and_empty_ranges_are_empty() -> Result<(), String>
{
    use rustgm::borrow::ownership::{self, Component};
    let (report, mut facts) = analyze(
        "struct N(u8);struct S{a:N,b:N}fn f(xs:[S;3]){let[S{a,..},..]=xs;let[_,tail@..]=xs;let[_,_,_,empty@..]=xs;let _=(a,tail,empty);}",
    )?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    let checked = ownership::check(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        &program,
    )
    .map_err(|e| e.message.to_owned())?;
    let paths = &checked.bodies[0].paths;
    let element = paths
        .entries
        .iter()
        .find(|p| p.component == Some(Component::Index(0)))
        .ok_or("missing element move path")?;
    let rest = paths
        .entries
        .iter()
        .find(|p| p.component == Some(Component::Subarray { start: 1, end: 3 }))
        .ok_or("missing rest move path")?;
    let empty = paths
        .entries
        .iter()
        .find(|p| p.component == Some(Component::Subarray { start: 3, end: 3 }))
        .ok_or("missing empty rest move path")?;
    assert!(element.end <= rest.start);
    assert_eq!(empty.start, empty.end);
    assert!(
        paths
            .entries
            .iter()
            .any(|p| p.parent.is_some() && p.start > element.start && p.end <= element.end)
    );
    assert!(paths.bits < 20);
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

fn range_pattern(length: usize, start: usize, end: usize, name: &str) -> String {
    let mut parts = vec!["_".to_owned(); start];
    parts.push(format!("{name}@.."));
    parts.extend(std::iter::repeat_n("_".to_owned(), length - end));
    format!("[{}]", parts.join(","))
}
#[test]
fn every_pair_of_small_array_ranges_matches_rustc_move_rules()
-> Result<(), Box<dyn std::error::Error>> {
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/array-range-matrix")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&folder)?;
    let mut count = 0;
    for length in 0..=4 {
        let ranges: Vec<_> = (0..=length)
            .flat_map(|start| (start..=length).map(move |end| (start, end)))
            .collect();
        for (&(a, b), &(c, d)) in ranges
            .iter()
            .flat_map(|first| ranges.iter().map(move |second| (first, second)))
        {
            let allowed = a == b || c == d || b <= c || d <= a;
            let source = format!(
                "struct N(u8);fn f(xs:[N;{length}]){{let {}=xs;let {}=xs;}}fn main(){{}}",
                range_pattern(length, a, b, "first"),
                range_pattern(length, c, d, "second")
            );
            let (report, mut facts) = analyze(&source)?;
            let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
            let actual = rustgm::borrow::ownership::check(
                &facts.types,
                &facts.aggregates,
                &facts.derives,
                &mut facts.copies,
                &program,
            );
            assert_eq!(actual.is_ok(), allowed, "{source}");
            let path = folder.join(format!("{length}-{a}-{b}-{c}-{d}.rs"));
            std::fs::write(&path, &source)?;
            let rustc = Command::new("rustc")
                .args([
                    "--edition=2024",
                    "--crate-name",
                    "array_ranges",
                    "--emit=metadata",
                ])
                .arg(&path)
                .arg("-o")
                .arg(folder.join("metadata"))
                .output()?;
            std::fs::write(path.with_extension("stderr"), &rustc.stderr)?;
            assert_eq!(
                rustc.status.code(),
                Some(i32::from(!allowed)),
                "{source}: {}",
                String::from_utf8_lossy(&rustc.stderr)
            );
            count += 1;
        }
    }
    assert_eq!(count, 371);
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

#[test]
fn borrowed_patterns_keep_input_types_capture_modes_and_region_provenance() -> Result<(), String> {
    use rustgm::{
        sema::{model::BodyRegionOrigin, patterns::Capture},
        types::{Kind as Type, Primitive, RegionId, TypeId},
    };
    let (_, facts) = analyze(
        "fn f(xs:&&mut[u8;1]){let[x]=xs;}fn g(ys:&mut[u8;1]){let[y]=ys;}fn h(mut z:u8){let ref mut a=z;}",
    )?;
    let bindings: Vec<_> = facts
        .patterns
        .nodes
        .iter()
        .filter_map(|p| {
            if let Kind::Binding {
                definition,
                mutable,
                capture,
                ..
            } = p.kind
            {
                Some((p, definition, mutable, capture))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(bindings.len(), 3);
    assert_eq!(
        facts
            .body_regions
            .iter()
            .filter(|r| matches!(r.origin, BodyRegionOrigin::PatternBorrow))
            .count(),
        3
    );
    let mut previous_body = None;
    for (index, (pattern, definition, local_mutable, capture)) in bindings.iter().enumerate() {
        assert!(!local_mutable);
        assert_eq!(pattern.ty, Some(TypeId::primitive(Primitive::U8)));
        let Capture::Borrow { mutable, region } = capture else {
            return Err("missing borrow capture".into());
        };
        assert_eq!(*mutable, index != 0);
        assert_ne!(*region, RegionId::STATIC);
        assert_eq!(region.bound_parts(), None);
        let origin = &facts.body_regions[region.0 as usize - 1];
        assert!(matches!(origin.origin, BodyRegionOrigin::PatternBorrow));
        assert_eq!(origin.source, pattern.source);
        if let Some(previous) = previous_body {
            assert_ne!(origin.body, previous);
        }
        previous_body = Some(origin.body);
        assert!(
            matches!(facts.types.kind(facts.definitions[definition.0 as usize].ok_or("missing local type")?),
            Some(Type::Reference { mutable: m, region: r, referent }) if m == *mutable && r == *region && Some(referent) == pattern.ty)
        );
    }
    assert_eq!(
        facts
            .patterns
            .nodes
            .iter()
            .filter(|p| matches!(p.kind, Kind::Dereference { .. }))
            .count(),
        3
    );
    Ok(())
}
#[test]
fn alternatives_share_local_types_without_erasing_per_occurrence_captures() -> Result<(), String> {
    use rustgm::sema::patterns::Capture;
    for qualifier in ["", "mut "] {
        let (_, facts) = analyze(&format!(
            "fn f(xs:(&{qualifier}u8,&{qualifier}[u8;1])){{let((x,_)|(_,[x]))=xs;}}"
        ))?;
        let bindings: Vec<_> = facts
            .patterns
            .nodes
            .iter()
            .filter_map(|p| {
                if let Kind::Binding {
                    definition,
                    capture,
                    ..
                } = p.kind
                {
                    Some((definition, capture))
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(bindings.len(), 2);
        assert_eq!(bindings[0].0, bindings[1].0);
        assert_eq!(bindings[0].1, Capture::Value);
        assert!(
            matches!(bindings[1].1, Capture::Borrow { mutable, .. } if mutable != qualifier.is_empty())
        );
        assert!(!facts.region_equalities.is_empty());
        assert_eq!(
            facts
                .body_regions
                .iter()
                .filter(|r| matches!(
                    r.origin,
                    rustgm::sema::model::BodyRegionOrigin::PatternBorrow
                ))
                .count(),
            1
        );
    }
    Ok(())
}
#[test]
fn slice_rest_binds_a_reference_to_an_unsized_place() -> Result<(), String> {
    use rustgm::{sema::patterns::Projection, types::Kind as Type};
    let (_, facts) = analyze("fn f(xs:&[u8]){let[tail@..]=xs;}")?;
    assert!(facts.patterns.fields.iter().any(|f| f.selection
        == Projection::Subslice {
            prefix: 0,
            suffix: 0
        }));
    let pattern = facts
        .patterns
        .nodes
        .iter()
        .find(|p| matches!(p.kind, Kind::Binding { .. }))
        .ok_or("missing slice binding")?;
    let Kind::Binding { definition, .. } = pattern.kind else {
        return Err("missing binding".into());
    };
    let input = pattern.ty.ok_or("missing slice input type")?;
    assert!(matches!(facts.types.kind(input), Some(Type::Slice(_))));
    assert_eq!(facts.types.is_sized(input), Some(false));
    let local = facts.definitions[definition.0 as usize].ok_or("missing slice local type")?;
    assert!(
        matches!(facts.types.kind(local), Some(Type::Reference { mutable: false, referent, .. }) if referent == input)
    );
    assert_eq!(facts.types.is_sized(local), Some(true));
    Ok(())
}

#[test]
fn scalar_coverage_retains_redundancy_without_counting_guards() -> Result<(), String> {
    let (_, facts) =
        analyze("fn f(x:u8)->u8{match x{0..=5 if false=>1,0..=1=>2,0..=1=>3,2..=255=>4}}")?;
    assert_eq!(facts.patterns.decisions.len(), 1);
    let decision = &facts.patterns.decisions[0];
    assert_eq!(decision.start, 0);
    assert_eq!(decision.end, 4);
    assert_eq!(
        facts
            .patterns
            .arms
            .iter()
            .map(|a| a.useful)
            .collect::<Vec<_>>(),
        [true, true, false, true]
    );
    assert!(decision.work > 0 && decision.work <= rustgm::sema::patterns::scalar::Coverage::LIMIT);
    assert!(facts.patterns.arms[0].guard.is_some());
    assert!(facts.patterns.arms[1..].iter().all(|a| a.guard.is_none()));
    Ok(())
}

#[test]
fn scalar_witnesses_are_target_values_and_errors_are_atomic()
-> Result<(), Box<dyn std::error::Error>> {
    for (source, expected) in [
        ("fn f(x:bool){match x{true=>()}}", "missing false"),
        ("fn f(x:u8){match x{0..3=>(),4.. =>()}}", "missing 3"),
        ("fn f(x:i8){match x{0.. =>()}}", "missing -128"),
        (
            "fn f(x:char){match x{'\\0'..='\\u{d7ff}'=>()}}",
            "missing '\\u{e000}'",
        ),
    ] {
        for mode in ["--emit-patterns", "--count-patterns"] {
            let result = compile(source, mode)?;
            assert_eq!(result.status.code(), Some(1));
            assert!(result.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&result.stderr).contains(expected),
                "{}",
                String::from_utf8_lossy(&result.stderr)
            );
        }
    }
    Ok(())
}

#[test]
fn guarded_scalar_inspection_retains_a_read_only_view() -> Result<(), Box<dyn std::error::Error>> {
    // Inspection retains a protected view; executable lowering must prove
    // its use is read-only instead of silently copying it before the guard.
    let source = "fn main(){match 1u8{mut x if {x=2;true}=>(),_=>()}}";
    for mode in ["--emit-patterns", "--count-patterns", "--emit-body-ir"] {
        let result = compile(source, mode)?;
        assert!(result.status.success());
        assert!(result.stderr.is_empty());
    }
    for mode in ["--emit-lowir", "--count-lowir"] {
        let result = compile(source, mode)?;
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(String::from_utf8_lossy(&result.stderr).contains("guard binding"));
    }
    Ok(())
}

#[test]
fn conditional_guards_retain_selected_facts_and_protected_views() -> Result<(), String> {
    let (report, facts) =
        analyze("fn f(x:u8)->u8{match x{n if let n=n&&n>0&&let m=n+1=>m,_=>99}}")?;
    assert_eq!(facts.patterns.conditions.len(), 2);
    assert_eq!(facts.patterns.chains.len(), 4);
    let guarded: Vec<_> = facts.patterns.arms.iter().filter_map(|a| a.guard).collect();
    assert_eq!(guarded.len(), 1);
    let boolean = rustgm::types::TypeId::primitive(rustgm::types::Primitive::Bool);
    assert_eq!(facts.node(guarded[0]).and_then(|f| f.ty), Some(boolean));
    let program = body::build(&report, &facts).map_err(|error| error.to_string())?;
    let guards: usize = program.bodies.iter().map(|body| body.guards.len()).sum();
    assert_eq!(guards, 1);
    assert!(program.bodies.iter().any(|body| {
        body.locals
            .iter()
            .any(|local| matches!(local.kind, rustgm::body::ir::LocalKind::Guard(_)))
    }));
    Ok(())
}
