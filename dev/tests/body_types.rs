use rustgm::{
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{
        bodies, declarations,
        model::{AdjustmentKind, Cast, Declarations, Operation, Scalar},
    },
    support::source::SourceFile,
    types::{Primitive, TypeId},
};
use std::{
    io::Write as _,
    path::Path,
    process::{Command, Stdio},
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
    let declarations = declarations::analyze(&report).map_err(|e| e.to_string())?;
    let facts = bodies::analyze(&report, declarations).map_err(|e| e.to_string())?;
    Ok((report, facts))
}
fn definition(report: &Report, facts: &Declarations, name: &str) -> Result<TypeId, String> {
    report
        .resolution
        .definitions
        .iter()
        .enumerate()
        .find(|(_, d)| report.package.names.get(d.name) == name)
        .and_then(|(i, _)| facts.definitions[i])
        .ok_or_else(|| format!("missing type for {name}"))
}
#[test]
fn body_constraints_delay_fallback_and_retain_selected_operations() -> Result<(), String> {
    let (report, facts) = analyze(
        "fn pass(x:u8)->u8{x} fn f()->u8{let a=17; let b=pass(a); let default_integer=1; let default_float=1.0; b+1}",
    )?;
    for name in ["a", "b"] {
        assert_eq!(
            definition(&report, &facts, name)?,
            TypeId::primitive(Primitive::U8)
        );
    }
    assert_eq!(
        definition(&report, &facts, "default_integer")?,
        TypeId::primitive(Primitive::I32)
    );
    assert_eq!(
        definition(&report, &facts, "default_float")?,
        TypeId::primitive(Primitive::F64)
    );
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Call { .. })))
    );
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.scalar, Some(Scalar::Integer(17))))
    );
    Ok(())
}
#[test]
fn casts_function_reification_and_minimum_literals_publish_lowering_facts() -> Result<(), String> {
    let (_, facts) = analyze(
        "type F=fn(u8)->u8; fn pass(x:u8)->u8{x} fn f()->i8{let callback:F=pass; let n=callback(1) as i16; -(128i8)}",
    )?;
    assert!(facts.nodes.iter().flatten().any(|n| matches!(
        n.operation,
        Some(Operation::Cast {
            kind: Cast::IntegerToInteger,
            ..
        })
    )));
    assert!(facts.nodes.iter().flatten().any(|n| {
        n.adjustment
            .is_some_and(|a| a.kind == AdjustmentKind::FunctionPointer)
    }));
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.scalar, Some(Scalar::Integer(128))))
    );
    assert!(
        facts
            .nodes
            .iter()
            .flatten()
            .any(|n| matches!(n.operation, Some(Operation::Unary { literal: true, .. })))
    );
    Ok(())
}
#[test]
fn scalar_body_acceptance_matches_pinned_rustc() -> Result<(), String> {
    let cases = [
        ("fn f()->u8{1+2}", true),
        ("fn f()->u8{let _:()=();1}", true),
        ("fn f()->u8{let g=a as fn()->u8;g()} fn a()->u8{1}", true),
        ("fn f(){let g=a as fn()->u16;} fn a()->u8{1}", false),
        (
            "fn f()->u8{let f:fn(u8)->u8=g;f(1)} fn g(x:u8)->u8{x}",
            true,
        ),
        ("fn f()->u8{let f:fn(_)->_=g;f(1)} fn g(x:u8)->u8{x}", true),
        ("fn f(){let f:fn(_)->_;}", false),
        ("fn f(){let f:fn(u8)->u8;}", true),
        ("fn f(){let _:for<'a> fn(&'a i32)->&'a i32;}", true),
        ("fn f(){let _:(u8,bool);}", true),
        ("fn f(){let _:(u8,_);}", false),
        ("fn f(){let _:[u8;4];}", true),
        ("fn f(){let _:[_;4];}", false),
        ("fn f(){let _:[u8;4u8];}", false),
        ("fn f(){let _:*const u8;}", true),
        ("fn f(){#[cfg(any())] let _:Missing;}", true),
        ("fn f()->u8{let _=return 2;}", true),
        ("fn f()->u8{let x=return 2;}", true),
        ("fn f()->u8{if true{return 1}else{let _=return 2;}}", true),
        ("fn f()->f32{1e100}", false),
        ("fn f()->f64{1e100}", true),
        ("fn f()->f64{1e1000}", false),
        (
            "fn f()->i128{-170141183460469231731687303715884105728}",
            true,
        ),
        (
            "fn f()->i128{170141183460469231731687303715884105728}",
            false,
        ),
        ("fn f(){let a=return;}", true),
        ("fn f()->u8{if true{return 1}else{return 2}}", true),
        ("fn f()->u8{let a=1;a}", true),
        ("fn f()->u8{let a=1;let b=a;b}", true),
        ("fn f(){let a=1;let b=2.0;}", true),
        ("fn f(){let a;}", false),
        ("fn f(){let a:_;}", false),
        ("fn f(){let a:u8;}", true),
        ("fn f()->u8{let a:i16=1;a}", false),
        ("fn f()->u8{1i16}", false),
        ("fn f()->u8{256}", false),
        ("fn f()->i8{128}", false),
        ("fn f()->i8{-128}", true),
        ("fn f()->i8{-(128)}", true),
        ("fn f()->i8{-129}", false),
        ("fn f()->u8{-1}", false),
        ("fn f()->u8{!1}", true),
        ("fn f()->bool{!false}", true),
        ("fn f()->f32{!1.0}", false),
        ("fn f()->bool{true+false}", false),
        ("fn f()->bool{true&false}", true),
        ("fn f()->bool{'a'<'b'}", true),
        ("fn f()->bool{false<true}", true),
        ("fn f()->bool{1u8==1u16}", false),
        ("fn f()->u8{1u8<<2u16}", true),
        ("fn f()->u8{1u8<<2.0}", false),
        ("fn f()->bool{true&&false}", true),
        ("fn f()->bool{true&&1}", false),
        ("fn f()->bool{true&&(return false)}", true),
        ("fn f()->u8{if true{1}else{2}}", true),
        ("fn f()->u8{if true{return 1}else{2}}", true),
        ("fn f()->u8{if true{1}else{return 2}}", true),
        ("fn f(){if true{1}}", false),
        ("fn f(){if true{return}}", true),
        ("fn f()->u8{return 1;}", true),
        ("fn f()->u8{return;}", false),
        ("fn f()->u8{return 1;false}", false),
        ("fn f()->u8{g(1)} fn g(x:u8)->u8{x}", true),
        ("fn f(){g();} fn g(x:u8){}", false),
        ("fn f(){g(1,2);} fn g(x:u8){}", false),
        ("fn f(){g(false);} fn g(x:u8){}", false),
        ("fn f()->u8{let h=g;h(2)} fn g(x:u8)->u8{x}", true),
        ("fn f()->u8{257u16 as u8}", true),
        ("fn f()->u8{1.5 as u8}", true),
        ("fn f()->f32{1u8 as f32}", true),
        ("fn f()->f32{true as f32}", false),
        ("fn f()->u8{true as u8}", true),
        ("fn f()->u32{'a' as u32}", true),
        ("fn f()->char{65u8 as char}", true),
        ("fn f()->char{65u32 as char}", false),
        ("fn f()->bool{1 as bool}", false),
        ("fn f()->u8{let _:u16=1;2}", true),
        ("fn f()->u8{#[cfg(any())] let x=missing;2}", true),
        ("macro_rules! m{()=>{1+2}} fn f()->u8{m!()}", true),
    ];
    check_cases("scalar", &cases)
}

