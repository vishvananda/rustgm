use rustgm::{
    body::{
        self,
        ir::{Action, Body, LocalKind, Owner, Place, Rvalue},
        storage::Location,
    },
    borrow::ownership::initialization::{self, Cause, Event, Status},
};
#[path = "helpers/qualify.rs"]
mod rustc;
#[path = "support/semantic.rs"]
mod semantic;

const PREFIX: &str =
    "fn trace(_:u8){}struct D(u8);impl Drop for D{fn drop(&mut self){trace(self.0);}}";

struct Observed {
    label: String,
    cause: Cause,
    status: Status,
    location: Location,
}
fn inspect(source: &str, function: &str) -> Result<Vec<Observed>, Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let (index, cfg) = program.bodies.iter().enumerate().find(|(_, b)| {
        matches!(b.owner, Owner::Function { definition, .. }
            if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == function)
    }).ok_or("missing function body")?;
    let needs = &program
        .drops
        .as_ref()
        .ok_or("missing pending drops")?
        .bodies[index];
    let init = initialization::analyze(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        cfg,
    )
    .map_err(|e| e.message)?;
    let mut observed = Vec::new();
    init.visit_events(|event, point| {
        let Event::Cleanup(candidate) = event else {
            return Ok(());
        };
        let (needed, label) = match candidate.place {
            Place::Local(id) => {
                let local = &cfg.locals[id.0 as usize];
                let label = if let Some(def) = local.definition {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[def.0 as usize].name)
                        .to_owned()
                } else if let LocalKind::Parameter(index) = local.kind {
                    format!("parameter:{index}")
                } else {
                    "merge".to_owned()
                };
                (needs.locals[id.0 as usize], label)
            }
            Place::Temporary(id) => (needs.values[id.0 as usize], "temporary".to_owned()),
            Place::Projection(id) => (needs.projections[id.0 as usize], "projection".to_owned()),
            Place::Static(_) => return Err("unexpected static cleanup"),
        };
        if !needed {
            return Ok(());
        }
        let status = if let Some(path) = candidate.path {
            point.status(&init.ownership.paths.entries[path.0 as usize])?
        } else {
            Status::Always
        };
        if status != Status::Never {
            observed.push(Observed {
                label,
                cause: candidate.cause,
                status,
                location: candidate.location,
            });
        }
        Ok(())
    })?;
    Ok(observed)
}
// Independent Rust traces establish language order. The frontend variant only
// removes the println observer; it inspects cleanup facts, not native Drop parity.
fn trace(name: &str, source: &str, expected: &str) -> Result<(), Box<dyn std::error::Error>> {
    use std::{path::Path, process::Command};
    let folder = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/cleanup-traces-{}", std::process::id()));
    std::fs::create_dir_all(&folder)?;
    let input = folder.join(format!("{name}.rs"));
    std::fs::write(
        &input,
        source.replacen(
            "fn trace(_:u8){}",
            "fn trace(value:u8){println!(\"{}\",value);}",
            1,
        ),
    )?;
    let binary = folder.join(name);
    let build = Command::new("rustc")
        .args([
            "--edition=2024",
            "--crate-name=cleanup_trace",
            "--cap-lints=allow",
            "-o",
        ])
        .arg(&binary)
        .arg(input)
        .output()?;
    assert!(
        build.status.success(),
        "{name}: {}",
        String::from_utf8_lossy(&build.stderr)
    );
    let output = Command::new(binary).output()?;
    assert!(output.status.success(), "{name}");
    assert!(output.stderr.is_empty(), "{name}");
    assert_eq!(output.stdout, expected.as_bytes(), "{name}");
    Ok(())
}

