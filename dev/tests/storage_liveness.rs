use crate::semantic;
use rustgm::{
    body::{
        self,
        ir::{Body, Program},
        scopes,
        storage::{End, Location},
    },
    borrow,
};
use std::fmt::Write as _;

fn build(source: &str) -> Result<Program, String> {
    let (report, facts) = semantic::analyze(source)?;
    body::build(&report, &facts).map_err(|e| e.to_string())
}
fn verify(program: &Program) -> Result<(), String> {
    for body in &program.bodies {
        borrow::storage::check(body).map_err(|e| format!("{:?}: {}", e.node, e.message))?;
    }
    Ok(())
}
fn final_function(program: &mut Program) -> Result<&mut Body, String> {
    program.bodies.last_mut().ok_or("missing function".into())
}

#[test]
fn storage_follows_branches_loops_guards_and_abrupt_exits() -> Result<(), String> {
    let prefix = "fn temp()->u8{7}fn see(_: &u8)->bool{true}";
    let bodies = [
        "let x=&temp();see(x);",
        "let x=&{temp()};see(x);",
        "let x={&temp()};see(x);",
        "if see(&temp()){see(&temp());}else{see(&temp());}",
        "if let x=&temp()&&see(x){see(x);}else{see(&temp());}",
        "let p=&temp();while see(p){see(&temp());break;}",
        "while let x=&temp(){if see(x){continue;}break;}",
        "loop{let p=&temp();if see(p){break;}else{continue;}}",
        "'outer:loop{loop{see(&temp());break 'outer;}}",
        "'done:{let p=&temp();if see(p){break 'done;}see(p);}",
        "match temp(){0=>{see(&temp());},x if see(&x)=>{},_=>{}}",
        "match temp(){0|0 if see(&temp())=>{},x if see(&x)=>{},_=>{}}",
        "let p=&temp();let 0=temp() else{see(p);return;};see(p);",
        "let x=if see(&temp()){&temp()}else{&temp()};see(x);",
        "let x=match temp(){0=>&temp(),_=>&temp()};see(x);",
        "if see(&temp()){return;}see(&temp());",
        "let _=see(&temp())&&{see(&temp())};",
        "let _=see(&temp())||{see(&temp())};",
    ];
    for text in bodies {
        let source = format!("{prefix}fn f(){{{text}}}");
        verify(&build(&source)?).map_err(|e| format!("{source}: {e}"))?;
    }
    Ok(())
}

#[test]
fn if_let_failure_ends_on_the_selected_edge_before_else() -> Result<(), String> {
    let program = build("fn temp()->u8{7}fn f(){if let 0=temp(){}else{let x=1;}}")?;
    verify(&program)?;
    let body = program.bodies.last().ok_or("missing f")?;
    let facts = body.storage.as_ref().ok_or("missing storage")?;
    let conditional = facts
        .scopes
        .iter()
        .position(|s| s.kind == scopes::Kind::IfThen)
        .ok_or("missing if scope")?;
    let ends = facts
        .exits
        .iter()
        .filter(|e| matches!(e.end, End::Scopes(r) if r.inner.0 as usize == conditional))
        .count();
    assert_eq!(
        ends, 2,
        "success and failure must each end if-let temporaries"
    );
    Ok(())
}

#[test]
fn unavailable_temporary_uses_and_missing_cleanup_are_rejected() -> Result<(), String> {
    let mut program = build("fn temp()->u8{7}fn f(){let x=&temp();}")?;
    verify(&program)?;
    let body = final_function(&mut program)?;
    let temporary = body.temporaries.first().ok_or("missing temporary")?;
    let location = Location::At {
        block: temporary.block,
        before: temporary.before,
    };
    body.storage
        .as_mut()
        .ok_or("missing storage")?
        .exits
        .push(body::storage::Exit {
            location,
            end: End::Temporaries { start: 0, end: 1 },
        });
    // The injected endpoint is not in builder construction order, so this
    // hand-edited metadata goes through the validating entry point.
    let error = borrow::storage::validate(body)
        .err()
        .ok_or("unavailable borrow accepted")?;
    assert_eq!(
        error.message,
        "temporary place is outside its storage lifetime"
    );
    body.storage
        .as_mut()
        .ok_or("missing storage")?
        .exits
        .clear();
    // Removing all endpoints must also update the materialization event order.
    for temporary in &mut body.temporaries {
        temporary.exits_before = 0;
    }
    let error = borrow::storage::validate(body)
        .err()
        .ok_or("missing cleanup accepted")?;
    assert_eq!(error.message, "temporary storage remains live at return");
    Ok(())
}