fn check_cases(name: &str, cases: &[(&str, bool)]) -> Result<(), String> {
    let version = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| e.to_string())?;
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("rustc 1.98.0 "));
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa9-work/body-rustc")
        .join(std::process::id().to_string())
        .join(name);
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    for (index, (source, expected)) in cases.iter().enumerate() {
        let prefix = directory.join(index.to_string());
        std::fs::write(prefix.with_extension("rs"), source).map_err(|e| e.to_string())?;
        let mut child = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=body_check",
                "--emit=metadata",
                "-o",
            ])
            .arg(prefix.with_extension("rmeta"))
            .arg("-")
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|e| e.to_string())?;
        child
            .stdin
            .take()
            .ok_or("missing rustc stdin")?
            .write_all(source.as_bytes())
            .map_err(|e| e.to_string())?;
        let result = child.wait_with_output().map_err(|e| e.to_string())?;
        std::fs::write(prefix.with_extension("stderr"), &result.stderr)
            .map_err(|e| e.to_string())?;
        assert_eq!(
            result.status.success(),
            *expected,
            "rustc case {index}: {source}\n{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let ours = analyze(source);
        assert_eq!(
            ours.is_ok(),
            *expected,
            "frontend case {index}: {source}\n{:?}",
            ours.err()
        );
    }
    Ok(())
}

