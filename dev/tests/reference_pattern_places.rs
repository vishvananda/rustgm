use crate::semantic;
use rustgm::{
    body::{
        self,
        ir::{Action, LocalKind, Place, Rvalue},
    },
    borrow,
    sema::patterns::{Capture, Kind as Pattern},
    types::Kind,
};

fn build(source: &str) -> Result<(rustgm::sema::model::Declarations, body::ir::Program), String> {
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
    for body in &program.bodies {
        borrow::storage::check(body).map_err(|e| e.message.to_owned())?;
    }
    Ok((facts, program))
}

#[test]
fn reference_captures_use_selected_places_and_occurrence_types() -> Result<(), String> {
    for source in [
        "fn f(p:u8){let ref x=p;}",
        "fn f(mut p:u8){let ref mut x=p;}",
        "fn f(p:&(u8,u8))->&u8{let(x,y)=p;x}",
        "fn f(p:&mut(u8,u8))->&mut u8{let(x,y)=p;x}",
        "fn f([x,y]:&[u8;2])->&u8{x}",
        "fn f([x,y]:&mut[u8;2])->&mut u8{x}",
        "fn f(p:&[u8;3])->&[u8;1]{let[_,tail@..,_]=p;tail}",
        "fn f(p:&u8)->u8{let &x=p;x}",
        "fn f(p:&mut u8)->u8{let &mut x=p;x}",
        "fn f(p:(u8,u8)){let((ref x,_)|(_,ref x))=p;}",
        "enum E{A(u8),B}fn f(p:&E)->u8{match p{E::A(x)=>*x,E::B=>0}}",
        "enum E{A(u8),B}fn f(p:&E)->u8{let E::A(x)=p else{return 0;};*x}",
    ] {
        let (facts, program) = build(source).map_err(|e| format!("{source}: {e}"))?;
        for (id, ty) in &facts.patterns.borrow_types {
            let node = &facts.patterns.nodes[id.0 as usize];
            let Pattern::Binding {
                capture: Capture::Borrow { mutable, region },
                ..
            } = node.kind
            else {
                return Err("borrow type belongs to a non-borrow capture".into());
            };
            assert!(
                matches!(facts.types.kind(*ty), Some(Kind::Reference {
                mutable: m, region: r, referent
            }) if m==mutable && r==region && Some(referent)==node.ty),
                "{source}"
            );
            assert!(
                program
                    .bodies
                    .iter()
                    .flat_map(|b| &b.values)
                    .any(|v| v.ty == *ty && matches!(v.kind, Rvalue::Borrow { .. })),
                "{source}"
            );
        }
    }
    Ok(())
}

#[test]
fn guarded_reference_captures_view_reference_storage() -> Result<(), String> {
    for source in [
        "fn f(x:u8)->u8{match x{ref r if *r>0=>*r,_=>0}}",
        "fn f(p:&(u8,u8))->u8{match p{(x,y) if *x>0=>*y,_=>0}}",
        "fn f(p:&mut(u8,u8))->u8{match p{(x,y) if *x>0=>*y,_=>0}}",
        "enum E{A(u8),B(u8)}fn f(p:&E)->u8{match p{E::A(x)|E::B(x) if *x>0=>*x,_=>0}}",
        "fn f(p:&(u8,u8))->u8{match p{(x,_)|(_,x) if *x>0=>*x,_=>0}}",
        "fn f(p:(&u8,&[u8;1]))->u8{match p{(x,_)|(_,[x]) if *x>0=>*x,_=>0}}",
        "fn f(p:(&mut u8,&mut[u8;1]))->u8{match p{(x,_)|(_,[x]) if *x>0=>*x,_=>0}}",
    ] {
        let (facts, program) = build(source).map_err(|e| format!("{source}: {e}"))?;
        let body = program
            .bodies
            .iter()
            .find(|b| !b.guards.is_empty())
            .ok_or("missing guarded body")?;
        for guard in &body.guards {
            assert!(
                body.locals[guard.views_start as usize..guard.views_end as usize]
                    .iter()
                    .all(|l| matches!(l.kind, LocalKind::Guard(_)))
            );
        }
        let mut checked = 0;
        for statement in &body.statements {
            if let Action::BindView {
                view,
                place: Place::Local(slot),
            } = statement.action
            {
                let view = &body.locals[view.0 as usize];
                let slot = &body.locals[slot.0 as usize];
                assert_eq!(view.ty, slot.ty, "{source}");
                assert!(matches!(
                    facts.types.kind(slot.ty),
                    Some(Kind::Reference { .. })
                ));
                assert_eq!(slot.kind, LocalKind::Binding);
                assert!(slot.definition.is_none());
                checked += 1;
            }
        }
        assert!(checked > 0, "{source}");
    }
    Ok(())
}

