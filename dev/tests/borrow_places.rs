#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body::{
        self,
        ir::{Action, Place, ProjectionKind, Rvalue},
    },
    types::Kind,
};

#[test]
fn borrows_preserve_local_referent_and_projected_storage_without_moves() -> Result<(), String> {
    for (source, dereferences, borrowed_local) in [
        ("fn f(p:&u8){let r=&p;}", 0, true),
        ("fn f(p:&u8)->&u8{&*p}", 1, false),
        ("struct S(u8);fn f(p:&S)->&u8{&p.0}", 1, false),
        ("struct S(u8);fn f<'a>(p:&'a &'a S)->&'a u8{&p.0}", 2, false),
        ("fn f(p:&[u8;2])->&u8{&p[0]}", 1, false),
        ("fn f<'a>(p:&'a &'a u8)->&'a u8{&**p}", 2, false),
    ] {
        let (report, facts) = semantic::analyze(source).map_err(|e| format!("{source}: {e}"))?;
        let program = body::build(&report, &facts).map_err(|e| e.to_string())?;
        let body = program.bodies.first().ok_or("missing body")?;
        assert_eq!(
            body.projections
                .iter()
                .filter(|p| matches!(p.kind, ProjectionKind::Dereference { .. }))
                .count(),
            dereferences,
            "{source}"
        );
        let borrows: Vec<_> = body
            .values
            .iter()
            .filter_map(|v| match v.kind {
                Rvalue::Borrow { place, .. } => Some((place, v.ty)),
                _ => None,
            })
            .collect();
        assert_eq!(borrows.len(), 1, "{source}");
        assert_eq!(
            matches!(borrows[0].0, Place::Local(_)),
            borrowed_local,
            "{source}"
        );
        assert!(matches!(
            facts.types.kind(borrows[0].1),
            Some(Kind::Reference { .. })
        ));
        assert!(
            !body
                .values
                .iter()
                .any(|v| matches!(v.kind, Rvalue::Read(Place::Projection(_)))),
            "a borrow must not read its referent: {source}"
        );
    }
    Ok(())
}

#[test]
fn borrow_place_evaluation_runs_effects_once_and_balances_diverging_operands() -> Result<(), String>
{
    for (source, calls, borrows) in [
        (
            "fn p()->&'static [u8;2]{loop{}}fn i()->usize{0}fn f()->&'static u8{&p()[i()]}",
            2,
            1,
        ),
        ("fn f(){let _=&(loop{});}", 0, 0),
        ("fn f(p:&u8)->&u8{loop{};&*p}", 0, 0),
        ("fn f(p:&u8,c:bool){if c {let r=&*p;}else{let r=&p;}}", 0, 2),
        ("fn f(p:&u8){if false {let r=&*p;}}", 0, 1),
    ] {
        let (report, facts) = semantic::analyze(source).map_err(|e| format!("{source}: {e}"))?;
        let program = body::build(&report, &facts).map_err(|e| format!("{source}: {e}"))?;
        let body = program.bodies.last().ok_or("missing body")?;
        assert_eq!(
            body.values
                .iter()
                .filter(|v| matches!(v.kind, Rvalue::Call { .. }))
                .count(),
            calls,
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
        for (index, _) in body.values.iter().enumerate() {
            assert_eq!(
                body.statements
                    .iter()
                    .filter(|s| matches!(s.action,Action::Evaluate(id) if id.0 as usize == index))
                    .count(),
                1,
                "each expression has one CFG evaluation: {source}"
            );
        }
    }
    Ok(())
}

#[test]
fn borrow_contracts_distinguish_storage_from_referents_and_shared_from_exclusive_chains()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("fn f<'a>(p:&'a u8)->&'a u8{&*p}", true),
        ("fn f<'a>(p:u8)->&'a u8{&p}", false),
        ("fn f<'a>(p:&'a u8)->&'a &'a u8{&p}", false),
        ("fn f<'a>(p:&'a u8)->&'a u8{let q=p;&*q}", true),
        ("fn f<'a,'b>(p:&'a &'b u8)->&'b u8{&**p}", true),
        ("fn f<'a,'b>(p:&'a &'b mut u8)->&'b u8{&**p}", false),
        ("fn f<'a,'b>(p:&'a &'b mut u8)->&'a u8{&**p}", true),
        ("fn f<'a,'b>(p:&'a mut &'b u8)->&'b u8{&**p}", true),
        ("fn f<'a,'b>(p:&'a mut &'b mut u8)->&'b u8{&**p}", false),
        ("fn f<'a,'b>(p:&'a mut &'b mut u8)->&'a u8{&**p}", true),
        ("struct S(u8);fn f<'a>(p:&'a S)->&'a u8{&p.0}", true),
        ("struct S(u8);fn f<'a>(p:S)->&'a u8{&p.0}", false),
        ("fn f<'a>(p:&'a [u8;2])->&'a u8{&p[0]}", true),
        ("fn f<'a>(p:[u8;2])->&'a u8{&p[0]}", false),
        ("fn f()->&'static u8{loop{};let x=1;&x}", true),
        (
            "fn f<'a>(p:&'a u8)->&'static u8{if false{&*p}else{loop{}}}",
            false,
        ),
        ("static X:u8=7;fn f()->&'static u8{&X}", true),
        ("fn f(){let _=&(loop{});}", true),
    ];
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../obj/borrow-place-contracts-{}",
        std::process::id()
    ));
    std::fs::create_dir_all(&directory)?;
    for (index, (source, accepted)) in cases.iter().copied().enumerate() {
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
        let (report, facts) = semantic::analyze(source).map_err(|e| format!("{source}: {e}"))?;
        let program = body::build(&report, &facts).map_err(|e| format!("{source}: {e}"))?;
        let contracts = body::regions::collect(&facts)?;
        let result = rustgm::borrow::regions::check_bodies(
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
