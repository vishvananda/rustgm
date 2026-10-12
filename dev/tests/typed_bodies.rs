#[path = "support/body_machine.rs"]
mod body_machine;
use body_machine::{Data, Machine};
use rustgm::{
    body::{
        self,
        ir::{Owner, Program},
    },
    expand::config::Config,
    resolve::{
        self,
        expansion::Report,
        load::{InputFile, Sources},
    },
    sema::{bodies, declarations, model::Declarations},
    support::source::SourceFile,
};
use std::{path::Path, process::Command};
struct NoFiles;
impl Sources for NoFiles {
    fn read(&mut self, _: &Path) -> Result<Option<InputFile>, String> {
        Ok(None)
    }
}
fn analyze(source: &str) -> Result<(Report, Declarations, Program), String> {
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
    let facts = declarations::analyze(&report)
        .and_then(|f| bodies::analyze(&report, f))
        .map_err(|e| e.to_string())?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    Ok((report, facts, program))
}
const CASES: &[(&str, &str, u128)] = &[
    (
        "storage-ancestry",
        "fn entry()->u64{let mut out=0;let a=5;{let a=100;out+=a;}let b=7;'outer:loop{let c=11;{let gone=13;out+=gone;}let d=17;out+=a+b+c+d;break 'outer;}out}",
        153,
    ),
    (
        "macro-binding-scope",
        "macro_rules! bind{($v:ident,$e:expr)=>{let $v=$e;}}fn entry()->u64{bind!(x,40);{bind!(x,7);let _=x;}bind!(y,2);x+y}",
        42,
    ),
    (
        "wildcard-uninitialized-place",
        "fn entry()->u64{let x:u64;let _=(x);let y:();let _=y;0}",
        0,
    ),
    ("literal", "fn entry()->u64{42}", 42),
    (
        "binary-snapshot",
        "fn entry()->u64{let mut x=1;x+{x=2;x}}",
        3,
    ),
    (
        "argument-order",
        "fn sum(a:u64,b:u64)->u64{a*10+b} fn entry()->u64{let mut x=1;sum(x,{x=9;x})}",
        19,
    ),
    (
        "callee-before-arguments",
        "fn a(x:u64)->u64{x+1} fn b(x:u64)->u64{x+20} fn entry()->u64{let mut f:fn(u64)->u64=a;f({f=b;1})}",
        2,
    ),
    (
        "compound-rhs-first",
        "fn entry()->u64{let mut x=1;x+={x=10;3};x}",
        13,
    ),
    (
        "assignment-rhs-first",
        "fn entry()->u64{let mut x=1;x={x=5;9};x}",
        9,
    ),
    (
        "and-skips",
        "fn entry()->u64{let mut x=3;false&&{x=9;true};x}",
        3,
    ),
    (
        "or-skips",
        "fn entry()->u64{let mut x=3;true||{x=9;false};x}",
        3,
    ),
    (
        "nested-side-effects",
        "fn entry()->u64{let mut x=0;(true&&{x+=1;false})||{x+=2;true};x}",
        3,
    ),
    (
        "nested-short-circuit",
        "fn entry()->u64{let a=false;let b=true;((a&&(b||a))||(b&&(a||b))) as u64}",
        1,
    ),
    (
        "nested-if",
        "fn entry()->u64{if true{if false{1}else{2}}else{3}}",
        2,
    ),
    (
        "else-if",
        "fn entry()->u64{if false{1}else if true{2}else{3}}",
        2,
    ),
    ("while", "fn entry()->u64{let mut x=0;while x<5{x+=1;}x}", 5),
    (
        "loop-continue",
        "fn entry()->u64{let mut x=0;loop{x+=1;if x<4{continue;}break x;}}",
        4,
    ),
    (
        "while-continue",
        "fn entry()->u64{let mut n=0;let mut sum=0;while n<5{n+=1;if n==3{continue;}sum+=n;}sum}",
        12,
    ),
    (
        "outer-break",
        "fn entry()->u64{'a:loop{loop{break 'a 9;}}}",
        9,
    ),
    (
        "outer-continue",
        "fn entry()->u64{let mut x=0;'a:loop{x+=1;loop{if x<3{continue 'a;}break 'a x;}}}",
        3,
    ),
    ("labeled-block", "fn entry()->u64{'a:{break 'a 7;}}", 7),
    (
        "block-fallthrough",
        "fn entry()->u64{'a:{if false{break 'a 7;}8}}",
        8,
    ),
    ("early-return", "fn entry()->u64{if true{return 12;}99}", 12),
    (
        "condition-effects",
        "fn entry()->u64{let mut x=0;while {x+=1;x<4}{}x}",
        4,
    ),
    (
        "condition-short-circuit",
        "fn entry()->u64{let mut x=0;while (x<4)&&(x!=2){x+=1;}x}",
        2,
    ),
    (
        "labeled-condition-break",
        "fn entry()->u64{let mut x=0;'a:while {x+=1;if x==3{break 'a;}true}{}x}",
        3,
    ),
    (
        "branch-initialization",
        "fn entry()->u64{let x;if true{x=5;}else{x=6;}x}",
        5,
    ),
    ("unit-call", "fn unit(){} fn entry()->u64{unit();42}", 42),
    (
        "recursive-call",
        "fn fact(n:u64)->u64{if n==0{1}else{n*fact(n-1)}} fn entry()->u64{fact(5)}",
        120,
    ),
    (
        "function-pointer-join",
        "fn a()->u64{7} fn b()->u64{8} fn entry()->u64{let f:fn()->u64=loop{if true{break a;}break b;};f()}",
        7,
    ),
    ("grouped-binding", "fn entry()->u64{let (x)=4;let _=9;x}", 4),
    (
        "shadowing",
        "fn entry()->u64{let x=1;{let x=2;let _=x;}x}",
        1,
    ),
    (
        "signed-shift",
        "fn entry()->u64{let x:i8=-8;((x>>1)==-4) as u64}",
        1,
    ),
    (
        "const-initializers",
        "const A:u64=B+1;const B:u64=8;fn entry()->u64{A}",
        9,
    ),
    (
        "static-initializer",
        "static A:u64=17;fn entry()->u64{A}",
        17,
    ),
];
#[test]
fn region_conversions_remain_explicit_obligations() -> Result<(), String> {
    let (_, facts, program) = analyze("fn f<'a,'b>(x:&'a u8,y:&'b u8)->&'a u8{y}")?;
    assert_eq!(program.region_equalities.len(), 1);
    assert_eq!(facts.region_equalities.len(), 1);
    let body = &program.bodies[0];
    let conversion = body
        .values
        .iter()
        .find(|value| matches!(value.kind, body::ir::Rvalue::Constrain(_)))
        .ok_or("missing constrained conversion")?;
    assert_eq!(conversion.ty, body.result);
    Ok(())
}