#[test]
fn aggregate_expression_types_match_pinned_rustc() -> Result<(), String> {
    check_cases(
        "aggregates",
        &[
            ("struct N; fn f(){return;let _a=[N;2];}", false),
            ("struct N; fn f()->[N;2]{[return f();2]}", false),
            (
                "#[derive(Clone,Copy)]struct N; fn f(){return;let _a=[N;2];}",
                true,
            ),
            ("struct N; const C:N=N; fn f(){return;let _a=[C;2];}", true),
            ("fn f()->(u8,bool){(1,true)}", true),
            ("fn f()->u8{let a=(1,false);a.0}", true),
            ("fn f()->u8{let a:((u8,),bool)=((1,),false);a.0.0}", true),
            ("fn f(){let a=(1,false);let _=a.2;}", false),
            ("fn f(){let a=(1,false);let _=a.no;}", false),
            ("fn f(){let a=(1,false);let _=a.00;}", false),
            ("fn f()->(u8,bool){(1,2)}", false),
            ("fn f()->[u8;3]{[1,2,3]}", true),
            ("fn f()->[u8;3]{[1,2]}", false),
            ("fn f()->[u8;0]{[]}", true),
            ("fn f(){let a=[];}", false),
            ("fn f()->u8{let a=[1,2,3];a[1]}", true),
            ("fn f(x:usize)->u8{let a=[4;3usize];a[x]}", true),
            ("fn f(x:u8)->u8{let a=[4;3];a[x]}", false),
            ("fn f()->[u8;2]{[1,true]}", false),
            ("fn f()->[(u8,bool);2]{[(1,true),(2,false)]}", true),
            ("fn f(){let a:[_;2]=[1u8,2];let b:u8=a[0];}", true),
            ("fn f(){let mut a=(1u8,false);a.0=2;a.1=true;}", true),
            ("fn f(){let mut a=[1u8,2];a[0]=3;a[1]+=4;}", true),
            ("fn f(){let mut a=[1u8,2];a[0]=false;}", false),
            ("fn g()->u8{1} fn f()->(fn()->u8,){(g,)}", true),
            (
                "fn g()->u8{1} fn h()->u8{2} fn f()->u8{let a=[g,h];a[0]()}",
                true,
            ),
            ("fn g()->u8{1} fn f()->[fn()->u8;2]{[g,g]}", true),
            ("fn g()->u8{1} fn f()->[fn()->u8;2]{[g;2]}", true),
            ("struct S{a:u8,b:bool} fn f()->S{S{b:true,a:1}}", true),
            ("struct S{a:u8,b:bool} fn f()->S{S{a:1}}", false),
            ("struct S{a:u8} fn f()->S{S{a:1,a:2}}", false),
            ("struct S{a:u8} fn f()->S{S{a:1,b:2}}", false),
            ("struct S{a:u8} fn f()->S{S{a:false}}", false),
            ("struct S{a:u8} type A=S; fn f()->S{A{a:1}}", true),
            ("struct S{a:u8} fn f()->u8{let s=S{a:1};s.a}", true),
            ("struct S{a:u8} fn f(){let mut s=S{a:1};s.a+=2;}", true),
            ("struct S{a:u8,b:bool} fn f(s:S)->S{S{a:2,..s}}", true),
            ("struct S{a:u8} struct T{a:u8} fn f(s:T)->S{S{..s}}", false),
            (
                "struct S{a:u8} fn f(s:S)->S{S{#[cfg(any())] missing:false,..s}}",
                true,
            ),
            ("struct S; fn f()->S{S}", true),
            ("struct S; fn f()->S{S{}}", true),
            ("struct S(); fn f()->S{S()}", true),
            ("struct S(u8,bool); fn f()->S{S(1,true)}", true),
            ("struct S(u8,bool); fn f()->S{S{1:true,0:1}}", true),
            ("struct S(u8,bool); fn f(s:S)->u8{s.0}", true),
            (
                "struct S(u8); fn f()->S{let ctor:fn(u8)->S=S;ctor(3)}",
                true,
            ),
            ("struct S(u8); fn f()->S{S(false)}", false),
            ("enum E{A,B(u8),C{a:u8}} fn f()->E{E::A}", true),
            ("enum E{A,B(u8),C{a:u8}} fn f()->E{E::B(1)}", true),
            ("enum E{A,B(u8),C{a:u8}} fn f()->E{E::C{a:1}}", true),
            ("enum E{A,B(u8),C{a:u8}} fn f()->E{E::B{0:1}}", true),
            ("enum E{A,B(u8),C{a:u8}} fn f(e:E)->u8{e.0}", false),
            ("enum E{A,B{a:u8}} fn f(e:E)->E{E::B{a:1,..e}}", false),
            (
                "mod m{pub struct S{pub x:u8,hidden:bool}} fn f(s:m::S)->u8{s.x}",
                true,
            ),
            (
                "mod m{pub struct S{pub x:u8,hidden:bool}} fn f(s:m::S)->bool{s.hidden}",
                false,
            ),
            (
                "mod m{pub struct S{pub x:u8,hidden:bool}} fn f(s:m::S)->m::S{m::S{x:1,..s}}",
                false,
            ),
            (
                "mod m{pub struct S(pub u8,pub(crate) bool);} fn f()->m::S{m::S(1,true)}",
                true,
            ),
            (
                "mod m{pub struct S(pub u8,bool);} fn f(s:m::S)->bool{s.1}",
                false,
            ),
            ("struct S{é:u8} fn f()->u8{let s=S{e\u{301}:1};s.é}", true),
            (
                "macro_rules! make{()=>{struct S{v:u8}}} make!(); fn f()->u8{S{v:1}.v}",
                true,
            ),
        ],
    )
}

