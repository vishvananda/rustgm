//! Neutral SSA snapshots are distinct from later writes to their source local.
use super::*;
use crate::support::crates::owner_catalog::{BodyRef, DeclarationRef};
type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

fn locals() -> Vec<g::Local> {
    vec![
        g::Local {
            scope: 0,
            storage_parent: None,
            ty: g::TypeRef(1),
            kind: g::LocalKind::Parameter(0),
            definition: None,
            mutable: true,
            site: 0,
        },
        g::Local {
            scope: 0,
            storage_parent: None,
            ty: g::TypeRef(1),
            kind: g::LocalKind::Parameter(1),
            definition: None,
            mutable: false,
            site: 0,
        },
    ]
}

fn values() -> Vec<s::Value> {
    vec![
        s::Value {
            ty: g::TypeRef(1),
            kind: s::ValueKind::Read(s::Place::Local(0)),
        },
        s::Value {
            ty: g::TypeRef(1),
            kind: s::ValueKind::Read(s::Place::Local(1)),
        },
        s::Value {
            ty: g::TypeRef(0),
            kind: s::ValueKind::Read(s::Place::Projection(0)),
        },
    ]
}

fn snapshot() -> s::Original {
    let statements = [
        s::Action::Evaluate(0),
        s::Action::Evaluate(1),
        s::Action::Store {
            place: s::Place::Local(0),
            value: 1,
        },
        s::Action::Project(0),
        s::Action::Evaluate(2),
    ]
    .into_iter()
    .map(|action| s::Statement { site: 0, action })
    .collect();
    s::Original {
        body: BodyRef(0),
        declaration: DeclarationRef(0),
        result: g::TypeRef(0),
        sites: 1,
        context: s::CallContext::Runtime,
        variables: Vec::new(),
        locals: locals(),
        parameter_bindings: Vec::new(),
        projections: vec![s::Projection {
            base: s::Place::Temporary(0),
            ty: g::TypeRef(0),
            kind: s::ProjectionKind::Dereference {
                region: s::RegionRef(0),
                mutable: false,
            },
        }],
        values: values(),
        statements,
        blocks: vec![g::Block {
            statements: g::Range { start: 0, end: 5 },
            site: 0,
            terminator: g::Terminator::Return(2),
        }],
        scopes: Vec::new(),
        exits: Vec::new(),
        temporaries: Vec::new(),
        owned: None,
        arguments: Vec::new(),
        openings: Vec::new(),
        calls: Vec::new(),
        guards: None,
        texts: None,
        assembly: None,
    }
}

#[test]
fn evaluated_snapshot_survives_local_overwrite_and_distinct_block_numbering() -> Result {
    let mut original = snapshot();
    validate(&original, &mut Work::default(), false)?;
    assert_eq!(original.projections[0].base, s::Place::Temporary(0));
    // Physical block numbering is not dominance. Execution still visits 0,2,1.
    original.blocks = vec![
        g::Block {
            statements: g::Range { start: 0, end: 2 },
            site: 0,
            terminator: g::Terminator::Jump(2),
        },
        g::Block {
            statements: g::Range { start: 4, end: 5 },
            site: 0,
            terminator: g::Terminator::Return(2),
        },
        g::Block {
            statements: g::Range { start: 2, end: 4 },
            site: 0,
            terminator: g::Terminator::Jump(1),
        },
    ];
    validate(&original, &mut Work::default(), false)?;
    original.blocks[1].terminator = g::Terminator::Jump(2);
    validate(&original, &mut Work::default(), false)?;
    assert_eq!(size_of::<s::Place>(), 8);
    assert_eq!(size_of::<crate::body::ir::Body>(), 456);
    assert_eq!(size_of::<crate::syntax::ast::Node>(), 36);
    let mut previous = None;
    for width in [16, 64, 256] {
        let mut wide = snapshot();
        for _ in 0..width {
            let value = u32::try_from(wide.values.len())?;
            wide.values.push(s::Value {
                ty: g::TypeRef(0),
                kind: s::ValueKind::Read(s::Place::Projection(0)),
            });
            wide.statements.push(s::Statement {
                site: 0,
                action: s::Action::Evaluate(value),
            });
        }
        wide.blocks[0].statements.end = u32::try_from(wide.statements.len())?;
        let mut work = Work::default();
        validate(&wide, &mut work, false)?;
        let mut repeated = Work::default();
        validate(&wide, &mut repeated, false)?;
        assert_eq!(repeated.steps, work.steps);
        if let Some(previous) = previous {
            assert!(work.steps < previous * 5);
        }
        previous = Some(work.steps);
    }
    Ok(())
}

#[test]
fn snapshot_rejects_forward_unevaluated_duplicate_and_cyclic_dependencies() {
    for mutation in 0..6 {
        let mut original = snapshot();
        match mutation {
            0 => original.statements.swap(0, 3),
            1 => original.statements.swap(3, 4),
            2 => original.projections[0].base = s::Place::Temporary(u32::MAX),
            3 => original.values[0].kind = s::ValueKind::Read(s::Place::Projection(0)),
            4 => original.statements[0].action = s::Action::Evaluate(1),
            5 => original.statements[0].action = s::Action::Mention(s::Place::Local(0)),
            _ => unreachable!(),
        }
        assert!(
            validate(&original, &mut Work::default(), false).is_err(),
            "{mutation}"
        );
    }
}

