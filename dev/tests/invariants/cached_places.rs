//! One cache shares validated ordinary paths and preserves consumer distinctions.
use crate::support::hash::HashMap;
use crate::{
    body::ir::{BlockId, LocalKind, Place, ProjectionId, Rvalue},
    const_eval::{
        Frame, Limits, Machine,
        memory::{RawAccess, RawAlignment, Resolved},
        place_cache::{Key, Mode},
        static_reads::EvaluationContext,
    },
    sema::{
        model::Scalar,
        values::{Constant, Evaluator, Failure, Reader},
    },
    test_source,
    types::{Primitive, TypeId},
};
use std::fmt::Write;

fn scalar(value: u128) -> Constant {
    Constant::scalar(Scalar::Integer(value))
}
fn empty_frame() -> Frame {
    Frame {
        instance: 0,
        context: EvaluationContext::Constant,
        body: 0,
        block: BlockId(0),
        next: 0,
        operand_cursor: usize::MAX,
        return_to: None,
        locals: Vec::new(),
        values: Vec::new(),
        temporaries: HashMap::default(),
        projections: HashMap::default(),
        raw_views: HashMap::default(),
        union_views: HashMap::default(),
        allocations: Vec::new(),
    }
}
fn place(value: u128) -> Resolved {
    Resolved {
        address: scalar(value),
        ty: TypeId::primitive(Primitive::U32),
        raw: None,
        union: None,
    }
}
#[test]
fn address_cache_layout_and_mode_invalidation_remain_bounded() {
    type Old = HashMap<ProjectionId, Resolved>;
    type Current = HashMap<Key, Resolved>;
    println!(
        "cache-layout Frame={} Resolved={} Key={} OldEntry={} NewEntry={} OldMap={} NewMap={}",
        size_of::<Frame>(),
        size_of::<Resolved>(),
        size_of::<Key>(),
        size_of::<(ProjectionId, Resolved)>(),
        size_of::<(Key, Resolved)>(),
        size_of::<Old>(),
        size_of::<Current>()
    );
    assert_eq!(size_of::<Frame>(), 312);
    assert_eq!(size_of::<Key>(), 8);
    assert_eq!(
        size_of::<(Key, Resolved)>(),
        size_of::<(ProjectionId, Resolved)>()
    );
    let mut frame = empty_frame();
    let id = ProjectionId(7);
    frame.remember_projection(id, Mode::Value, false, place(1));
    frame.remember_projection(id, Mode::Address, false, place(2));
    assert!(
        frame
            .projected(id, Mode::Value)
            .is_some_and(|(p, c)| !c && p.address == scalar(1))
    );
    assert!(
        frame
            .projected(id, Mode::Address)
            .is_some_and(|(p, c)| !c && p.address == scalar(2))
    );
    frame.forget_projection(id);
    assert!(frame.projections.is_empty());
    frame.remember_projection(id, Mode::Value, true, place(3));
    for mode in [Mode::Value, Mode::Address] {
        assert!(
            frame
                .projected(id, mode)
                .is_some_and(|(p, c)| c && p.address == scalar(3))
        );
    }
    frame.forget_projection(id);
    assert!(frame.projections.is_empty());
}

fn with_path(
    depth: usize,
    check: impl FnOnce(&mut Machine<'_, '_>, &mut Frame, Place, usize) -> Result<(), Failure>,
) -> Result<(), String> {
    let mut source = String::from("#![recursion_limit=\"512\"]struct E{v:u32}");
    let mut parent = String::from("E");
    for n in 0..depth {
        write!(&mut source, "struct S{n}{{p:{parent}}}").map_err(|e| e.to_string())?;
        parent = format!("S{n}");
    }
    write!(
        &mut source,
        "const fn f(mut r:{parent}){{let p=&raw mut r{}.v;let _=p;}}",
        ".p".repeat(depth)
    )
    .map_err(|e| e.to_string())?;
    let report = test_source::resolve(&source, true)?;
    let mut facts = crate::pipeline::analyze_source(&report)
        .map_err(|e| e.to_string())?
        .facts;
    let program = crate::body::build_concrete(&report, &facts).map_err(|e| e.to_string())?;
    let body = program.bodies.first().ok_or("body")?;
    let selected = body
        .values
        .iter()
        .find_map(|v| {
            if let Rvalue::Address(p) = v.kind {
                Some(p)
            } else {
                None
            }
        })
        .ok_or("address place")?;
    let count = body.projections.len();
    let evaluator = Evaluator::new(&report, &mut facts);
    let mut machine = Machine::new(evaluator, [body], Limits::default())?;
    let mut value = machine
        .evaluator
        .fields(&[scalar(37)], None)
        .map_err(|e| e.message().to_owned())?;
    for _ in 0..depth {
        value = machine
            .evaluator
            .fields(&[value], None)
            .map_err(|e| e.message().to_owned())?;
    }
    machine
        .begin(0, &[value])
        .map_err(|e| e.message.to_owned())?;
    let mut frame = machine.frames.pop().ok_or("frame")?;
    check(&mut machine, &mut frame, selected, count).map_err(|e| e.message().to_owned())
}