#[test]
fn aggregate_selections_keep_nominal_field_ids_and_array_bounds() -> Result<(), String> {
    use rustgm::sema::members::Selection;
    let (_, facts) = analyze(
        "struct S{a:u8,b:[u16;2]} fn f()->u16{let mut s=S{b:[4,5],a:1};s.b[1]=6;s.b[0]+s.b[1]}",
    )?;
    let mut initializers = Vec::new();
    let mut fields = Vec::new();
    let mut indices = 0;
    let mut writes = 0;
    for fact in facts.nodes.iter().flatten() {
        match fact.operation {
            Some(Operation::FieldInit { field }) => initializers.push(field.0),
            Some(Operation::Field {
                field: Selection::Nominal(field),
            }) => fields.push(field.0),
            Some(Operation::Index { length }) => {
                assert_eq!(length, 2);
                indices += 1;
            }
            Some(Operation::AssignPlace { left, right, .. }) => {
                assert_eq!(left, right);
                assert_eq!(left, TypeId::primitive(Primitive::U16));
                writes += 1;
            }
            _ => {}
        }
    }
    assert_eq!(initializers, [1, 0]);
    assert_eq!(fields, [1, 1, 1]);
    assert_eq!((indices, writes), (3, 1));
    assert_eq!(std::mem::size_of::<rustgm::sema::model::NodeFacts>(), 80);
    Ok(())
}

