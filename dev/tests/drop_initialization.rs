use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body::{
        self,
        ir::{Action, BlockId, Body, Owner, Rvalue},
    },
    borrow::ownership::{
        Component, Root,
        initialization::{self, Status},
    },
};

fn probe(body: &Body, mark: rustgm::resolve::model::DefId) -> Result<BlockId, &'static str> {
    for (index, block) in body.blocks.iter().enumerate() {
        for statement in body.statements(block) {
            if let Action::Evaluate(id) = statement.action
                && let Rvalue::Call { callee, .. } = body.values[id.0 as usize].kind
                && matches!(body.values[callee.0 as usize].kind, Rvalue::Function(id) if id == mark)
            {
                return u32::try_from(index)
                    .map(BlockId)
                    .map_err(|_| "block overflow");
            }
        }
    }
    Err("missing marker call")
}

#[test]
fn initialized_bits_do_not_reuse_write_history() -> Result<(), Box<dyn std::error::Error>> {
    let cases = [
        ("never", "let mut x:D;if flag{let y=D;}", Status::Never),
        (
            "conditional",
            "let mut x:D;if flag{x=D;}",
            Status::Conditional,
        ),
        (
            "both",
            "let mut x:D;if flag{x=D;}else{x=D;}",
            Status::Always,
        ),
        (
            "both-moved",
            "let mut x=D;if flag{let y=x;}else{let z=x;}",
            Status::Never,
        ),
        (
            "one-moved",
            "let mut x=D;if flag{let y=x;}",
            Status::Conditional,
        ),
        (
            "reinitialized",
            "let mut x=D;if flag{let y=x;x=D;}",
            Status::Always,
        ),
        (
            "loop-moved",
            "let mut x:D;while flag{x=D;let y=x;}",
            Status::Never,
        ),
        (
            "loop-initialized",
            "let mut x:D;while flag{x=D;}",
            Status::Conditional,
        ),
    ];
    for (name, statements, expected) in cases {
        let source = format!(
            "struct D;impl Drop for D{{fn drop(&mut self){{}}}}fn mark(){{}}fn f(flag:bool){{{statements}mark();}}"
        );
        rustc::qualify(name, &source, true)?;
        let (report, mut facts) = semantic::analyze(&source)?;
        let program = body::build(&report, &facts)?;
        let definition = |name| {
            facts
                .functions
                .iter()
                .find(|f| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[f.definition.0 as usize].name)
                        == name
                })
                .map(|f| f.definition)
                .ok_or("missing function")
        };
        let f = definition("f")?;
        let mark = definition("mark")?;
        let body = program
            .bodies
            .iter()
            .find(|b| matches!(b.owner, Owner::Function { definition, .. } if definition==f))
            .ok_or("missing body")?;
        let local = body
            .locals
            .iter()
            .position(|local| {
                local.definition.is_some_and(|id| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[id.0 as usize].name)
                        == "x"
                })
            })
            .ok_or("missing x")?;
        let init = initialization::analyze(
            &facts.types,
            &facts.aggregates,
            &facts.derives,
            &mut facts.copies,
            body,
        )
        .map_err(|e| e.message)?;
        let path = &init.ownership.paths.entries[local];
        assert_eq!(init.at_entry(probe(body, mark)?, path)?, expected, "{name}");
    }
    Ok(())
}

#[test]
fn partial_moves_preserve_independent_field_initialization()
-> Result<(), Box<dyn std::error::Error>> {
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}fn mark(){}fn f(flag:bool){let pair=(D,D);if flag{let moved=pair.0;}else{let view=&pair.0;}mark();let other=pair.1;}";
    rustc::qualify("drop-partial-flags", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let find = |name| {
        facts
            .functions
            .iter()
            .find(|f| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[f.definition.0 as usize].name)
                    == name
            })
            .map(|f| f.definition)
            .ok_or("function")
    };
    let f = find("f")?;
    let mark = find("mark")?;
    let cfg = program
        .bodies
        .iter()
        .find(|b| matches!(b.owner,Owner::Function{definition,..} if definition==f))
        .ok_or("body")?;
    let pair = cfg
        .locals
        .iter()
        .position(|l| {
            l.definition.is_some_and(|id| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[id.0 as usize].name)
                    == "pair"
            })
        })
        .ok_or("pair")?;
    let init = initialization::analyze(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        cfg,
    )
    .map_err(|e| e.message)?;
    let block = probe(cfg, mark)?;
    let mut checked = 0;
    for path in &init.ownership.paths.entries {
        if matches!(path.root, Root::Local(id) if id.0 as usize == pair)
            && let Some(Component::Field(rustgm::sema::members::Selection::Tuple(field))) =
                path.component
        {
            assert_eq!(
                init.at_entry(block, path)?,
                if field == 0 {
                    Status::Conditional
                } else {
                    Status::Always
                }
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 2);
    Ok(())
}

#[test]
fn statement_queries_distinguish_replacement_from_first_initialization()
-> Result<(), Box<dyn std::error::Error>> {
    use rustgm::body::{ir::Place, storage::Location};
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}fn f(flag:bool){let mut x:D;if flag{x=D;}x=D;}";
    rustc::qualify("drop-replacement-state", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let cfg=program.bodies.iter().find(|b|matches!(b.owner,Owner::Function{definition,..} if report.package.names.get(report.resolution.definitions[definition.0 as usize].name)=="f")).ok_or("f")?;
    let local = cfg
        .locals
        .iter()
        .position(|l| {
            l.definition.is_some_and(|id| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[id.0 as usize].name)
                    == "x"
            })
        })
        .ok_or("x")?;
    let init = initialization::analyze(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        cfg,
    )
    .map_err(|e| e.message)?;
    let path = &init.ownership.paths.entries[local];
    let mut stores = Vec::new();
    let mut cleanup = Vec::new();
    init.visit(|location, point| {
        let Location::At { block, before } = location else {
            return Err("unexpected entry edge");
        };
        if before == cfg.blocks[block.0 as usize].end {
            return Ok(());
        }
        match cfg.statements[before as usize].action {
            Action::Store {
                place: Place::Local(id),
                ..
            } if id.0 as usize == local => stores.push(point.status(path)?),
            Action::StorageEnd { first, last }
                if first.0 as usize <= local && local <= last.0 as usize =>
            {
                cleanup.push(point.status(path)?);
            }
            Action::StorageDead(id) if id.0 as usize == local => cleanup.push(point.status(path)?),
            _ => {}
        }
        Ok(())
    })?;
    assert_eq!(stores.len(), 2);
    assert!(stores.contains(&Status::Never));
    assert!(stores.contains(&Status::Conditional));
    // The lexical endpoint runs before its overlapping explicit storage end.
    assert_eq!(cleanup, vec![Status::Never]);
    let mut scopes = Vec::new();
    init.visit_events(|event, point| {
        if let initialization::Event::Cleanup(candidate) = event
            && matches!(candidate.cause, initialization::Cause::Scope(_))
            && matches!(candidate.place, Place::Local(id) if id.0 as usize == local)
        {
            scopes.push(point.status(path)?);
        }
        Ok(())
    })?;
    assert_eq!(scopes, vec![Status::Always]);
    assert_eq!(
        init.at_entry(BlockId(u32::MAX), path),
        Err("initialization block is unreachable")
    );
    Ok(())
}