#[test]
fn reference_pattern_lifetime_contracts_match_rustc() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "fn f<'a>(p:(&'static u8,&'a[u8;1]))->&'static u8{let((x,_)|(_,[x]))=p;x}",
            false,
        ),
        (
            "fn f<'a>(p:(&'a u8,&'static[u8;1]))->&'a u8{let((x,_)|(_,[x]))=p;x}",
            true,
        ),
        (
            "fn f(p:&(u8,u8))->&u8{match p{(x,y) if *x>0=>y,_=>loop{}}}",
            true,
        ),
        (
            "fn f(p:(u8,u8))->&'static u8{match p{(ref x,ref y) if *x>0=>y,_=>loop{}}}",
            false,
        ),
        ("fn f(p:&(u8,u8))->&u8{let(x,y)=p;x}", true),
        ("fn f(p:&mut(u8,u8))->&mut u8{let(x,y)=p;x}", true),
        ("fn f<'a>(p:(u8,u8))->&'a u8{let(ref x,_)=p;x}", false),
        ("fn f<'a>(p:&'a(u8,u8))->&'static u8{let(x,_)=p;x}", false),
        ("fn f(p:&[u8;3])->&[u8;1]{let[_,tail@..,_]=p;tail}", true),
        ("fn f<'a>(p:&'a(u8,u8))->&'a u8{let((x,_)|(_,x))=p;x}", true),
        (
            "fn f<'a>(p:(u8,u8))->&'a u8{let((ref x,_)|(_,ref x))=p;x}",
            false,
        ),
        (
            "enum E{A(u8),B}fn f(p:&E)->&u8{match p{E::A(x)=>x,E::B=>loop{}}}",
            true,
        ),
        (
            "enum E{A(u8),B}fn f(p:E)->&'static u8{match p{E::A(ref x)=>x,E::B=>loop{}}}",
            false,
        ),
    ];
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/reference-pattern-places-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory)?;
    for (index, (source, accepted)) in cases.into_iter().enumerate() {
        let path = directory.join(format!("case-{index}.rs"));
        std::fs::write(&path, source)?;
        let output = std::process::Command::new("rustc")
            .args([
                "+1.98.0",
                "--edition=2024",
                "--crate-name=probe",
                "--crate-type=lib",
                "--emit=metadata",
                "--cap-lints=allow",
                "-o",
            ])
            .arg(directory.join("probe.rmeta"))
            .arg(path)
            .output()?;
        std::fs::write(
            directory.join(format!("case-{index}.stderr")),
            &output.stderr,
        )?;
        assert_eq!(
            output.status.code(),
            Some(i32::from(!accepted)),
            "rustc {source}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let (facts, program) = build(source)?;
        let contracts = body::regions::collect(&facts)?;
        let checked = borrow::regions::check_bodies(
            &facts.types,
            &facts.lifetimes.variances,
            &contracts,
            &program,
        );
        assert_eq!(
            checked.is_ok(),
            accepted,
            "RustGM {source}: {:?}",
            checked.err()
        );
    }
    Ok(())
}