#[test]
fn control_flow_and_assignment_types_match_pinned_rustc() -> Result<(), String> {
    check_cases(
        "control",
        &[
            ("fn f()->u8{loop{}}", true),
            ("fn f()->u8{loop{break 1}}", true),
            ("fn f()->u8{loop{break}}", false),
            ("fn f(){loop{1}}", false),
            ("fn f(){loop{continue}}", true),
            ("fn f(){while false{}}", true),
            ("fn f()->u8{while false{}}", false),
            ("fn f(){while 1{}}", false),
            ("fn f(){while false{1}}", false),
            ("fn f(){while true{break;}}", true),
            ("fn f(){while true{break ();}}", false),
            ("fn f(){while true{continue;}}", true),
            ("fn f()->u8{loop{if true{break 1}else{break 2}}}", true),
            ("fn f(){loop{if true{break 1}else{break false}}}", false),
            ("fn f()->u8{'a:loop{while true{break 'a 2;}}}", true),
            ("fn f(){'a:loop{while true{continue 'a;}}}", true),
            ("fn f()->u8{'a:{break 'a 2;}}", true),
            ("fn f()->u8{'a:{if true{break 'a 2;}3}}", true),
            ("fn f(){'a:{continue 'a;}}", false),
            ("fn f(){loop{'b:{break;}}}", false),
            ("fn f(){loop{'b:{continue;}}}", false),
            ("fn f(){while {break;true}{}}", false),
            ("fn f(){while {continue;true}{}}", false),
            ("fn f(){'a:while {break 'a;true}{}}", true),
            ("fn f(){'a:while {continue 'a;true}{}}", true),
            ("fn f(){break;}", false),
            ("fn f(){continue;}", false),
            ("fn f()->u8{let mut x=1;x=2;x}", true),
            ("fn f()->u8{let mut x=1;x+=2;x}", true),
            ("fn f(){let mut x:u8=1;x+=2u16;}", false),
            ("fn f()->u8{let mut x=1;x<<=2u16;x}", true),
            ("fn f(){let mut x=1;x+=false;}", false),
            ("fn f(){let mut x=true;x&=false;}", true),
            ("fn f(){let mut x=1.0;x&=2.0;}", false),
            ("fn f(){1=2;}", false),
            ("fn f(){f=f;}", false),
            ("fn f()->u8{let mut x=0; x=return 2;}", true),
            ("fn f()->u8{let mut x=0; x+=return 2;}", false),
            ("fn f()->u8{let mut x=0; x<<=return 2;}", false),
            ("fn f()->u8{1+(return 2)}", false),
            (
                "fn f()->u8{let f=if true{a}else{b};f()} fn a()->u8{1} fn b()->u8{2}",
                true,
            ),
            (
                "fn f()->u8{let f=if true{a}else{b};f()} fn a()->u8{1} fn b()->u16{2}",
                false,
            ),
            ("fn f()->u8{let x; x=2; x}", true),
            ("fn f()->u8{let mut x=0;while x<10{x+=1;}x}", true),
            (
                "fn f()->u8{let f=loop{if true{break a;}break b;};f()} fn a()->u8{1} fn b()->u8{2}",
                false,
            ),
            (
                "fn f()->u8{let f='b:{if true{break 'b a;}b};f()} fn a()->u8{1} fn b()->u8{2}",
                false,
            ),
            ("fn f()->u8{loop{break return 2}}", true),
            (
                "fn f()->u8{let f:fn()->u8='b:{if true{break 'b a;}b};f()} fn a()->u8{1} fn b()->u8{2}",
                true,
            ),
            (
                "fn f()->u8{let mut f:fn()->u8=a; f=loop{if true{break a;}break b;};f()} fn a()->u8{1} fn b()->u8{2}",
                true,
            ),
            (
                "fn f()->u8{let f:fn()->u8=loop{if true{break a;}break b;};f()} fn a()->u8{1} fn b()->u8{2}",
                true,
            ),
            (
                "fn f()->fn()->u8{loop{if true{break a;}break b;}} fn a()->u8{1} fn b()->u8{2}",
                true,
            ),
            (
                "fn f()->u8{take(loop{if true{break a;}break b;})} fn take(f:fn()->u8)->u8{f()} fn a()->u8{1} fn b()->u8{2}",
                true,
            ),
        ],
    )
}