#[test]
fn disconnected_definitions_cannot_authorize_reached_snapshot_and_failure_is_paid() {
    let mut original = snapshot();
    original.blocks = vec![
        g::Block {
            statements: g::Range { start: 3, end: 5 },
            site: 0,
            terminator: g::Terminator::Return(2),
        },
        g::Block {
            statements: g::Range { start: 0, end: 3 },
            site: 0,
            terminator: g::Terminator::Unreachable,
        },
    ];
    let mut work = Work::default();
    assert!(matches!(
        validate(&original, &mut work, false),
        Err(error) if error.contains("not dominated")
    ));
    assert!(work.steps > 0);
    work.steps = crate::support::crates::MAX_COMPONENTS;
    for _ in 0..2 {
        assert!(validate(&original, &mut work, false).is_err());
    }
    assert!(work.steps > crate::support::crates::MAX_COMPONENTS);
}

#[test]
fn direct_ssa_reads_retain_exact_evaluations_and_reject_invalid_dependencies() -> Result {
    let mut original = snapshot();
    original.values[2].kind = s::ValueKind::Read(s::Place::Temporary(0));
    validate(&original, &mut Work::default(), false)?;
    assert!(matches!(
        original.values[2].kind,
        s::ValueKind::Read(s::Place::Temporary(0))
    ));
    original.statements.swap(0, 4);
    assert!(matches!(
        validate(&original, &mut Work::default(), false),
        Err(error) if error.contains("not dominated")
    ));
    original.statements.swap(0, 4);
    original.values[0].kind = s::ValueKind::Read(s::Place::Temporary(2));
    assert!(matches!(
        validate(&original, &mut Work::default(), false),
        Err(error) if error.contains("cyclic")
    ));
    original.values[0].kind = s::ValueKind::Read(s::Place::Local(0));
    original.values[2].kind = s::ValueKind::Read(s::Place::Temporary(u32::MAX));
    assert!(validate(&original, &mut Work::default(), false).is_err());
    Ok(())
}

/// Version thirteen: a branching body keeps each snapshot use in the block
/// that evaluated it. Older profiles still refuse every two-way exit.
#[test]
fn branching_snapshots_are_block_local_only_in_version_thirteen() {
    let branch = g::Terminator::Branch {
        condition: 1,
        yes: 1,
        no: 2,
    };
    let block = |start, end, terminator| g::Block {
        statements: g::Range { start, end },
        site: 0,
        terminator,
    };
    let mut local = snapshot();
    local.blocks = vec![
        block(0, 5, branch),
        block(5, 5, g::Terminator::Unreachable),
        block(5, 5, g::Terminator::Unreachable),
    ];
    assert!(validate(&local, &mut Work::default(), true).is_ok());
    assert!(matches!(
        validate(&local, &mut Work::default(), false),
        Err(error) if error.contains("two-way exits")
    ));
    // The projection and its read follow the branch, away from `v0`.
    let mut crossing = snapshot();
    crossing.blocks = vec![
        block(0, 3, branch),
        block(3, 5, g::Terminator::Return(2)),
        block(5, 5, g::Terminator::Unreachable),
    ];
    let mut work = Work::default();
    assert!(matches!(
        validate(&crossing, &mut work, true),
        Err(error) if error.contains("not dominated")
    ));
    assert!(work.steps > 0);
    // A terminator's condition is a use in its own block too.
    local.blocks[0] = block(
        0,
        1,
        g::Terminator::Branch {
            condition: 2,
            yes: 1,
            no: 2,
        },
    );
    local.blocks[1] = block(1, 5, g::Terminator::Unreachable);
    assert!(validate(&local, &mut Work::default(), true).is_err());
}

/// Version thirteen: an assembly value's inputs are snapshot uses like a
/// call's arguments. An input evaluated only in a sibling block is refused;
/// without the assembly arm it would drop out of both availability walks.
#[test]
fn assembly_inputs_must_be_dominated_by_their_evaluation() {
    let block = |start, end, terminator| g::Block {
        statements: g::Range { start, end },
        site: 0,
        terminator,
    };
    let body = |actions: [s::Action; 6], blocks| {
        let mut original = snapshot();
        original.values.push(s::Value {
            ty: g::TypeRef(0),
            kind: s::ValueKind::Assembly {
                instruction: 0,
                arguments: g::Range { start: 0, end: 1 },
            },
        });
        original.arguments = vec![2];
        original.statements = actions
            .into_iter()
            .map(|action| s::Statement { site: 0, action })
            .collect();
        original.blocks = blocks;
        original
    };
    let branch = g::Terminator::Branch {
        condition: 1,
        yes: 1,
        no: 2,
    };
    let store = s::Action::Store {
        place: s::Place::Local(0),
        value: 1,
    };
    let forged = body(
        [
            s::Action::Evaluate(1),
            store,
            s::Action::Evaluate(3),
            s::Action::Evaluate(0),
            s::Action::Project(0),
            s::Action::Evaluate(2),
        ],
        vec![
            block(0, 2, branch),
            block(2, 3, g::Terminator::Unreachable),
            block(3, 6, g::Terminator::Unreachable),
        ],
    );
    let mut work = Work::default();
    assert_eq!(
        validate(&forged, &mut work, true),
        Err("source SSA snapshot use is not dominated by its actual evaluation")
    );
    assert!(work.steps > 0);
    let twin = body(
        [
            s::Action::Evaluate(1),
            store,
            s::Action::Evaluate(0),
            s::Action::Project(0),
            s::Action::Evaluate(2),
            s::Action::Evaluate(3),
        ],
        vec![
            block(0, 2, branch),
            block(2, 2, g::Terminator::Unreachable),
            block(2, 6, g::Terminator::Unreachable),
        ],
    );
    assert_eq!(validate(&twin, &mut Work::default(), true), Ok(()));
}