#[test]
fn scope_cleanup_interleaves_extended_temporaries_and_locals()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        (
            "local-temporary",
            "fn main(){let a=D(1);let b=(&D(2),D(3));trace(9);}",
            "9\n3\n2\n1\n",
            vec!["b", "temporary", "a"],
        ),
        (
            "two-temporaries",
            "fn main(){let a=(&D(1),D(2));let b=(&D(3),D(4));trace(9);}",
            "9\n4\n3\n2\n1\n",
            vec!["b", "temporary", "a", "temporary"],
        ),
        (
            "partial-temporary",
            "fn main(){let [a,..]=[D(1),D(2),D(3)];trace(9);}",
            "2\n3\n9\n1\n",
            vec!["temporary", "a"],
        ),
        (
            "wildcard",
            "fn main(){let _=D(1);trace(9);}",
            "1\n9\n",
            vec!["temporary"],
        ),
        (
            "replacement",
            "fn make(n:u8)->D{trace(n+10);D(n)}fn main(){let mut a=make(1);a=make(2);trace(9);}",
            "11\n12\n1\n9\n2\n",
            vec!["a", "a"],
        ),
        (
            "zero-repeat",
            "fn main(){let _empty=[D(1);0];trace(9);}",
            "1\n9\n",
            vec!["temporary"],
        ),
    ];
    for (name, function, expected, labels) in cases {
        let source = format!("{PREFIX}{function}");
        trace(name, &source, expected)?;
        let observed = inspect(&source, "main")?;
        assert_eq!(
            observed
                .iter()
                .map(|o| o.label.as_str())
                .collect::<Vec<_>>(),
            labels,
            "{name}"
        );
        match name {
            "partial-temporary" => assert_eq!(observed[0].status, Status::Conditional),
            "replacement" => assert_eq!(observed[0].cause, Cause::Replacement),
            "zero-repeat" => assert_eq!(observed[0].cause, Cause::Discard),
            _ => assert!(observed.iter().all(|o| matches!(o.cause, Cause::Scope(_)))),
        }
    }
    Ok(())
}

#[test]
fn parameter_captures_and_remaining_fields_keep_parameter_order()
-> Result<(), Box<dyn std::error::Error>> {
    let source = format!(
        "{PREFIX}fn f((x,_):(D,D),(_,y):(D,D)){{trace(9);}}fn main(){{f((D(0),D(1)),(D(2),D(3)));}}"
    );
    trace("parameter-patterns", &source, "9\n3\n2\n0\n1\n")?;
    let observed = inspect(&source, "f")?;
    assert_eq!(
        observed
            .iter()
            .map(|o| o.label.as_str())
            .collect::<Vec<_>>(),
        ["y", "parameter:1", "x", "parameter:0"]
    );
    assert_eq!(
        observed.iter().map(|o| o.status).collect::<Vec<_>>(),
        [
            Status::Always,
            Status::Conditional,
            Status::Always,
            Status::Conditional
        ]
    );
    Ok(())
}

#[test]
fn return_transfer_precedes_cleanup_at_shared_points() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("tail", "fn f()->D{let a=D(1);D(2)}"),
        ("explicit", "fn f()->D{let a=D(1);return D(2);}"),
        ("block", "fn f()->D{let a=D(1);{{D(2)}}}"),
    ];
    for (name, function) in cases {
        let source = format!("{PREFIX}{function}fn main(){{let result=f();trace(9);}}");
        trace(name, &source, "1\n9\n2\n")?;
        let observed = inspect(&source, "f")?;
        assert_eq!(
            observed
                .iter()
                .map(|o| o.label.as_str())
                .collect::<Vec<_>>(),
            ["a"]
        );
    }
    Ok(())
}

fn function<'a>(
    program: &'a body::ir::Program,
    report: &rustgm::resolve::expansion::Report,
    name: &str,
) -> Result<&'a Body, &'static str> {
    program.bodies.iter().find(|b| matches!(b.owner, Owner::Function { definition, .. }
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == name))
        .ok_or("missing function")
}

