#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{body, borrow};

const CASES: &[(&str, bool)] = &[
    (
        "macro_rules! id{($e:expr)=>{$e}}fn f()->&'static u8{&(7u8/id!(2))}",
        true,
    ),
    (
        "macro_rules! id{($e:expr)=>{$e}}fn f()->&'static u8{&(7u8/id!({2}))}",
        false,
    ),
    ("fn f()->&'static u8{&7}", true),
    ("fn f()->&'static u8{&(1+2)}", true),
    ("fn f()->&'static u8{&(255u8+1)}", true),
    ("fn f(){if false{let x:&'static u8=&(255u8+1);}}", true),
    ("fn f()->&'static u8{&(1u8<<8)}", true),
    ("fn f()->&'static u8{&(7u8/0)}", false),
    ("fn f(){if false{let x:&'static u8=&(7u8/0);}}", false),
    ("fn f()->&'static u8{&(7u8/2)}", true),
    ("fn f()->&'static u8{&(7u8/(1+1))}", false),
    ("const C:u8=1+1;fn f()->&'static u8{&(7u8/C)}", true),
    ("fn f()->&'static i8{&(-(-128i8))}", true),
    ("fn f()->&'static u8{&(if true{1}else{2})}", false),
    ("fn f()->&'static bool{&(true&&true)}", false),
    ("fn effect(){}fn f()->&'static u8{&{effect();7}}", true),
    ("fn f()->&'static u8{&{let x=7;x}}", false),
    ("fn f()->&'static u8{&(1,2).0}", true),
    ("fn f(i:usize)->&'static u8{&[1,2][i]}", true),
    ("fn f()->&'static u8{&[1,2][2]}", true),
    ("fn f()->&'static u8{&([1,2][0]+1)}", true),
    ("fn f()->&'static u8{&([1,2][0+0]+1)}", false),
    ("fn f()->&'static u8{&([1,2][2]+1)}", false),
    ("fn g(){}fn f()->&'static fn(){&(g as fn())}", true),
    ("fn g(){}fn f(){let x:&'static _=&g;}", true),
    ("const C:&u8=&1;fn f()->&'static u8{&(*C+1)}", false),
    ("const C:&u8=&1;fn f()->&'static &'static u8{&(&*C)}", true),
    ("static C:u8=1;fn f()->&'static u8{&(C+1)}", false),
    ("static C:u8=1;fn f()->&'static &'static u8{&&C}", false),
    ("fn g(){}fn f()->&'static (){&{g();}}", true),
    ("const fn g()->u8{1}fn f()->&'static u8{&g()}", false),
    ("struct S(u8);fn f()->&'static S{&S(1)}", true),
    ("fn g(){}struct S(u8);fn f()->&'static S{&S({g();1})}", true),
    (
        "fn g(){}struct S{a:u8}fn f()->&'static S{&S{a:{g();1}}}",
        true,
    ),
    (
        "struct S(u8);fn f(c:bool)->&'static S{&S(if c{1}else{2})}",
        false,
    ),
    ("fn f()->&'static &'static u8{&(&*&1)}", true),
    ("const C:&u8=&1;fn f()->&'static &'static u8{&C}", true),
    ("fn g(){}const C:fn()=g;fn f()->&'static fn(){&C}", true),
    ("fn f()->&'static i8{&((1i8+2)/ -1)}", false),
    ("fn f()->&'static i8{&(3i8/ -1)}", true),
    (
        "struct S{a:u8}fn g()->S{S{a:2}}fn f()->&'static S{&S{a:1,..g()}}",
        true,
    ),
    ("fn f()->&'static u8{&({let x=1;x})}", false),
    ("struct S(u8);fn f()->&'static S{let c=S;&c(1)}", false),
    ("fn g(){}fn f()->&'static fn(){let h=g;&(h as fn())}", false),
    (
        "struct S(u8);fn f()->&'static fn(u8)->S{let c=S;&(c as fn(u8)->S)}",
        false,
    ),
    ("fn g()->u8{7}fn f()->&'static [u8;0]{&[g();0]}", true),
    ("fn f()->&'static [u8;0]{&[]}", true),
    ("fn f()->&'static (){&()}", true),
    ("const C:u8=if true{1}else{2};fn f()->&'static u8{&C}", true),
    (
        "const C:bool=false&&(1u8/0==0);fn f()->&'static bool{&C}",
        true,
    ),
    ("fn f()->&'static f64{&(0.0/0.0)}", true),
    ("fn f()->&'static f32{&(1.0/0.0)}", true),
    ("fn f()->&'static i8{&((1i8+2) % -1)}", false),
    ("fn f()->&'static i8{&(-128i8 / -1)}", false),
    ("fn f()->&'static u8{&(7u8 / (2u16 as u8))}", false),
    ("const C:usize=1-1;fn f()->&'static u8{&([1,2][C]+1)}", true),
    ("fn f()->&'static u8{&((1u8,2u8).0+1)}", true),
    (
        "struct S{a:[u8;2],b:(u8,u8)}fn f()->&'static S{&S{a:[3;2],b:(4,5)}}",
        true,
    ),
    (
        "fn g(){}const C:fn()=g as fn();fn f()->&'static fn(){&C}",
        true,
    ),
    ("const C:&u8=&1;fn f()->&'static u8{&*C}", true),
    ("fn f()->&'static u8{let x=7;&x}", false),
    ("fn f(i:usize)->&'static u8{let a=[1,2];&a[i]}", false),
    ("fn i()->usize{0}fn f()->&'static u8{&[1,2][i()]}", true),
    ("fn f()->&'static u8{&(7u8/(2u8 as u8))}", false),
    ("fn f()->&'static u8{&(7u8/(2 as u8))}", false),
    ("fn g(){}fn f()->&'static u8{&(7u8/{g();2})}", false),
    ("fn f()->&'static i8{&(7i8/(-1))}", true),
    ("fn f()->&'static u8{&([1,2][(0usize as usize)]+1)}", true),
    ("fn f()->&'static u8{&(7u8/(1u8<<1))}", false),
    (
        "struct S(u8);fn g()->S{S(1)}fn f()->&'static [S;0]{&[g();0]}",
        true,
    ),
    ("fn f(p:&u8)->&'static [&'static u8;0]{&[p;0]}", true),
    ("fn f<'a>(p:[&'a u8;0])->[&'static u8;0]{p}", false),
    ("fn f<'a>(p:&'a u8)->[&'static u8;0]{[p;0]}", true),
    ("fn f<'a>(p:&'a u8)->[&'static u8;0]{[]}", true),
    ("fn f<'a>(p:&'a [&'a u8;0])->&'a [&'static u8;0]{p}", false),
    (
        "struct S<'a>(&'a u8);fn f<'a>(p:S<'a>)->&'static [S<'static>;0]{&[p;0]}",
        true,
    ),
    (
        "fn f<'a>(p:fn(&'a u8))->&'static [fn(&'static u8);0]{&[p;0]}",
        true,
    ),
    ("fn f<'a>(p:&'a u8)->[&'static u8;1]{[p;1]}", false),
    ("fn f()->&'static u8{&(7u8/{2})}", false),
    ("fn f()->&'static u8{&([1,2][{0}]+1)}", true),
    ("fn g(){}fn f()->&'static u8{&([1,2][{g();0}]+1)}", true),
    (
        "fn f()->&'static u8{&([1,2][((0usize as usize) as usize)]+1)}",
        true,
    ),
    ("fn f()->&'static u8{&([1,2][({0usize} as usize)]+1)}", true),
    (
        "fn f(p:fn(&'static u8))->[for<'a>fn(&'a u8);0]{[p;0]}",
        false,
    ),
    ("fn f<'a>(p:fn(&'static u8))->[fn(&'a u8);0]{[p;0]}", true),
    (
        "fn f<'a>(p:fn()->&'a u8)->[fn()->&'static u8;0]{[p;0]}",
        true,
    ),
    (
        "fn f(p:for<'a>fn(&'a u8)->&'a u8)->[fn(&'static u8)->&'static u8;0]{[p;0]}",
        true,
    ),
    (
        "fn f(p:(fn(&'static u8),))->[(for<'a>fn(&'a u8),);0]{[p;0]}",
        false,
    ),
    (
        "fn give_static(x:&'static u8)->&'static u8{x}fn f<'a>(p:&'a u8)->[&'static u8;0]{[give_static(p);0]}",
        false,
    ),
    (
        "fn f<'a>(p:&'a u8)->[&'static u8;0]{[{let q:&'static u8=p;q};0]}",
        false,
    ),
    ("fn f<'a>(p:&'a u8)->[&'static u8;0]{[&*p;0]}", true),
    (
        "fn take(_: [&'static u8;0]){}fn f<'a>(p:&'a u8){let a=[p;0];take(a)}",
        true,
    ),
    (
        "fn f<'a>(p:&'a u8)->&'static [&'static u8;0]{let q=[p;0];&q}",
        false,
    ),
];

