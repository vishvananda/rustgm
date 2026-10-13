#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body::{
        self,
        ir::{BorrowKind, LocalKind, Place, Rvalue},
    },
    borrow,
};

fn build(source: &str) -> Result<body::ir::Program, String> {
    let (report, facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    for body in &program.bodies {
        borrow::storage::check(body).map_err(|e| e.message.to_owned())?;
    }
    Ok(program)
}

#[test]
fn mutable_call_arguments_reborrow_without_moving_the_pointer() -> Result<(), String> {
    let program = build("fn take(_: &mut u8){}fn f(p:&mut u8){take(p);take(p);*p=4;}")?;
    let body = program.bodies.last().ok_or("missing f")?;
    assert_eq!(
        body.values
            .iter()
            .filter(|v| matches!(
                v.kind,
                Rvalue::Borrow {
                    kind: BorrowKind::TwoPhase,
                    ..
                }
            ))
            .count(),
        2
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
fn two_phase_argument_borrows_do_not_extend_to_block_tails()
-> Result<(), Box<dyn std::error::Error>> {
    for (index, (expression, two_phase)) in [
        ("p", true),
        ("(p)", true),
        ("{p}", false),
        ("if c{p}else{p}", false),
        ("match c{true=>p,false=>p}", false),
        ("'b:{break 'b p;}", false),
        ("loop{break p;}", false),
        ("&mut *p", false),
    ]
    .into_iter()
    .enumerate()
    {
        let source =
            format!("fn take(_: &mut u8,_:u8){{}}fn f(p:&mut u8,c:bool){{take({expression},*p);}}");
        check_rustc(&source, "two-phase", index, two_phase)?;
        let program = build(&source)?;
        let body = program.bodies.last().ok_or("missing f")?;
        assert_eq!(
            body.values.iter().any(|v| matches!(
                v.kind,
                Rvalue::Borrow {
                    kind: BorrowKind::TwoPhase,
                    ..
                }
            )),
            two_phase,
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn an_inferred_binding_moves_while_an_annotated_binding_reborrows() -> Result<(), String> {
    for (text, borrows, reads) in [
        ("let q=p;", 0, 1),
        ("let q:&mut u8=p;", 1, 0),
        ("let q:&u8=p;", 1, 0),
    ] {
        let program = build(&format!("fn f(p:&mut u8){{{text}}}"))?;
        let body = program.bodies.first().ok_or("missing f")?;
        let parameter = body
            .locals
            .iter()
            .position(|l| l.kind == LocalKind::Parameter(0))
            .ok_or("missing parameter")?;
        assert_eq!(
            body.values
                .iter()
                .filter(|v| matches!(v.kind, Rvalue::Borrow { .. }))
                .count(),
            borrows,
            "{text}"
        );
        assert_eq!(
            body.values
                .iter()
                .filter(
                    |v| matches!(v.kind,Rvalue::Read(Place::Local(id)) if id.0 as usize==parameter)
                )
                .count(),
            reads,
            "{text}"
        );
    }
    Ok(())
}

#[test]
fn coercion_sites_preserve_rustc_moves_and_reborrows() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("let r=[p];", 1, 0),
        ("let r=(p,);", 1, 0),
        ("let r=(p);", 1, 0),
        ("let r={p};", 1, 0),
        ("let r=[p;0];", 1, 0),
        ("let r=[p;1];", 1, 0),
        ("let r=if c{p}else{p};", 2, 0),
        ("let r=match c{true=>p,false=>p};", 1, 1),
        ("let r=loop{break p;};", 1, 0),
        ("let r='b:{break 'b p;};", 1, 0),
        ("let r=[p,q];", 0, 1),
        ("let r=if c{p}else{q};", 1, 1),
        ("let r=match c{true=>p,false=>q};", 0, 1),
        ("let r:&mut u8=(p);", 0, 1),
        ("let r:&mut u8={p};", 0, 1),
        ("let r:[&mut u8;1]=[p];", 0, 1),
        ("let r:(&mut u8,)=(p,);", 0, 1),
        ("let r:[&mut u8;0]=[p;0];", 0, 1),
        ("let r:&mut u8=if c{p}else{p};", 0, 2),
        ("let r:&mut u8=match c{true=>p,false=>p};", 0, 2),
        ("let r:&mut u8=loop{break p;};", 0, 1),
        ("let r:&mut u8='b:{break 'b p;};", 0, 1),
        ("let r:&mut u8='b:{if c{break 'b p;}p};", 0, 2),
        ("let r:&u8=if c{{p}}else{return;};", 0, 1),
        ("macro_rules! v{()=>{{p}}}let r:&u8=v!();", 0, 1),
    ];
    for (index, (statement, reads, borrows)) in cases.into_iter().enumerate() {
        let source = format!("fn f(p:&mut u8,q:&u8,c:bool){{{statement}*p=3;}}");
        check_rustc(&source, "moves", index, reads == 0)?;
        // The loan checker is not released yet. Verify its input operations
        // independently of rustc's expected whole-program acceptance above.
        let program = build(&source).map_err(|e| format!("{source}: {e}"))?;
        let body = program.bodies.first().ok_or("missing f")?;
        let parameter = body
            .locals
            .iter()
            .position(|l| l.kind == LocalKind::Parameter(0))
            .ok_or("missing p")?;
        assert_eq!(
            body.values
                .iter()
                .filter(|v| matches!(v.kind,
            Rvalue::Read(Place::Local(id)) if id.0 as usize == parameter))
                .count(),
            reads,
            "{source}"
        );
        assert_eq!(
            body.values
                .iter()
                .filter(|v| matches!(v.kind, Rvalue::Borrow { .. }))
                .count(),
            borrows,
            "{source}"
        );
    }
    Ok(())
}

#[test]
fn later_reference_sites_use_the_earlier_type_hint() -> Result<(), Box<dyn std::error::Error>> {
    for (index, (expression, q_moves)) in [
        ("[p,q]", false),
        ("[p,{q}]", false),
        ("[{p},q]", false),
        ("if c{p}else{q}", true),
        ("match c{true=>p,false=>q}", false),
        ("match c{true=>p,false=>{q}}", true),
        ("loop{if c{break p;}break q;}", false),
        ("loop{if c{break p;}break {q};}", false),
        ("'b:{if c{break 'b p;}q}", false),
    ]
    .into_iter()
    .enumerate()
    {
        let source = format!("fn f(p:&mut u8,q:&mut u8,c:bool){{let r={expression};*q=3;}}");
        check_rustc(&source, "later-sites", index, !q_moves)?;
        let program = build(&source)?;
        let body = program.bodies.first().ok_or("missing f")?;
        for (parameter, moves) in [(0, true), (1, q_moves)] {
            let local = body
                .locals
                .iter()
                .position(|l| l.kind == LocalKind::Parameter(parameter))
                .ok_or("missing parameter")?;
            assert_eq!(
                body.values
                    .iter()
                    .filter(|v| matches!(v.kind,
                Rvalue::Read(Place::Local(id)) if id.0 as usize==local))
                    .count(),
                usize::from(moves),
                "parameter {parameter}: {source}"
            );
        }
    }
    Ok(())
}

#[test]
fn adjusted_rvalues_preserve_effects_and_balance_diverging_paths() -> Result<(), String> {
    for source in [
        "fn p()->&'static mut u8{loop{}}fn take(_: &u8){}fn f(){take(p());}",
        "fn take(_: &mut u8){}fn f(p:&mut [u8;2]){take(&mut p[0]);}",
        "struct S{p:&'static mut u8}fn take(_: &u8){}fn f(mut s:S){take(s.p);}",
        "fn take(_: &u8){}fn f(){take({return;loop{}});}",
        "fn take(_: &mut u8){}fn f(p:&mut u8,c:bool){take(if c{p}else{return;});}",
        "fn take(_: &u8){}fn f(p:&mut u8){take({p});}",
        "fn take(_: &u8){}fn f(p:&mut [u8;2]){take(&mut p[{return;0}]);}",
        "fn f(p:&mut [u8;2]){let q:&mut [u8]=p;}",
        "fn f(p:&mut [u8;2]){let q:&[u8]=p;}",
    ] {
        build(source).map_err(|e| format!("{source}: {e}"))?;
    }
    let program =
        build("fn effect(){}fn get()->&'static mut u8{loop{}}fn f(){let q:&u8={effect();get()};}")?;
    let body = program.bodies.last().ok_or("missing f")?;
    assert_eq!(
        body.values
            .iter()
            .filter(|v| matches!(v.kind, Rvalue::Call { .. }))
            .count(),
        2
    );
    Ok(())
}

#[test]
fn reference_coercion_contracts_match_pinned_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("fn f<'a>(p:&'a mut u8)->[&'static u8;0]{[{p};0]}", true),
        ("fn f<'a>(p:&'a mut u8)->[&'static u8;1]{[{p};1]}", false),
        (
            "fn f(p:&mut u8,q:&u8,c:bool){let r=loop{if c{break p;}break q;};}",
            false,
        ),
        (
            "fn f(p:&mut u8,q:&u8,c:bool){let r='b:{if c{break 'b p;}q};}",
            false,
        ),
        ("fn f(p:&mut u8,q:&u8,c:bool){let r=if c{p}else{q};}", true),
        ("fn f(p:&&u8,q:&u8,c:bool){let r=if c{p}else{q};}", true),
        (
            "fn f(p:&&mut u8,q:&mut u8,c:bool){let r=if c{p}else{q};}",
            false,
        ),
        ("fn f(p:&mut &u8,q:&u8,c:bool){let r=if c{p}else{q};}", true),
        ("fn f(p:&mut u8,q:&u8){let r=[p,q];}", true),
        ("fn f(p:&u8,q:&mut u8){let r=[p,q];}", true),
        ("fn f()->&'static mut [u8;0]{&mut []}", true),
        ("fn f()->&'static mut u8{&mut 7}", false),
        ("fn f()->&'static u8{&mut 7}", false),
        ("fn f<'a>(p:&'a mut u8)->&'a mut u8{p}", true),
        ("fn f<'a>(p:&'a mut u8)->&'a u8{p}", true),
        ("fn f<'a>(p:&'a mut u8)->&'a mut u8{&mut *p}", true),
        ("fn f<'a>(p:&'a mut u8)->&'static u8{p}", false),
        ("fn f<'a>(p:&'a u8)->&'a mut u8{p}", false),
        ("fn f<'a,'b>(p:&'a &'b u8)->&'b u8{p}", true),
        ("fn f<'a,'b>(p:&'a &'b mut u8)->&'b u8{p}", false),
        ("fn f<'a,'b>(p:&'a &'b mut u8)->&'a u8{p}", true),
        ("fn f<'a,'b>(p:&'a mut &'b u8)->&'b u8{p}", true),
        ("fn f<'a,'b>(p:&'a mut &'b mut u8)->&'b mut u8{p}", false),
        ("fn f<'a,'b>(p:&'a mut &'b mut u8)->&'a mut u8{p}", true),
        ("fn f<'a,'b>(p:&'a &'b mut u8)->&'a mut u8{p}", false),
        ("fn f<'a>(p:&'a mut [u8;2])->&'a [u8]{p}", true),
        ("fn f<'a>(p:&'a mut [u8;2])->&'a mut [u8]{p}", true),
        ("fn f<'a>(p:&'a mut u8)->&'a u16{p}", false),
        ("fn f<'a>(p:&'a mut &'static u8)->&'a &'a u8{p}", true),
        ("fn f<'a>(p:&'a mut &'static u8)->&'a mut &'a u8{p}", false),
        ("fn f<'a>(p:&'a mut u8){let q:&mut _=p;}", true),
        ("fn f<'a>(p:&'a mut &'a u8){let q:&mut _=p;}", true),
    ];
    for (i, (source, accepted)) in cases.into_iter().enumerate() {
        check_rustc(source, "contracts", i, accepted)?;
        let checked = (|| -> Result<(), String> {
            let (report, mut facts) = semantic::analyze(source)?;
            let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
            body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
            let contracts = body::regions::collect(&facts).map_err(str::to_owned)?;
            borrow::regions::check_bodies(
                &facts.types,
                &facts.aggregates,
                &facts.lifetimes.variances,
                &contracts,
                &program,
            )
            .map_err(|e| e.to_string())?;
            Ok(())
        })();
        assert_eq!(checked.is_ok(), accepted, "RustGM {source}: {checked:?}");
    }
    Ok(())
}

#[test]
fn reference_coercion_propagation_is_iterative() -> Result<(), String> {
    let depth = 96;
    let source = format!(
        "fn f(p:&mut u8)->&mut u8{{{}p{}}}",
        "{".repeat(depth),
        "}".repeat(depth)
    );
    let program = build(&source)?;
    assert_eq!(
        program.bodies[0]
            .values
            .iter()
            .filter(|v| matches!(v.kind, Rvalue::Borrow { .. }))
            .count(),
        1
    );
    assert!(
        !program.bodies[0]
            .values
            .iter()
            .any(|v| matches!(v.kind, Rvalue::Read(Place::Local(_))))
    );
    Ok(())
}

fn check_rustc(
    source: &str,
    group: &str,
    index: usize,
    accepted: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/reborrow-{group}-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let path = directory.join(format!("case-{index}.rs"));
    std::fs::write(&path, source)?;
    let result = std::process::Command::new("rustc")
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
        &result.stderr,
    )?;
    assert_eq!(
        result.status.code(),
        Some(i32::from(!accepted)),
        "rustc {source}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}
