#[path = "support/semantic.rs"]
mod semantic;
use rustgm::{
    body::{
        self,
        ir::{Action, BlockId, Place, Rvalue, Terminator},
    },
    borrow::{
        liveness,
        regions::{
            self,
            points::{Point, infer},
        },
    },
    types::{Kind, RegionId, outlives::LiveRegions},
};

fn check_write(source: &str, expected: bool) -> Result<(), Box<dyn std::error::Error>> {
    let (report, facts) = semantic::analyze(source)?;
    let program = body::build(&report, &facts)?;
    let contracts = body::regions::collect(&facts)?;
    let proof = regions::check_bodies(
        &facts.types,
        &facts.aggregates,
        &facts.lifetimes.variances,
        &contracts,
        &program,
    )?;
    let cfg = program.bodies.last().ok_or("missing f")?;
    let proof = proof
        .bodies
        .iter()
        .find(|b| b.source == cfg.source)
        .ok_or("missing proof")?;
    let x = cfg
        .locals
        .iter()
        .position(|l| {
            l.definition.is_some_and(|d| {
                report
                    .package
                    .names
                    .get(report.resolution.definitions[d.0 as usize].name)
                    == "x"
            })
        })
        .ok_or("missing x")?;
    let at = cfg.statements.iter().rposition(|s|
        matches!(s.action, Action::Store { place: Place::Local(id), .. } if id.0 as usize == x)
    ).ok_or("missing mutation")?;
    let borrow = cfg
        .values
        .iter()
        .find(|v| matches!(v.kind, Rvalue::Borrow { .. }))
        .ok_or("missing borrow")?;
    let Some(Kind::Reference { region, .. }) = facts.types.kind(borrow.ty) else {
        return Err("missing borrow region".into());
    };
    let flow = liveness::analyze(&facts.types, &mut LiveRegions::default(), cfg)?;
    let seeds = flow
        .seeds
        .iter()
        .map(|&(id, point)| {
            proof
                .identities
                .get(id)
                .map(|id| (id, point))
                .ok_or("unregistered live region")
        })
        .collect::<Result<Vec<_>, _>>()?;
    let live = infer(&proof.solution, flow.points, seeds, 100_000)?;
    let block = cfg
        .blocks
        .iter()
        .position(|b| b.start as usize <= at && at < b.end as usize)
        .ok_or("missing mutation block")?;
    let point = flow
        .point_before(BlockId(u32::try_from(block)?), u32::try_from(at)?)
        .ok_or("missing point")?;
    assert_eq!(
        live.contains(
            proof.identities.get(region).ok_or("missing loan region")?,
            point
        )?,
        expected,
        "{source}"
    );
    assert!(flow.steps < 10_000);
    Ok(())
}