#[test]
fn promotion_contracts_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/promotion-contracts-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    for (index, (source, accepted)) in CASES.iter().copied().enumerate() {
        let path = directory.join(format!("case-{index}.rs"));
        std::fs::write(&path, source)?;
        let rustc = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-type=lib",
                "--crate-name=probe",
                "--emit=metadata",
                "--cap-lints=allow",
                "-o",
            ])
            .arg(directory.join("probe.rmeta"))
            .arg(path)
            .output()?;
        std::fs::write(
            directory.join(format!("case-{index}.stderr")),
            &rustc.stderr,
        )?;
        assert_eq!(
            rustc.status.code(),
            Some(i32::from(!accepted)),
            "rustc: {source}: {}",
            String::from_utf8_lossy(&rustc.stderr)
        );
        let (report, mut facts) =
            semantic::analyze(source).map_err(|e| format!("{source}: {e}"))?;
        let mut program = body::build(&report, &facts).map_err(|e| format!("{source}: {e}"))?;
        body::promote::analyze(&report, &mut facts, &mut program)
            .map_err(|e| format!("{source}: {e}"))?;
        let contracts = body::regions::collect(&facts)?;
        let result = borrow::regions::check_bodies(
            &facts.types,
            &facts.aggregates,
            &facts.lifetimes.variances,
            &contracts,
            &program,
        );
        assert_eq!(
            result.is_ok(),
            accepted,
            "RustGM: {source}: {:?}",
            result.as_ref().err()
        );
        if let Err(error) = result {
            assert!(
                error
                    .message
                    .contains("required lifetime does not outlive its target"),
                "{source}: {error}"
            );
        }
    }
    Ok(())
}