#[test]
fn edge_cleanup_does_not_clear_the_other_successor() -> Result<(), Box<dyn std::error::Error>> {
    let source = format!(
        "{PREFIX}enum E{{A(D),B}}fn f(flag:bool){{if let E::A(ref a)=E::A(D(1))&&flag{{trace(8);}}else{{trace(9);}}}}fn main(){{f(false);f(true);}}"
    );
    trace("edge-order", &source, "1\n9\n8\n1\n")?;
    let (report, mut declarations) = semantic::analyze(&source)?;
    let mut program = body::build(&report, &declarations)?;
    let cfg = program.bodies.iter_mut().find(|b| matches!(b.owner, Owner::Function { definition, .. }
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == "f"))
        .ok_or("missing function")?;
    // The builder currently gives let-chain failure its own block. Move that
    // block's initial scope exits onto its unique incoming edge: equivalent CFG,
    // with a live owner on only one successor's cleanup edge.
    let (predecessor, failed) = cfg
        .blocks
        .iter()
        .enumerate()
        .find_map(|(index, block)| {
            if let body::ir::Terminator::Branch { condition, no, .. } = block.terminator
                && matches!(
                    cfg.values[condition.0 as usize].kind,
                    Rvalue::Read(Place::Local(_))
                )
            {
                Some((index, no))
            } else {
                None
            }
        })
        .ok_or("missing boolean branch")?;
    let start = cfg.blocks[failed.0 as usize].start;
    let mut relocated = 0;
    for exit in &mut cfg.storage.as_mut().ok_or("missing storage")?.exits {
        if exit.location
            == (Location::At {
                block: failed,
                before: start,
            })
        {
            exit.location = Location::Edge {
                block: body::ir::BlockId(u32::try_from(predecessor)?),
                target: failed,
            };
            relocated += 1;
        }
    }
    assert!(relocated > 0);
    let init = initialization::analyze(
        &declarations.types,
        &declarations.aggregates,
        &declarations.derives,
        &mut declarations.copies,
        cfg,
    )
    .map_err(|e| e.message)?;
    let mut edges = 0;
    init.visit_events(|event, point| {
        let Event::Cleanup(candidate) = event else {
            return Ok(());
        };
        let Location::Edge { block, target } = candidate.location else {
            return Ok(());
        };
        let Place::Temporary(value) = candidate.place else {
            return Ok(());
        };
        if cfg
            .owned
            .as_ref()
            .is_none_or(|o| o.creation(value).is_none())
        {
            return Ok(());
        }
        let path =
            &init.ownership.paths.entries[candidate.path.ok_or("missing edge path")?.0 as usize];
        if point.status(path)? == Status::Never {
            return Ok(());
        }
        let body::ir::Terminator::Branch { yes, no, .. } = cfg.blocks[block.0 as usize].terminator
        else {
            return Err("edge cleanup is not a conditional branch");
        };
        let other = if yes == target { no } else { yes };
        assert_eq!(init.at_entry(target, path)?, Status::Never);
        assert_eq!(init.at_entry(other, path)?, Status::Always);
        edges += 1;
        Ok(())
    })?;
    assert_eq!(edges, 1);
    Ok(())
}

#[test]
fn let_else_failure_ends_initializer_before_entering_else() -> Result<(), Box<dyn std::error::Error>>
{
    let source = format!(
        "{PREFIX}enum E{{A(D),B(D)}}fn f(){{let E::A(ref a)=E::B(D(1))else{{trace(9);return;}};}}fn main(){{f();}}"
    );
    trace("let-else", &source, "1\n9\n")?;
    let observed = inspect(&source, "f")?;
    assert_eq!(
        observed
            .iter()
            .filter(|o| o.cause == Cause::Initializer)
            .count(),
        1
    );
    let failure = observed
        .iter()
        .find(|o| o.cause == Cause::Initializer)
        .ok_or("missing initializer cleanup")?;
    assert_eq!(failure.label, "temporary");
    assert_eq!(failure.status, Status::Always);
    assert!(matches!(failure.location, Location::At { .. }));
    Ok(())
}