#[test]
fn aggregate_places_preserve_evaluation_order_and_partial_update_reads() -> Result<(), String> {
    use body::ir::{Action, Place, ProjectionKind, Rvalue};
    use rustgm::sema::members::{FieldId, Selection};
    let (report, mut facts, program) = analyze(
        "struct S{a:u8,b:u16} fn rhs()->u8{7} fn index()->usize{1} fn make()->[u8;2]{[3,4]} fn f(mut a:[u8;2],s:S)->(u8,S){a[index()]+=rhs();let n=make()[index()];let t=S{a:rhs(),..s};let _=a[index()];(n,t)}",
    )?;
    let body = program.bodies.last().ok_or("missing aggregate body")?;
    let mut calls = Vec::new();
    let mut index_projections = 0;
    let mut projected_reads = 0;
    for statement in &body.statements {
        match statement.action {
            Action::Project(id) => {
                if let ProjectionKind::Index { length, .. } = body.projections[id.0 as usize].kind {
                    assert_eq!(length, 2);
                    index_projections += 1;
                }
            }
            Action::Evaluate(id) => match body.values[id.0 as usize].kind {
                Rvalue::Call { callee, .. } => {
                    let Rvalue::Function(definition) = body.values[callee.0 as usize].kind else {
                        return Err("missing direct call selection".into());
                    };
                    calls.push(
                        report
                            .package
                            .names
                            .get(report.resolution.definitions[definition.0 as usize].name),
                    );
                }
                Rvalue::Read(Place::Projection(_)) => projected_reads += 1,
                Rvalue::Read(Place::Local(id)) => assert!(
                    id.0 >= 2,
                    "aggregate parameters must be projected without a whole-value read"
                ),
                _ => {}
            },
            _ => {}
        }
    }
    assert_eq!(calls, ["rhs", "index", "make", "index", "rhs", "index"]);
    assert_eq!(index_projections, 3);
    assert_eq!(projected_reads, 3); // compound LHS, indexed temporary, missing update field
    let fields: Vec<_> = body
        .projections
        .iter()
        .filter_map(|p| {
            if let ProjectionKind::Field(field) = p.kind {
                Some(field)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(fields, [Selection::Nominal(FieldId(1))]);
    assert!(
        rustgm::borrow::ownership::check(
            &facts.types,
            &facts.aggregates,
            &facts.derives,
            &mut facts.copies,
            &program,
        )
        .is_ok()
    );
    Ok(())
}

#[test]
fn aggregate_place_stacks_balance_across_divergence_and_nested_cfgs() -> Result<(), String> {
    for source in [
        "fn f(){let mut a=[1,2];a[0]=return;}",
        "fn f(){let mut a=[1,2];a[return]=1;}",
        "fn f(b:bool)->i32{let a=[1,2];a[if b{return 3}else{0}]}",
        "fn f(b:bool){let mut a=[1,2];a[if b{return}else{0}]=3;}",
        "fn f(){let a=[1,2];let _=a[return];}",
        "struct S{a:i32} fn f(s:S){let t=S{a:return,..s};}",
        "struct S{a:i32} fn f(s:S){let t=S{a:1,..{return;s}};}",
        "fn f()->i32{let a=([1,2],);a.0[0]}",
        "fn f(){let a:(u8,u8);let _=a.0;}",
        "struct S{a:u8} fn f(){let a:S;let _=a.a;}",
    ] {
        analyze(source).map_err(|e| format!("{source}: {e}"))?;
    }
    Ok(())
}

#[test]
fn tuple_constructors_are_typed_bodies_and_repetition_stays_compact() -> Result<(), String> {
    use body::ir::{Action, Rvalue};
    let (report, facts, program) = analyze(
        "struct S(u8,u16);enum E{V(u8),Empty()}fn f()->S{let ctor=S;ctor(1,2)}fn huge(){let a=[();18446744073709551615usize];}",
    )?;
    let constructors: Vec<_> = program.bodies.iter().filter(|body| matches!(body.owner,Owner::Function{definition,..} if matches!(report.resolution.definitions[definition.0 as usize].kind,rustgm::resolve::model::DefKind::Constructor|rustgm::resolve::model::DefKind::Variant))).collect();
    assert_eq!(constructors.len(), 3);
    for body in constructors {
        let result = body.values.last().ok_or("missing constructor result")?;
        let Rvalue::Aggregate {
            variant: Some(_),
            fields,
        } = result.kind
        else {
            return Err("missing typed constructor aggregate".into());
        };
        assert_eq!(body.arguments(fields).len(), body.locals.len());
        assert_eq!(
            body.statements
                .iter()
                .filter(|s| matches!(s.action, Action::StorageLive(_)))
                .count(),
            body.locals.len()
        );
    }
    let repeated: Vec<_> = program
        .bodies
        .iter()
        .flat_map(|b| &b.values)
        .filter_map(|v| {
            if let Rvalue::Repeat { .. } = v.kind
                && let Some(rustgm::types::Kind::Array { length, .. }) = facts.types.kind(v.ty)
            {
                Some(length)
            } else {
                None
            }
        })
        .collect();
    assert_eq!(repeated, [u64::MAX]);
    assert!(std::mem::size_of::<body::ir::Value>() <= 24);
    assert!(program.bodies.iter().all(|b| b.values.len() < 20));
    Ok(())
}

#[test]
fn deep_field_projection_is_iterative_and_reads_only_the_selected_leaf() -> Result<(), String> {
    use body::ir::{Place, Rvalue};
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
    let (_, _, program) = analyze(&source)?;
    let body = program.bodies.first().ok_or("missing projected body")?;
    assert_eq!(body.projections.len(), 10_000);
    assert_eq!(
        body.values
            .iter()
            .filter(|v| matches!(v.kind, Rvalue::Read(Place::Projection(_))))
            .count(),
        1
    );
    assert!(
        !body
            .values
            .iter()
            .any(|v| matches!(v.kind, Rvalue::Read(Place::Local(_))))
    );
    Ok(())
}
#[test]
fn deep_bodies_are_iterative_and_views_are_deterministic() -> Result<(), String> {
    let source = format!("fn entry()->u64{{0{}}}", "+1".repeat(10_000));
    let (report, facts, program) = analyze(&source)?;
    let Owner::Function { definition, .. } = program.bodies[0].owner else {
        return Err("expected function".into());
    };
    let actual = Machine {
        program: &program,
        types: &facts.types,
        fuel: 100_000,
    }
    .run(definition, &[], 0)?;
    assert_eq!(actual, Data::Bits(10_000));
    let mut first = Vec::new();
    let mut second = Vec::new();
    let mut count = Vec::new();
    body::body_view::write(&program, &facts.types, &mut first).map_err(|e| e.to_string())?;
    let rebuilt = body::build(&report, &facts).map_err(|e| e.to_string())?;
    body::body_view::write(&rebuilt, &facts.types, &mut second).map_err(|e| e.to_string())?;
    body::body_view::count(&program, &mut count).map_err(|e| e.to_string())?;
    assert_eq!(first, second);
    assert!(first.ends_with(&[b"eof ", count.as_slice()].concat()));
    Ok(())
}
#[test]
fn cfg_execution_agrees_with_declared_results_and_rustc() -> Result<(), String> {
    let directory = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa10-work/runtime")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    for (index, (name, source, expected)) in CASES.iter().enumerate() {
        let fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("../pa10/tests/spec/execution/{name}.rs"));
        assert_eq!(
            std::fs::read_to_string(fixture)
                .map_err(|e| e.to_string())?
                .trim_end(),
            *source
        );
        let (report, facts, program) =
            analyze(source).map_err(|e| format!("case {index}: {source}: {e}"))?;
        let entry = program
            .bodies
            .iter()
            .find_map(|body| match body.owner {
                Owner::Function { definition, .. }
                    if report
                        .package
                        .names
                        .get(report.resolution.definitions[definition.0 as usize].name)
                        == "entry" =>
                {
                    Some(definition)
                }
                _ => None,
            })
            .ok_or("no entry function")?;
        let actual = Machine {
            program: &program,
            types: &facts.types,
            fuel: 100_000,
        }
        .run(entry, &[], 0)
        .map_err(|e| format!("case {index}: {source}: {e}"))?;
        assert_eq!(actual, Data::Bits(*expected), "case {index}: {source}");
        let path = directory.join(format!("case-{index}.rs"));
        let binary = directory.join(format!("case-{index}"));
        std::fs::write(
            &path,
            format!("{source}\nfn main(){{print!(\"{{}}\",entry());}}\n"),
        )
        .map_err(|e| e.to_string())?;
        let compiled = Command::new("rustc")
            .args([
                "--edition=2024",
                "--crate-name",
                "body_check",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=on",
            ])
            .arg(&path)
            .arg("-o")
            .arg(&binary)
            .output()
            .map_err(|e| e.to_string())?;
        if !compiled.status.success() {
            return Err(format!(
                "rustc case {index}: {}",
                String::from_utf8_lossy(&compiled.stderr)
            ));
        }
        let ran = Command::new(binary).output().map_err(|e| e.to_string())?;
        assert!(ran.status.success(), "rustc runtime case {index}");
        assert!(ran.stderr.is_empty(), "rustc stderr case {index}");
        assert_eq!(
            ran.stdout,
            expected.to_string().as_bytes(),
            "rustc result case {index}"
        );
    }
    Ok(())
}

#[test]
fn conditional_storage_balances_success_failure_and_reentry() -> Result<(), String> {
    let cases = [
        (
            "fn entry()->u64{let x=7;let n@1..=9=x else{return 99;};n}",
            7,
        ),
        ("fn entry()->u64{let x=1;let n@0=x else{return 7;};n}", 7),
        (
            "fn entry()->u64{let x=10;let y=3;let n@0..10=x else{let m@0..10=y else{return 99;};return m+10;};n}",
            13,
        ),
        (
            "fn entry()->u64{let x=7;if let n@1..=9=x&&let m=n&&m==7{n+m}else{0}}",
            14,
        ),
        (
            "fn entry()->u64{let mut x=4;let mut sum=0;while let mut n@1..=4=x&&n>0{x-=1;if n==3{continue;}n+=1;sum+=n;}sum}",
            10,
        ),
        (
            "fn entry()->u64{let x=7;if let n@1..=9=x&&false{n}else{9}}",
            9,
        ),
        (
            "fn entry()->u64{let x=1;'out:loop{let n@0=x else{break 'out 9;};break n;}}",
            9,
        ),
    ];
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../obj/pa13-work/conditional-storage")
        .join(std::process::id().to_string());
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (index, (source, expected)) in cases.iter().enumerate() {
        let (_, facts, program) = analyze(source)?;
        let Owner::Function { definition, .. } = program.bodies[0].owner else {
            return Err("missing entry function".into());
        };
        let actual = Machine {
            program: &program,
            types: &facts.types,
            fuel: 100_000,
        }
        .run(definition, &[], 0)?;
        assert_eq!(actual, Data::Bits(*expected), "{source}");
        let file = dir.join(format!("case{index}.rs"));
        std::fs::write(
            &file,
            format!("{source}\nfn main(){{print!(\"{{}}\",entry());}}\n"),
        )
        .map_err(|e| e.to_string())?;
        let executable = dir.join(format!("case{index}"));
        let compiled = Command::new("rustc")
            .args([
                "--edition=2024",
                "-C",
                "panic=abort",
                "-C",
                "overflow-checks=yes",
            ])
            .arg(&file)
            .arg("-o")
            .arg(&executable)
            .output()
            .map_err(|e| e.to_string())?;
        std::fs::write(file.with_extension("stderr"), &compiled.stderr)
            .map_err(|e| e.to_string())?;
        assert!(
            compiled.status.success(),
            "{source}: {}",
            String::from_utf8_lossy(&compiled.stderr)
        );
        let ran = Command::new(executable)
            .output()
            .map_err(|e| e.to_string())?;
        assert!(ran.status.success() && ran.stderr.is_empty());
        assert_eq!(ran.stdout, expected.to_string().as_bytes());
    }
    Ok(())
}

#[test]
fn storage_ranges_follow_live_ancestry_and_reject_invalid_links() -> Result<(), String> {
    use body::ir::{Action, LocalId};
    let source = std::fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../pa10/tests/spec/execution/storage-ancestry.rs"),
    )
    .map_err(|e| e.to_string())?;
    let (_, _, mut program) = analyze(&source)?;
    let body = &mut program.bodies[0];
    let mut skipped_closed_local = false;
    let mut selected = None;
    for statement in &body.statements {
        if let Action::StorageEnd { first, last } = statement.action {
            let locals: Vec<_> = body.storage_range(first, last).collect::<Result<_, _>>()?;
            assert_eq!(locals.first(), Some(&last));
            assert_eq!(locals.last(), Some(&first));
            skipped_closed_local |= locals.windows(2).any(|pair| pair[0].0 > pair[1].0 + 1);
            selected = Some((first, last));
        }
    }
    assert!(skipped_closed_local);
    let (first, last) = selected.ok_or("missing compact cleanup")?;
    body.locals[last.0 as usize].storage_parent = Some(last);
    assert!(
        body.storage_range(first, last)
            .next()
            .ok_or("empty range")?
            .is_err()
    );
    body.locals[last.0 as usize].storage_parent = None;
    assert!(
        body.storage_range(first, last)
            .next()
            .ok_or("empty range")?
            .is_err()
    );
    assert!(
        body.storage_range(first, LocalId(u32::MAX))
            .next()
            .ok_or("empty range")?
            .is_err()
    );
    Ok(())
}