#[test]
fn promotion_retains_effects_and_checks_and_shares_value_storage()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        body::ir::Rvalue,
        sema::{model::Scalar, values::Evaluator},
    };
    let source = "fn g()->u8{7}fn f()->&'static ([u8;0],u8,[u8;1000000000]){&([g();0],255u8+1,[3;1000000000])}";
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts)?;
    let before: Vec<_> = program
        .bodies
        .iter()
        .flat_map(|b| b.statements.iter())
        .map(|s| format!("{:?}", s.action))
        .collect();
    body::promote::analyze(&report, &mut facts, &mut program)?;
    let after: Vec<_> = program
        .bodies
        .iter()
        .flat_map(|b| b.statements.iter())
        .map(|s| format!("{:?}", s.action))
        .collect();
    assert_eq!(before, after, "promotion must preserve the executable CFG");
    let function = program.bodies.last().ok_or("missing body")?;
    assert_eq!(
        function
            .values
            .iter()
            .filter(|v| matches!(v.kind, Rvalue::Call { .. }))
            .count(),
        1
    );
    assert_eq!(
        function
            .values
            .iter()
            .filter(|v| matches!(v.kind, Rvalue::Binary { .. }))
            .count(),
        1
    );
    let promotions = function
        .promotions
        .as_ref()
        .ok_or("missing promotion facts")?;
    let entries: Vec<_> = promotions.entries().collect();
    assert_eq!(entries.len(), 1);
    let evaluator = Evaluator::new(&report, &mut facts);
    let value = entries[0].1;
    assert_eq!(evaluator.field(value, 1)?.as_scalar()?, Scalar::Integer(0));
    let repeat = evaluator.field(value, 2)?;
    assert_eq!(
        evaluator.field(repeat, 999_999_999)?.as_scalar()?,
        Scalar::Integer(3)
    );
    assert!(matches!(
        evaluator.field(repeat, 1_000_000_000),
        Err(rustgm::sema::values::Failure::Fault(_))
    ));
    Ok(())
}