#[test]
fn let_else_failure_closes_only_its_initializer() -> Result<(), String> {
    let program =
        build("fn temp()->u8{7}fn f(){let p=&temp();let 0=temp() else{return;};let _=p;}")?;
    verify(&program)?;
    let body = program.bodies.last().ok_or("missing f")?;
    let facts = body.storage.as_ref().ok_or("missing storage")?;
    let ranges = facts
        .exits
        .iter()
        .filter_map(|e| match e.end {
            End::Temporaries { start, end } => Some((start, end)),
            End::Scopes(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(ranges, [(1, 2)]);
    Ok(())
}

#[test]
fn configured_parameters_and_constructor_bodies_have_scope_facts() -> Result<(), String> {
    verify(&build(
        "struct S(u8);fn f(#[cfg(any())] missing:bool,#[cfg(all())] x:u8)->S{S(x)}",
    )?)?;
    Ok(())
}

#[test]
fn failure_edges_remove_condition_storage_before_else_entry() -> Result<(), String> {
    use body::ir::Terminator;
    let program =
        build("fn temp()->u8{7}fn see(_: &u8){}fn f(c:bool){if let p=&temp()&&c{see(p);}else{}}")?;
    let body = program.bodies.last().ok_or("missing f")?;
    let analysis = borrow::storage::check(body).map_err(|e| e.message.to_owned())?;
    let temporary = body
        .temporaries
        .first()
        .ok_or("missing condition temporary")?;
    let scopes::Lifetime::Scope(owner) = temporary.lifetime else {
        return Err("unexpected static".into());
    };
    let storage = body.storage.as_ref().ok_or("missing storage")?;
    let mut failures = 0;
    for exit in &storage.exits {
        let (Location::At { block, before }, End::Scopes(range)) = (exit.location, exit.end) else {
            continue;
        };
        if range.inner != owner || before != body.blocks[block.0 as usize].start {
            continue;
        }
        let Terminator::Jump(target) = body.blocks[block.0 as usize].terminator else {
            return Err("failure does not jump to else".into());
        };
        if analysis.reachable[block.0 as usize] {
            assert_eq!(analysis.live_on_entry(block, temporary.value), Some(true));
            assert_eq!(analysis.live_on_entry(target, temporary.value), Some(false));
            failures += 1;
        }
    }
    assert!(failures > 0);
    Ok(())
}

#[test]
fn let_else_failure_also_ends_lifetime_extended_initializer_operands() -> Result<(), String> {
    let program = build(
        "enum E<'a>{A(&'a u8),B}fn temp()->u8{7}fn see(_: &u8){}fn f(){let p=&temp();let E::B=E::A(&temp()) else{see(p);return;};}",
    )?;
    verify(&program)?;
    // Synthetic constructor bodies follow source functions in Program.
    let body = program
        .bodies
        .iter()
        .find(|b| b.temporaries.len() == 3)
        .ok_or("missing let-else body")?;
    let facts = body.storage.as_ref().ok_or("missing storage")?;
    assert_eq!(body.temporaries[0].lifetime, body.temporaries[1].lifetime);
    let scopes::Lifetime::Scope(scope) = body.temporaries[1].lifetime else {
        return Err("unexpected static".into());
    };
    assert_eq!(facts.scopes[scope.0 as usize].kind, scopes::Kind::Block);
    assert!(
        facts
            .exits
            .iter()
            .any(|e| matches!(e.end, End::Temporaries { start: 1, end: 3 }))
    );
    Ok(())
}

#[test]
fn invalid_scope_and_edge_metadata_is_diagnosed() -> Result<(), String> {
    use body::{
        ir::BlockId,
        storage::{Exit, Range},
    };
    let mut program = build("fn f(){let x=&1u8;}")?;
    let body = final_function(&mut program)?;
    body.storage
        .as_mut()
        .ok_or("missing storage")?
        .exits
        .push(Exit {
            location: Location::Edge {
                block: BlockId(0),
                target: BlockId(0),
            },
            end: End::Scopes(Range {
                inner: scopes::Id(0),
                outer: None,
            }),
        });
    assert_eq!(
        borrow::storage::validate(body)
            .err()
            .ok_or("invalid edge accepted")?
            .message,
        "storage exit has no matching CFG edge"
    );
    body.storage.as_mut().ok_or("missing storage")?.exits.pop();
    body.storage.as_mut().ok_or("missing storage")?.exits[0].end = End::Scopes(Range {
        inner: scopes::Id(u32::MAX),
        outer: Some(scopes::Id(u32::MAX)),
    });
    assert_eq!(
        borrow::storage::validate(body)
            .err()
            .ok_or("invalid range accepted")?
            .message,
        "invalid storage exit scope"
    );
    let (report, mut facts) = semantic::analyze("fn f(){let x=&1u8;}")?;
    let mut program = body::build(&report, &facts).map_err(|e| e.to_string())?;
    body::promote::analyze(&report, &mut facts, &mut program).map_err(|e| e.to_string())?;
    verify(&program)?;
    let body = final_function(&mut program)?;
    body.temporaries[0].lifetime = scopes::Lifetime::Scope(scopes::Id(u32::MAX));
    assert_eq!(
        borrow::storage::validate(body)
            .err()
            .ok_or("invalid promoted scope accepted")?
            .message,
        "invalid temporary scope"
    );
    Ok(())
}

#[test]
fn a_loop_cannot_reenter_a_still_live_temporary_instance() -> Result<(), String> {
    use body::ir::Terminator;
    let mut program =
        build("fn temp()->u8{7}fn f(c:bool){loop{let p=&temp();if c{continue;}continue;}}")?;
    let body = final_function(&mut program)?;
    let analysis = borrow::storage::check(body).map_err(|e| e.message.to_owned())?;
    let Terminator::Jump(header) = body.blocks[0].terminator else {
        return Err("missing loop entry".into());
    };
    let value = body.temporaries.first().ok_or("missing temporary")?.value;
    assert_eq!(analysis.maybe_live_on_entry(header, value), Some(false));
    body.storage.as_mut().ok_or("missing storage")?.exits.retain(|e| {
        !matches!(e.location, Location::At { block, .. } if matches!(body.blocks[block.0 as usize].terminator, Terminator::Jump(target) if target == header))
    });
    for temporary in &mut body.temporaries {
        temporary.exits_before = 0;
    }
    assert_eq!(
        borrow::storage::validate(body)
            .err()
            .ok_or("reentry accepted")?
            .message,
        "temporary storage is re-entered before its prior lifetime ends"
    );
    Ok(())
}

#[test]
fn many_early_returns_share_scope_masks_and_bounded_storage_state() -> Result<(), String> {
    for count in [128_usize, 512, 2048] {
        let mut source = String::from("fn temp()->u8{7}fn f(c:bool){");
        for i in 0..count {
            write!(source, "let p{i}=&temp();if c{{return;}}")
                .map_err(|error| error.to_string())?;
        }
        source.push('}');
        let program = build(&source)?;
        let body = program.bodies.last().ok_or("missing f")?;
        let facts = body.storage.as_ref().ok_or("missing storage")?;
        assert!(facts.exits.len() < count * 7 + 5);
        let analysis = borrow::storage::check(body).map_err(|e| e.message.to_owned())?;
        assert!(
            analysis.steps < 20 * body.blocks.len() * count.div_ceil(64) + 1000 * count,
            "{} steps for {count} temporaries",
            analysis.steps
        );
    }
    Ok(())
}

#[test]
fn rustc_drops_failed_extended_initializers_before_else() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/storage-let-else-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let source = "struct D(u8);impl Drop for D{fn drop(&mut self){println!(\"d{}\",self.0);}}enum E<'a>{A(&'a D),B}fn temp(n:u8)->D{D(n)}fn see(_: &D){}fn f(){let p=&temp(1);let E::B=E::A(&temp(2)) else{println!(\"else\");see(p);return;};}fn main(){f();}";
    let path = directory.join("probe.rs");
    let executable = directory.join("probe");
    std::fs::write(&path, source)?;
    let compile = std::process::Command::new("rustc")
        .args([
            "+1.98.0",
            "--edition=2024",
            "--cap-lints=allow",
            "-Cpanic=abort",
            "-o",
        ])
        .arg(&executable)
        .arg(path)
        .output()?;
    std::fs::write(directory.join("compile.stderr"), &compile.stderr)?;
    assert_eq!(
        compile.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&compile.stderr)
    );
    let result = std::process::Command::new(executable).output()?;
    std::fs::write(directory.join("stdout"), &result.stdout)?;
    assert_eq!(result.status.code(), Some(0));
    assert!(result.stderr.is_empty());
    assert_eq!(result.stdout, b"d2\nelse\nd1\n");
    Ok(())
}