fn rustc(source: &str, index: usize, accepted: bool) -> Result<(), Box<dyn std::error::Error>> {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../obj/value-liveness-{}", std::process::id()));
    std::fs::create_dir_all(&directory)?;
    let input = directory.join(format!("case-{index}.rs"));
    std::fs::write(&input, source)?;
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
        .arg(input)
        .output()?;
    std::fs::write(
        directory.join(format!("case-{index}.stderr")),
        &result.stderr,
    )?;
    assert_eq!(
        result.status.code(),
        Some(i32::from(!accepted)),
        "rustc: {source}: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    Ok(())
}

#[test]
fn lifetime_use_points_follow_last_use_branches_loops_and_pending_operands()
-> Result<(), Box<dyn std::error::Error>> {
    // These declarations are independent of the new pass. A live shared borrow
    // at the selected x write conflicts in Rust; loan conflict checking itself
    // remains a separate, unreleased phase.
    for (index, (source, live)) in [
        ("fn f(){let mut x=0;let p=&x;let y=*p;x=1;}", false),
        ("fn f(){let mut x=0;let p=&x;x=1;let y=*p;}", true),
        ("fn f(){let mut x=0;let p=&x;x=1;}", false),
        (
            "fn f(){let mut x=0;let z=0;let mut p=&x;let y=*p;x=1;p=&z;let y=*p;}",
            false,
        ),
        (
            "fn f(c:bool){let mut x=0;let p=&x;if c{let y=*p;}else{x=1;}}",
            false,
        ),
        (
            "fn f(c:bool){let mut x=0;let p=&x;if c{x=1;}let y=*p;}",
            true,
        ),
        (
            "fn f(mut c:bool){let mut x=0;let p=&x;while c{let y=*p;x=1;c=false;}}",
            true,
        ),
        (
            "fn f(mut c:bool){let mut x=0;let p=&x;while c{let y=*p;c=false;}x=1;}",
            false,
        ),
        (
            "fn take(_: &i32,_:i32){}fn f(){let mut x=0;take(&x,{x=1;0});}",
            true,
        ),
        ("fn f(){let mut x=0;let a=[&x;0];x=1;let b=a;}", false),
        ("fn f(){let mut x=0;let a=[&x;1];x=1;let b=a;}", true),
        (
            "fn f(c:bool){let mut x=0;let p=&x;if c{let y=*p;return;}x=1;}",
            false,
        ),
        (
            "fn f(){let mut x=0;let z=0;let t=(&x,&z);x=1;let y=*t.1;}",
            true,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        rustc(source, index, !live)?;
        check_write(source, live)?;
    }
    Ok(())
}

#[test]
fn fresh_implicit_reborrows_inherit_destination_use_points()
-> Result<(), Box<dyn std::error::Error>> {
    for suffix in ["let q:&u8=p;let y=*q;", "let q:[&'static u8;0]=[p;0];"] {
        let (report, facts) = semantic::analyze(&format!("fn f(p:&mut u8){{{suffix}}}"))?;
        let program = body::build(&report, &facts)?;
        let contracts = body::regions::collect(&facts)?;
        let proof = regions::check_bodies(
            &facts.types,
            &facts.aggregates,
            &facts.lifetimes.variances,
            &contracts,
            &program,
        )?;
        let cfg = &program.bodies[0];
        let proof = &proof.bodies[0];
        let q = cfg
            .locals
            .iter()
            .find(|l| {
                l.definition.is_some_and(|d| {
                    report
                        .package
                        .names
                        .get(report.resolution.definitions[d.0 as usize].name)
                        == "q"
                })
            })
            .ok_or("missing q")?;
        let mut components = LiveRegions::default();
        let q_regions = components.get(&facts.types, q.ty)?;
        let borrow = cfg
            .values
            .iter()
            .find(|v| matches!(v.kind, Rvalue::Borrow { .. }))
            .ok_or("missing reborrow")?;
        let Some(Kind::Reference { region, .. }) = facts.types.kind(borrow.ty) else {
            return Err("missing reference type".into());
        };
        if suffix.contains("static") {
            assert!(
                q_regions.is_empty(),
                "discarded repeat does not use its element lifetime"
            );
            assert_ne!(region, RegionId::STATIC);
        } else {
            let seeds = q_regions
                .iter()
                .map(|id| {
                    proof
                        .identities
                        .get(*id)
                        .map(|r| (r, Point(0)))
                        .ok_or("missing destination")
                })
                .collect::<Result<Vec<_>, _>>()?;
            let live = infer(&proof.solution, 1, seeds, 1_000)?;
            assert!(
                live.contains(
                    proof
                        .identities
                        .get(region)
                        .ok_or("missing reborrow identity")?,
                    Point(0)
                )?
            );
        }
    }
    Ok(())
}

#[test]
fn imaginary_edges_keep_uses_and_empty_blocks_have_distinct_points()
-> Result<(), Box<dyn std::error::Error>> {
    let (report, facts) = semantic::analyze("fn f(p:&u8,c:bool){if c{let y=*p;}}")?;
    let mut program = body::build(&report, &facts)?;
    let cfg = &mut program.bodies[0];
    for block in &mut cfg.blocks {
        if let Terminator::Branch { yes, no, .. } = block.terminator {
            block.terminator = Terminator::FalseEdge {
                real: no,
                imaginary: yes,
            };
        }
    }
    let extra = cfg.blocks.len();
    cfg.blocks.push(body::ir::Block {
        start: 0,
        end: 0,
        terminator: Terminator::Unreachable,
        source: cfg.blocks[0].source,
    });
    let flow = liveness::analyze(&facts.types, &mut LiveRegions::default(), cfg)?;
    assert!(!flow.reachable[extra]);
    let parameter_region = RegionId::bound(0, 0)?;
    for (index, block) in cfg.blocks.iter().enumerate() {
        if let Terminator::FalseEdge { imaginary, .. } = block.terminator {
            assert!(flow.reachable[imaginary.0 as usize]);
            let point = flow
                .point_before(BlockId(u32::try_from(index)?), block.end)
                .ok_or("missing branch point")?;
            assert!(flow.seeds.contains(&(parameter_region, point)));
        }
    }
    let mut unique = std::collections::HashSet::new();
    for (i, block) in cfg.blocks.iter().enumerate() {
        let id = BlockId(u32::try_from(i)?);
        for statement in block.start..=block.end {
            assert!(unique.insert(flow.point_before(id, statement).ok_or("missing location")?));
        }
        assert_eq!(flow.point_before(id, block.end + 1), None);
    }
    assert_eq!(unique.len(), flow.points as usize);
    cfg.blocks[0].terminator = Terminator::Jump(BlockId(u32::MAX));
    assert!(liveness::analyze(&facts.types, &mut LiveRegions::default(), cfg).is_err());
    Ok(())
}