#[test]
fn loop_reentry_does_not_retain_previous_iteration_temporaries()
-> Result<(), Box<dyn std::error::Error>> {
    let source = format!(
        "{PREFIX}fn f(mut n:u8){{while n>0{{let a=D(n);D(n+10);n-=1;if n>0{{continue;}}break;}}}}fn main(){{f(2);trace(9);}}"
    );
    trace("loop", &source, "12\n2\n11\n1\n9\n")?;
    rustc::qualify("cleanup-loop", &source, true)?;
    let (report, mut declarations) = semantic::analyze(&source)?;
    let program = body::build(&report, &declarations)?;
    let cfg = function(&program, &report, "f")?;
    let init = initialization::analyze(
        &declarations.types,
        &declarations.aggregates,
        &declarations.derives,
        &mut declarations.copies,
        cfg,
    )
    .map_err(|e| e.message)?;
    let mut births = 0;
    let mut cleanups = 0;
    init.visit_events(|event, point| {
        match event {
            Event::Point(Location::At { block, before })
                if before < cfg.blocks[block.0 as usize].end =>
            {
                if let Action::Evaluate(value) = cfg.statements[before as usize].action
                    && cfg
                        .owned
                        .as_ref()
                        .is_some_and(|o| o.creation(value).is_some())
                {
                    let path = init
                        .ownership
                        .paths
                        .temporary_path(value)
                        .ok_or("missing owned path")?;
                    assert_eq!(
                        point.status(&init.ownership.paths.entries[path.0 as usize])?,
                        Status::Never
                    );
                    births += 1;
                }
            }
            Event::Cleanup(candidate) if candidate.cause != Cause::Replacement => {
                if let Some(path) = candidate.path
                    && point.status(&init.ownership.paths.entries[path.0 as usize])?
                        == Status::Always
                {
                    cleanups += 1;
                }
            }
            _ => {}
        }
        Ok(())
    })?;
    assert_eq!(births, 2);
    assert!(cleanups >= 3);
    Ok(())
}

#[test]
fn scope_analysis_preserves_unowned_places_and_borrowed_replacements()
-> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("unit-pattern", "fn f(){let ()=();let _=&();}", false),
        ("empty-pattern", "fn f(){let []:[D;0]=[];}", false),
        ("wildcard-array", "fn f(){let [_,_]=[D(1),D(2)];}", false),
        (
            "extended-block",
            "fn f(){let a=&{D(1)};let b=&{D(2)};}",
            false,
        ),
        ("borrowed-store", "fn f(x:&mut D){*x=D(1);}", true),
        (
            "borrowed-index-store",
            "fn f(x:&mut [D;2],i:usize){x[i]=D(1);}",
            true,
        ),
        (
            "owned-index-store",
            "fn f(mut x:[D;2],i:usize){x[i]=D(1);}",
            true,
        ),
        (
            "guarded-match",
            "fn f(flag:bool){match (D(1),D(2)){(ref a,_)if flag=>{},(_,b)=>{}}}",
            false,
        ),
    ];
    for (name, function, untracked) in cases {
        let source = format!("{PREFIX}{function}");
        rustc::qualify(name, &source, true)?;
        let (report, mut declarations) = semantic::analyze(&source)?;
        let program = body::build(&report, &declarations)?;
        let mut external = 0;
        for cfg in &program.bodies {
            let init = initialization::analyze(
                &declarations.types,
                &declarations.aggregates,
                &declarations.derives,
                &mut declarations.copies,
                cfg,
            )
            .map_err(|e| format!("{name}: {}", e.message))?;
            init.visit_events(|event, _| {
                if let Event::Cleanup(candidate) = event
                    && candidate.cause == Cause::Replacement
                    && candidate.path.is_none()
                {
                    external += 1;
                }
                Ok(())
            })?;
        }
        assert_eq!(external, usize::from(untracked), "{name}");
    }
    Ok(())
}
