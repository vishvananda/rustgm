use crate::qualify as rustc;
use crate::semantic;
use rustgm::{
    body::{
        self,
        ir::{BlockId, Body, Terminator},
        storage::Location,
    },
    borrow::{
        destruction::{self, Effect, Plan, glue::Catalog},
        ownership::initialization::{self, Event, Status},
    },
};
use std::collections::{HashMap, VecDeque};

// Interpret only emitted flag stores. No body statement, move rule, ownership
// path or cleanup clearing rule is used to reconstruct a missing write.
fn lane(a: Location, b: Location) -> bool {
    match (a, b) {
        (Location::At { block: a, .. }, Location::At { block: b, .. }) => a == b,
        (
            Location::Edge {
                block: a,
                target: x,
            },
            Location::Edge {
                block: b,
                target: y,
            },
        ) => a == b && x == y,
        _ => false,
    }
}
fn execute(
    plan: &Plan,
    location: Location,
    state: &mut [u8],
    mut observe: impl FnMut(usize, &[u8]),
) {
    for (index, event) in plan
        .events
        .iter()
        .enumerate()
        .filter(|(_, e)| lane(e.location, location))
    {
        match event.effect {
            Effect::Flags { range, initialized } => state[range.start as usize..range.end as usize]
                .fill(if initialized { 2 } else { 1 }),
            Effect::Point | Effect::Cleanup(_) => observe(index, state),
            Effect::Statement
            | Effect::Birth(_)
            | Effect::StorageEnd(_)
            | Effect::Deinitialize { .. } => {}
        }
    }
}
fn successors(terminator: Terminator) -> Vec<BlockId> {
    match terminator {
        Terminator::Jump(target) => vec![target],
        Terminator::Branch { yes, no, .. }
        | Terminator::FalseEdge {
            real: yes,
            imaginary: no,
        } => vec![yes, no],
        _ => Vec::new(),
    }
}
fn simulate(body: &Body, plan: &Plan) -> Result<HashMap<usize, Vec<u8>>, &'static str> {
    let mut entries = vec![None; body.blocks.len()];
    entries[0] = Some(vec![1u8; plan.flags.len()]);
    let mut work = VecDeque::from([BlockId(0)]);
    let mut iterations = 0;
    while let Some(block) = work.pop_front() {
        iterations += 1;
        assert!(iterations < 10_000);
        let mut state = entries[block.0 as usize].clone().ok_or("entry")?;
        execute(
            plan,
            Location::At { block, before: 0 },
            &mut state,
            |_, _| {},
        );
        for target in successors(body.blocks[block.0 as usize].terminator) {
            let mut output = state.clone();
            execute(
                plan,
                Location::Edge { block, target },
                &mut output,
                |_, _| {},
            );
            let old = &mut entries[target.0 as usize];
            if let Some(previous) = old {
                for (next, &old) in output.iter_mut().zip(previous.iter()) {
                    *next |= old;
                }
                if *previous == output {
                    continue;
                }
            }
            *old = Some(output);
            work.push_back(target);
        }
    }
    let mut observed = HashMap::new();
    for (index, entry) in entries.into_iter().enumerate() {
        let Some(mut state) = entry else {
            continue;
        };
        let block = BlockId(u32::try_from(index).map_err(|_| "block id")?);
        execute(
            plan,
            Location::At { block, before: 0 },
            &mut state,
            |i, s| {
                observed.insert(i, s.to_vec());
            },
        );
        for target in successors(body.blocks[index].terminator) {
            let mut edge = state.clone();
            execute(plan, Location::Edge { block, target }, &mut edge, |i, s| {
                observed.insert(i, s.to_vec());
            });
        }
    }
    Ok(observed)
}
fn check(source: &str) -> Result<(usize, usize), Box<dyn std::error::Error>> {
    let (report, mut facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let mut catalog = Catalog::new(&facts.types, &facts.aggregates, &facts.members, methods)?;
    let mut counts = (0, 0);
    for body in &program.bodies {
        let init = initialization::analyze(
            &facts.types,
            &facts.aggregates,
            &facts.derives,
            &mut facts.copies,
            body,
        )
        .map_err(|e| e.message)?;
        let plan = destruction::select(&mut catalog, &init).map_err(|e| e.message)?;
        counts.0 += plan.flags.len();
        counts.1 += verify(body, &init, &plan)?;
    }
    Ok(counts)
}
fn verify(
    body: &Body,
    init: &initialization::Facts<'_>,
    plan: &Plan,
) -> Result<usize, Box<dyn std::error::Error>> {
    let mut count = 0;
    let observed = simulate(body, plan)?;
    let mut events = plan
        .events
        .iter()
        .enumerate()
        .filter(|(_, e)| matches!(e.effect, Effect::Point | Effect::Cleanup(_)));
    init.visit_events(|event, point| {
        let (index, selected) = events.next().ok_or("missing plan event")?;
        match (event, selected.effect) {
            (Event::Point(location), Effect::Point) => assert_eq!(selected.location, location),
            (Event::Cleanup(candidate), Effect::Cleanup(id)) => {
                assert_eq!(selected.location, candidate.location);
                assert_eq!(plan.cleanups[id as usize].candidate.place, candidate.place);
            }
            _ => return Err("plan observation order differs"),
        }
        let actual = observed.get(&index).ok_or("missing flag observation")?;
        for (&state, flag) in actual.iter().zip(&plan.flags) {
            let expected = match point.bit(flag.0)? {
                Status::Never => 1,
                Status::Always => 2,
                Status::Conditional => 3,
            };
            assert_eq!(state, expected, "flag {} at {event:?}", flag.0);
            count += 1;
        }
        Ok(())
    })?;
    assert!(events.next().is_none());
    Ok(count)
}
#[test]
fn emitted_flag_writes_match_initialization_on_branches_loops_and_replacements()
-> Result<(), Box<dyn std::error::Error>> {
    let prefix = "struct D;impl Drop for D{fn drop(&mut self){}}";
    let cases = [
        "fn f(flag:bool){let d:D;if flag{d=D;}}",
        "fn f(flag:bool,mut pair:(D,D)){if flag{let _x=pair.0;}pair.0=D;}",
        "fn f(flag:bool,mut pair:(D,D)){if flag{let _x=pair.0;}pair=(D,D);}",
        "fn f(flag:bool,items:[D;1000000000]){if flag{let [a,..,z]=items;}}",
        "fn f(mut flag:bool){loop{let d:D;if flag{d=D;}if flag{flag=false;continue;}break;}}",
        "fn f(flag:bool)->D{let d:D;if flag{d=D;if flag{return d;}}D}",
        "fn f(flag:bool){let d:D;let _reference=&D;if flag{d=D;}let _empty=[D;0];}",
        "enum E{A(D),B}fn f(flag:bool,e:E){let d:D;if flag{d=D;}let E::A(_a)=e else{return;};}",
    ];
    for (index, case) in cases.iter().enumerate() {
        let source = format!("{prefix}{case}");
        rustc::qualify(&format!("drop-flags-{index}"), &source, true)?;
        let (flags, observations) = check(&source)?;
        assert!(flags > 0, "no conditional cleanup in {case}");
        assert!(observations > 10, "insufficient observations in {case}");
    }
    Ok(())
}

#[test]
fn edge_flag_clear_preserves_the_other_successor() -> Result<(), Box<dyn std::error::Error>> {
    use body::ir::{Action, Owner, Rvalue};
    let source = "struct D;impl Drop for D{fn drop(&mut self){}}fn f(flag:bool,branch:bool){let d:D;if flag{d=D;}if branch{return;}}";
    rustc::qualify("drop-flag-edge", source, true)?;
    let (report, mut facts) = semantic::analyze(source)?;
    let mut program = body::build(&report, &facts)?;
    let methods = &program.drops.as_ref().ok_or("methods")?.methods;
    let mut catalog = Catalog::new(&facts.types, &facts.aggregates, &facts.members, methods)?;
    let cfg = program.bodies.iter_mut().find(|b| matches!(b.owner, Owner::Function { definition, .. }
        if report.package.names.get(report.resolution.definitions[definition.0 as usize].name) == "f"))
        .ok_or("function")?;
    let (parent, target, other) = cfg
        .blocks
        .iter()
        .enumerate()
        .find_map(|(i, b)| {
            if let Terminator::Branch { yes, no, .. } = b.terminator
                && matches!(cfg.blocks[yes.0 as usize].terminator, Terminator::Return(_))
            {
                Some((i, yes, no))
            } else {
                None
            }
        })
        .ok_or("return branch")?;
    let parent = BlockId(u32::try_from(parent)?);
    let block = &cfg.blocks[target.0 as usize];
    // Unit return preparation has no owned effects. Its exits can equivalently
    // run on the unique incoming branch edge, before the remaining StorageEnd.
    let mut relocated = 0;
    for exit in &mut cfg.storage.as_mut().ok_or("storage")?.exits {
        if let Location::At { block: id, before } = exit.location
            && id == target
        {
            assert!(cfg.statements[block.start as usize..before as usize].iter().all(|s|
                matches!(s.action, Action::Evaluate(v) if matches!(cfg.values[v.0 as usize].kind, Rvalue::Unit))
                || matches!(s.action, Action::StorageEnd { .. })
            ));
            exit.location = Location::Edge {
                block: parent,
                target,
            };
            relocated += 1;
        }
    }
    assert!(relocated > 0);
    let init = initialization::analyze(
        &facts.types,
        &facts.aggregates,
        &facts.derives,
        &mut facts.copies,
        cfg,
    )
    .map_err(|e| e.message)?;
    let plan = destruction::select(&mut catalog, &init).map_err(|e| e.message)?;
    assert_eq!(plan.flags.len(), 1);
    let flag = plan.flags[0];
    assert_eq!(init.bit_at_entry(target, flag.0)?, Status::Never);
    assert_eq!(init.bit_at_entry(other, flag.0)?, Status::Conditional);
    assert!(plan.events.iter().any(|e| e.location
        == (Location::Edge {
            block: parent,
            target
        })
        && matches!(
            e.effect,
            Effect::Flags {
                initialized: false,
                ..
            }
        )));
    assert!(verify(cfg, &init, &plan)? > 10);
    Ok(())
}
