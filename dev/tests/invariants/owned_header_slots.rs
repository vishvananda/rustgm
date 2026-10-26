//! Header-slot indexes preserve real owners; successful lookup is not proof.
use super::*;
use crate::support::crates::generics::{Range, source::*};
const _: [(); 80] = [(); size_of::<ScopeIndex<'static>>()];

fn empty_rows(slots: Vec<ParameterSlot>, scopes: Vec<ParameterScope>) -> SourceRows {
    SourceRows {
        policy: Policy {
            edition: crate::support::edition::Edition::Rust2024,
            provider: ArtifactRef(0),
            roles: Range { start: 0, end: 0 },
        },
        declarations: Vec::new(),
        body_owners: Vec::new(),
        slots,
        scopes,
        binders: Vec::new(),
        types: Vec::new(),
        roots: Vec::new(),
        regions: Vec::new(),
        arguments: Vec::new(),
        clauses: Vec::new(),
        clause_indices: Vec::new(),
        region_bounds: Vec::new(),
        region_slots: Vec::new(),
        trait_applications: Vec::new(),
        function_applications: Vec::new(),
        traits: Vec::new(),
        implementations: Vec::new(),
        functions: Vec::new(),
        formals: Vec::new(),
        members: Vec::new(),
        member_links: Vec::new(),
        roles: Vec::new(),
        originals: Vec::new(),
        body_inputs: Vec::new(),
        body_units: Vec::new(),
    }
}
fn definition(kind: DefinitionKind, parent: Option<u32>) -> Definition {
    Definition {
        kind,
        parent: parent.map(DeclarationRef),
    }
}
fn slot(owner: u32, ordinal: u32, definition: Option<u32>, sized: bool) -> ParameterSlot {
    ParameterSlot {
        owner: DeclarationRef(owner),
        ordinal,
        index: ordinal,
        definition: definition.map(DeclarationRef),
        kind: ParameterKind::Type {
            sized: sized.then_some(DeclarationAddress {
                artifact: ArtifactRef(0),
                declaration: DeclarationRef(0),
            }),
        },
    }
}
fn scope(owner: u32, parent: Option<u32>, start: u32, end: u32, captures: u32) -> ParameterScope {
    ParameterScope {
        owner: DeclarationRef(owner),
        parent: parent.map(DeclarationRef),
        declared: Range { start, end },
        captures,
    }
}
fn fixture() -> (SourceRows, Vec<Definition>) {
    let definitions = vec![
        definition(DefinitionKind::Trait, None),
        definition(DefinitionKind::SelfType, Some(0)),
        definition(DefinitionKind::Function, Some(0)),
        definition(DefinitionKind::TypeParameter, Some(2)),
        definition(DefinitionKind::Trait, None),
        definition(DefinitionKind::SelfType, Some(4)),
    ];
    let rows = empty_rows(
        vec![
            slot(1, 0, Some(1), false),
            slot(2, 0, Some(3), true),
            slot(5, 0, Some(5), false),
        ],
        vec![
            scope(1, None, 0, 1, 0),
            scope(0, Some(1), 1, 1, 1),
            scope(2, Some(0), 1, 2, 1),
            scope(5, None, 2, 3, 0),
            scope(4, Some(5), 3, 3, 1),
        ],
    );
    (rows, definitions)
}

#[test]
fn inherited_slots_keep_self_and_method_owners_distinct() -> Result<(), &'static str> {
    let (rows, definitions) = fixture();
    let mut work = Work::default();
    let index = ScopeIndex::new(&rows, &definitions, 0, &mut work)?;
    assert_eq!(
        TraitFlags::from_bits(TraitFlags::RESTRICTED_IMPL)?.bits(),
        TraitFlags::RESTRICTED_IMPL
    );
    for unknown in [512, 512 | TraitFlags::CONST, u16::MAX] {
        assert!(TraitFlags::from_bits(unknown).is_err());
    }
    assert_eq!(TraitFlags::from_bits(255)?.bits(), 255);
    println!(
        "SourceRows={} SourceFunction={} SourceTrait={} SourceImpl={} SourceSlot={} SourceScope={} SourceCall={} SourceOriginal={} ScopeIndex={}",
        size_of::<SourceRows>(),
        size_of::<FunctionHeader>(),
        size_of::<TraitHeader>(),
        size_of::<ImplHeader>(),
        size_of::<ParameterSlot>(),
        size_of::<ParameterScope>(),
        size_of::<CallRequest>(),
        size_of::<Original>(),
        size_of::<ScopeIndex<'_>>()
    );
    assert_eq!(index.complete_slots(DeclarationRef(2), &mut work)?, 2);
    assert_eq!(index.complete_type_slots(DeclarationRef(2), &mut work)?, 2);
    assert!(index.contains(DeclarationRef(2), SlotRef(0), &mut work)?);
    assert!(index.contains(DeclarationRef(2), SlotRef(1), &mut work)?);
    assert!(!index.contains(DeclarationRef(2), SlotRef(2), &mut work)?);
    assert!(!index.contains(DeclarationRef(4), SlotRef(0), &mut work)?);
    let before = work.steps;
    assert!(
        index
            .contains(DeclarationRef(99), SlotRef(0), &mut work)
            .is_err()
    );
    assert!(
        index
            .contains(DeclarationRef(2), SlotRef(99), &mut work)
            .is_err()
    );
    assert_eq!(work.steps - before, 8);
    Ok(())
}