#[test]
fn constant_faults_are_typed_and_promotion_does_not_hide_invalid_facts()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        lex::token::Punctuation as Op,
        sema::{
            model::Scalar,
            values::{Constant, Evaluator, Failure, Overflow},
        },
        types::{Primitive, TypeId},
    };
    let (report, mut facts) = semantic::analyze("fn f(){}")?;
    let evaluator = Evaluator::new(&report, &mut facts);
    let one = Constant::scalar(Scalar::Integer(1));
    let zero = Constant::scalar(Scalar::Integer(0));
    let max = Constant::scalar(Scalar::Integer(255));
    let ty = TypeId::primitive(Primitive::U8);
    assert!(matches!(
        evaluator.binary(Op::Plus, ty, max, one, Overflow::Checked),
        Err(Failure::Fault(_))
    ));
    assert_eq!(
        evaluator
            .binary(Op::Plus, ty, max, one, Overflow::Wrapping)?
            .as_scalar()?,
        Scalar::Integer(0)
    );
    assert!(matches!(
        evaluator.binary(Op::Slash, ty, one, zero, Overflow::Wrapping),
        Err(Failure::Fault(_))
    ));
    assert!(matches!(
        evaluator.binary(Op::Plus, TypeId::UNIT, one, one, Overflow::Wrapping),
        Err(Failure::Invalid(_))
    ));
    assert!(matches!(evaluator.field(one, 0), Err(Failure::Invalid(_))));
    let (report, mut facts) = semantic::analyze("const C:u8=7/0;fn f()->&'static u8{&C}")?;
    let mut program = body::build(&report, &facts)?;
    let error = body::promote::analyze(&report, &mut facts, &mut program)
        .err()
        .ok_or("invalid required constant was ignored")?;
    assert!(error.message.contains("divides by zero"));
    Ok(())
}

#[test]
fn promotion_queries_are_bounded_iterative_and_cache_shared_subtrees()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{
        body::ir::{Place, Rvalue, Value, ValueId},
        syntax::ast::Unary,
    };
    // Build a typed DAG directly to test query depth independently of parser limits.
    for count in [64_u32, 4096] {
        let (report, mut facts) = semantic::analyze("fn f(){let _=&1u8;}")?;
        let mut program = body::build(&report, &facts)?;
        let function = program.bodies.first_mut().ok_or("missing body")?;
        let reference = function
            .values
            .iter()
            .find(|v| matches!(v.kind, Rvalue::Borrow { .. }))
            .ok_or("missing borrow")?
            .ty;
        let scalar = function.values[0].ty;
        function.values.truncate(1);
        for id in 1..=count {
            function.values.push(Value {
                ty: scalar,
                kind: Rvalue::Unary {
                    operator: Unary::Not,
                    operand: ValueId(id - 1),
                    literal: false,
                },
            });
        }
        for id in 1..=count {
            function.values.push(Value {
                ty: reference,
                kind: Rvalue::Borrow {
                    kind: body::ir::BorrowKind::Immediate,
                    place: Place::Temporary(ValueId(id)),
                },
            });
        }
        body::promote::analyze(&report, &mut facts, &mut program)?;
        let promotions = program.bodies[0]
            .promotions
            .as_ref()
            .ok_or("missing promotions")?;
        assert_eq!(promotions.entries().count(), count as usize);
        assert!(
            promotions.steps < 12 * count as usize,
            "shared query work: {}",
            promotions.steps
        );
        let steps = promotions.steps;
        body::promote::analyze(&report, &mut facts, &mut program)?;
        assert_eq!(
            program.bodies[0]
                .promotions
                .as_ref()
                .ok_or("missing promotions")?
                .steps,
            steps
        );
    }
    let (report, mut facts) = semantic::analyze("fn f(){let _=&1u8;}")?;
    let mut program = body::build(&report, &facts)?;
    program.bodies[0].values[0].kind = Rvalue::Unary {
        operator: Unary::Not,
        operand: ValueId(0),
        literal: false,
    };
    let error = body::promote::analyze(&report, &mut facts, &mut program)
        .err()
        .ok_or("cycle was ignored")?;
    assert!(error.message.contains("cycle in promotion"));
    Ok(())
}

#[test]
fn a_discarded_repeat_still_requires_the_selected_element_type() {
    assert!(semantic::analyze("fn f(p:u8)->[bool;0]{[p;0]}").is_err());
}

#[test]
fn invalid_fixed_projection_does_not_become_a_runtime_only_value()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::{body::ir::ProjectionKind, sema::members::Selection};
    let (report, mut facts) = semantic::analyze("fn f()->&'static u8{&((1u8,2u8).0+1)}")?;
    let mut program = body::build(&report, &facts)?;
    let projection = program.bodies[0]
        .projections
        .iter_mut()
        .find(|p| matches!(p.kind, ProjectionKind::Field(_)))
        .ok_or("missing field")?;
    projection.kind = ProjectionKind::Field(Selection::Tuple(9));
    assert!(body::promote::analyze(&report, &mut facts, &mut program).is_err());
    Ok(())
}