#[test]
fn demanded_shapes_and_initializers_match_pinned_rustc() -> Result<(), String> {
    check_cases(
        "wellformed",
        &[
            ("type A=str;", true),
            ("type A=[str];", true),
            ("type A=[str;0];", true),
            ("type A=(str,u8);", true),
            ("type A=fn(str);", true),
            ("fn f(x:str){}", false),
            ("fn f()->str{loop{}}", false),
            ("fn f(x:&str){}", true),
            ("fn f(x:&[str]){}", false),
            ("fn f(x:&[str;0]){}", false),
            ("fn f(x:&(str,u8)){}", false),
            ("fn f(x:&(u8,str)){}", true),
            ("fn f(x:fn(str)){}", true),
            ("fn f(x:fn()->str){}", true),
            ("fn f(x:fn([str])){}", false),
            ("fn f(){let x:str;}", false),
            ("fn f(){let _:str;}", true),
            ("fn f(){let _:[str];}", false),
            ("const A:u8=17; fn f()->u8{A}", true),
            ("const A:u8=false;", false),
            ("const A:u8=256;", false),
            ("const A:u8={return 2;};", false),
            ("const A:u8;", false),
            ("const A:u8=B; const B:u8=2;", true),
            (r#"const A:str="abc";"#, false),
            ("static A:u8=17; fn f()->u8{A}", true),
            ("static A:u8=false;", false),
            ("static A:u8=256;", false),
            ("static A:u8;", false),
            ("fn f()->u8{const A:u8=2;A}", true),
            ("fn f(){const A:u8=false;}", false),
        ],
    )
}
#[test]
fn control_and_initializer_facts_keep_lowering_targets() -> Result<(), String> {
    let (report, facts) = analyze(
        "const N:u8=2; fn f()->u8{'outer:loop{let mut n=0;n+=1;while n<N{break 'outer n;}}}",
    )?;
    assert_eq!(facts.initializers.len(), 1);
    assert_eq!(facts.initializers[0].ty, TypeId::primitive(Primitive::U8));
    let mut transfers = 0;
    let mut assignments = 0;
    for (file, nodes) in facts.nodes.iter().enumerate() {
        for node in nodes {
            if let Some(Operation::Transfer { target, continuing }) = node.operation {
                assert!(!continuing);
                let target = rustgm::resolve::unit::NodeKey {
                    file: rustgm::resolve::unit::FileId(
                        u32::try_from(file).map_err(|e| e.to_string())?,
                    ),
                    node: target,
                };
                transfers += 1;
                assert!(matches!(
                    report
                        .package
                        .unit(target.file)
                        .configured
                        .ast
                        .node(target.node)
                        .kind,
                    rustgm::syntax::ast::Kind::Loop { .. }
                ));
                assert_eq!(
                    facts.node(target).and_then(|n| n.ty),
                    Some(TypeId::primitive(Primitive::U8))
                );
            }
            if let Some(Operation::Assign { target, .. }) = node.operation {
                assignments += 1;
                assert_eq!(
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[target.0 as usize].name),
                    "n"
                );
            }
        }
    }
    assert_eq!(transfers, 1);
    assert_eq!(assignments, 1);
    Ok(())
}