#[test]
fn address_cache_common_paths_are_constant_work_on_warm_repetition() -> Result<(), String> {
    for depth in [1, 8, 32] {
        for repeats in [8, 32, 128] {
            with_path(depth, |machine, frame, selected, count| {
                let logical = machine.place(frame, selected)?.ok_or("logical")?;
                let entries = frame.projections.len();
                let capacity = frame.projections.capacity();
                assert!(entries <= count);
                let slots = machine.memory.retained_cells();
                let before = machine.work.steps;
                for _ in 0..repeats {
                    let address = machine.address_place(frame, selected)?.ok_or("address")?;
                    assert!(address.address == logical.address);
                    let value = machine.place(frame, selected)?.ok_or("value")?;
                    assert!(value.address == logical.address);
                }
                assert_eq!(machine.work.steps - before, 2 * repeats);
                assert_eq!(
                    (
                        frame.projections.len(),
                        frame.projections.capacity(),
                        machine.memory.retained_cells()
                    ),
                    (entries, capacity, slots)
                );
                println!(
                    "common-cache depth={depth} repeats={repeats} entries={entries} capacity={capacity} warm={}",
                    machine.work.steps - before
                );
                Ok(())
            })?;
        }
    }
    Ok(())
}

#[test]
fn address_cache_warm_paths_keep_backing_restrictions_and_expiry_fresh() -> Result<(), String> {
    with_path(8, |machine, frame, selected, _| {
        let value = machine.place(frame, selected)?.ok_or("field")?;
        let param = machine.bodies[0]
            .locals
            .iter()
            .position(|l| matches!(l.kind, LocalKind::Parameter(0)))
            .ok_or("parameter")?;
        let root = frame.locals[param]
            .ok_or("local")?
            .location()
            .ok_or("root")?;
        machine.convert_byte_root(root)?;
        let physical = machine.address_place(frame, selected)?.ok_or("address")?;
        assert!(physical.address == value.address);
        machine.write_place(physical, scalar(43))?;
        assert!(machine.read(value)?.is_some_and(|v| v == scalar(43)));
        let restricted = Resolved {
            address: physical.address.restrict_shared(),
            raw: Some(RawAccess {
                alignment: RawAlignment::Required,
                location: physical.address.location(),
                ty: physical.ty,
            }),
            ..physical
        };
        assert!(machine.check_raw_write(restricted).is_err());
        assert!(
            Reader::new(machine.evaluator.facts())
                .location(root)
                .is_ok()
        );
        machine.memory.expire(&[root]);
        assert!(machine.read(value).is_err());
        Ok(())
    })
}

#[test]
fn address_cache_project_and_select_invalidate_both_consumers() -> Result<(), String> {
    use crate::body::ir::{Action, LocalId};
    with_path(1, |machine, frame, selected, _| {
        let Place::Projection(id) = selected else {
            return Err("projection".into());
        };
        for action in [
            Action::Project(id),
            Action::Select {
                projection: id,
                anchor: Place::Local(LocalId(0)),
            },
        ] {
            frame.forget_projection(id);
            frame.remember_projection(id, Mode::Value, false, place(666));
            frame.remember_projection(id, Mode::Address, false, place(999));
            machine.statement(frame, action)?;
            let (selected, common) = frame.projected(id, Mode::Address).ok_or("recomputed")?;
            assert!(common);
            assert!(machine.read(selected)?.is_some_and(|v| v == scalar(37)));
            assert!(selected.address != scalar(999));
        }
        Ok(())
    })
}