#[test]
fn complete_type_formals_exclude_region_slots_without_losing_unused_parameters()
-> Result<(), &'static str> {
    let definitions = vec![
        definition(DefinitionKind::Function, None),
        definition(DefinitionKind::TypeParameter, Some(0)),
        definition(DefinitionKind::RegionParameter, Some(0)),
    ];
    let rows = empty_rows(
        vec![
            slot(0, 0, Some(1), false),
            ParameterSlot {
                owner: DeclarationRef(0),
                ordinal: 1,
                index: 0,
                definition: Some(DeclarationRef(2)),
                kind: ParameterKind::Region,
            },
        ],
        vec![scope(0, None, 0, 2, 0)],
    );
    let mut work = Work::default();
    let index = ScopeIndex::new(&rows, &definitions, 0, &mut work)?;
    assert_eq!(index.complete_slots(DeclarationRef(0), &mut work)?, 2);
    let before = work.steps;
    assert_eq!(index.complete_type_slots(DeclarationRef(0), &mut work)?, 1);
    assert!(
        index
            .complete_type_slots(DeclarationRef(99), &mut work)
            .is_err()
    );
    assert_eq!(work.steps - before, 4);
    Ok(())
}

#[test]
fn malformed_slots_reject_foreign_self_kind_owner_gap_and_missing_scope() {
    for case in 0..9 {
        let (mut rows, mut definitions) = fixture();
        match case {
            0 => rows.slots[0].definition = Some(DeclarationRef(5)),
            1 => rows.slots[1].definition = Some(DeclarationRef(2)),
            2 => definitions[3].parent = Some(DeclarationRef(5)),
            3 => rows.slots[1].ordinal = 1,
            4 => rows.scopes[2].captures = 0,
            5 => rows.scopes[3].parent = Some(DeclarationRef(1)),
            6 => rows.scopes.swap(1, 2),
            7 => rows.scopes.truncate(3),
            8 => rows.slots[1].index = 99,
            _ => unreachable!(),
        }
        let mut work = Work::default();
        assert!(
            ScopeIndex::new(&rows, &definitions, 0, &mut work).is_err(),
            "case {case}"
        );
        assert!(work.steps > 0);
    }
}

#[test]
fn duplicate_definition_and_scope_bindings_cannot_reuse_owner_identity() {
    let (mut rows, definitions) = fixture();
    rows.slots.insert(2, slot(2, 1, Some(3), true));
    rows.scopes[2].declared.end = 3;
    rows.scopes[3].declared = Range { start: 3, end: 4 };
    rows.scopes[4].declared = Range { start: 4, end: 4 };
    assert!(ScopeIndex::new(&rows, &definitions, 0, &mut Work::default()).is_err());
    let (mut rows, definitions) = fixture();
    rows.scopes[4].owner = DeclarationRef(0);
    assert!(ScopeIndex::new(&rows, &definitions, 0, &mut Work::default()).is_err());
}

#[test]
fn absent_binding_is_preserved_and_preflight_failure_keeps_consumed_work()
-> Result<(), &'static str> {
    let (mut rows, definitions) = fixture();
    rows.slots[1].definition = None;
    let mut work = Work::default();
    let index = ScopeIndex::new(&rows, &definitions, 0, &mut work)?;
    assert!(index.contains(DeclarationRef(2), SlotRef(1), &mut work)?);
    assert!(rows.slots[1].definition.is_none());
    let mut work = Work::default();
    assert!(ScopeIndex::new(&rows, &definitions, crates::MAX_BYTES, &mut work).is_err());
    assert!(work.steps > 0);
    let mut work = Work {
        steps: crates::MAX_COMPONENTS,
        ..Work::default()
    };
    assert!(ScopeIndex::new(&rows, &definitions, 0, &mut work).is_err());
    assert!(work.steps > crates::MAX_COMPONENTS);
    Ok(())
}

fn scaled(width: u32, contexts: u32) -> (SourceRows, Vec<Definition>) {
    let mut definitions = vec![
        definition(DefinitionKind::Trait, None),
        definition(DefinitionKind::SelfType, Some(0)),
    ];
    let mut slots = vec![slot(1, 0, Some(1), false)];
    for index in 0..width {
        definitions.push(definition(DefinitionKind::TypeParameter, Some(0)));
        slots.push(slot(0, index, Some(index + 2), true));
    }
    let mut scopes = vec![scope(1, None, 0, 1, 0), scope(0, Some(1), 1, width + 1, 1)];
    for index in 0..contexts {
        definitions.push(definition(DefinitionKind::Function, Some(0)));
        scopes.push(scope(
            width + 2 + index,
            Some(0),
            width + 1,
            width + 1,
            width + 1,
        ));
    }
    (empty_rows(slots, scopes), definitions)
}

#[test]
fn shared_scope_work_varies_width_and_siblings_without_flattening() -> Result<(), &'static str> {
    let widths = [8, 32, 128];
    let mut observations = [[0; 3]; 3];
    for (row, width) in widths.into_iter().enumerate() {
        for (column, contexts) in widths.into_iter().enumerate() {
            let (rows, definitions) = scaled(width, contexts);
            assert_eq!(rows.slots.len(), width as usize + 1);
            let mut work = Work::default();
            let index = ScopeIndex::new(&rows, &definitions, 0, &mut work)?;
            observations[row][column] = work.steps;
            let retained = index.retained_bytes();
            let before = work.steps;
            for query in 0..128 {
                assert!(index.contains(
                    DeclarationRef(width + 2 + query % contexts),
                    SlotRef(width),
                    &mut work
                )?);
            }
            assert_eq!(work.steps - before, 4 * 128);
            assert_eq!(index.retained_bytes(), retained);
            println!(
                "width={width} contexts={contexts} preparation={} lookup={} retained={retained}",
                observations[row][column],
                work.steps - before
            );
        }
    }
    for edge in 0..2 {
        for other in 1..3 {
            assert_eq!(
                observations[edge + 1][0] - observations[edge][0],
                observations[edge + 1][other] - observations[edge][other]
            );
            assert_eq!(
                observations[0][edge + 1] - observations[0][edge],
                observations[other][edge + 1] - observations[other][edge]
            );
        }
    }
    Ok(())
}
