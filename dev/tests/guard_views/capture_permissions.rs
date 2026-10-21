//! Capture binding authority is independent of the physical place and referent capability.
use super::*;
use rustgm::types::RegionId;
fn inspect(f: &Fixture) -> Result<(), &'static str> {
    let mut body = f.body.clone();
    body.blocks = vec![Block {
        start: 0,
        end: u32::try_from(body.statements.len()).map_err(|_| "statements")?,
        source: NodeId(0),
        terminator: Terminator::Unreachable,
    }];
    ownership::analyze_body(
        &f.types,
        &f.definitions,
        &HashMap::new(),
        &mut Copies::default(),
        &body,
    )
    .map(|_| ())
    .map_err(|e| e.message)
}
fn reference(f: &mut Fixture, referent: TypeId, mutable: bool) -> Result<TypeId, &'static str> {
    f.types.intern(Kind::Reference {
        referent,
        mutable,
        region: RegionId::bound(0, 0)?,
    })
}
fn dereference(f: &mut Fixture, place: Place, referent: TypeId) -> Result<Place, &'static str> {
    f.project(
        place,
        ProjectionKind::Dereference {
            mutable: true,
            region: RegionId::bound(0, 0)?,
        },
        referent,
    )
}
fn borrow(f: &mut Fixture, place: Place, ty: TypeId, kind: BorrowKind) -> Result<(), &'static str> {
    let id = ValueId(u32::try_from(f.body.values.len()).map_err(|_| "values")?);
    f.body.values.push(Value {
        ty,
        kind: Rvalue::Borrow { place, kind },
    });
    f.action(Action::Evaluate(id));
    Ok(())
}
#[test]
fn capture_binding_authority_survives_owned_fields_and_indices() -> Result<(), &'static str> {
    for mutable in [false, true] {
        for indexed in [false, true] {
            let mut f = Fixture::new();
            let container = if indexed {
                f.types.intern(Kind::Array {
                    element: BYTE,
                    length: 2,
                })?
            } else {
                f.tuple()?
            };
            let root = Place::Local(f.local(container, LocalKind::Parameter(0))?);
            let boundary =
                f.project(root, ProjectionKind::CaptureBoundary { mutable }, container)?;
            let child = if indexed {
                f.project(boundary, ProjectionKind::ConstantIndex(0), BYTE)?
            } else {
                f.field(boundary, 0)?
            };
            f.write(child)?;
            assert_eq!(inspect(&f).is_ok(), mutable);
        }
    }
    Ok(())
}
#[test]
fn only_a_source_dereference_clears_capture_binding_authority() -> Result<(), &'static str> {
    for mode in 0..3 {
        let mut f = Fixture::new();
        let inner = reference(&mut f, BYTE, true)?;
        let stored = reference(&mut f, inner, true)?;
        let root = Place::Local(f.local(stored, LocalKind::Parameter(0))?);
        let logical = dereference(&mut f, root, inner)?;
        let boundary = f.project(
            logical,
            ProjectionKind::CaptureBoundary { mutable: false },
            inner,
        )?;
        match mode {
            0 => {
                let replacement = Place::Local(f.local(inner, LocalKind::Parameter(1))?);
                let value = ValueId(u32::try_from(f.body.values.len()).map_err(|_| "values")?);
                f.read(replacement, inner)?;
                f.action(Action::Store {
                    place: boundary,
                    value,
                });
            }
            1 => borrow(&mut f, boundary, stored, BorrowKind::Immediate)?,
            _ => {
                let referent = dereference(&mut f, boundary, BYTE)?;
                f.write(referent)?;
            }
        }
        let result = inspect(&f);
        match mode {
            0 => assert_eq!(
                result,
                Err("cannot assign through an immutable captured binding")
            ),
            1 => assert_eq!(
                result,
                Err("cannot mutably borrow an immutable captured binding")
            ),
            _ => result?,
        }
    }
    Ok(())
}
#[test]
fn unique_capture_formation_does_not_require_a_mutable_binding() -> Result<(), &'static str> {
    for (kind, mutable, accepted) in [
        (BorrowKind::Immediate, true, false),
        (BorrowKind::UniqueImmutable, true, true),
        (BorrowKind::UniqueImmutable, false, false),
    ] {
        let mut f = Fixture::new();
        let inner = reference(&mut f, BYTE, true)?;
        let outer = reference(&mut f, inner, mutable)?;
        let local = f.local(inner, LocalKind::Parameter(0))?;
        f.body.locals[local.0 as usize].mutable = false;
        f.body.locals[local.0 as usize].definition = Some(DefId(1));
        borrow(&mut f, Place::Local(local), outer, kind)?;
        assert_eq!(inspect(&f).is_ok(), accepted);
    }
    Ok(())
}
#[test]
fn capture_boundaries_share_physical_owned_move_paths() -> Result<(), &'static str> {
    for same_field in [false, true] {
        let mut f = Fixture::new();
        let item = f.noncopy()?;
        let fields = f.types.intern_list(&[item, BYTE])?;
        let tuple = f.types.intern(Kind::Tuple(fields))?;
        let root = Place::Local(f.local(tuple, LocalKind::Parameter(0))?);
        let boundary = f.project(
            root,
            ProjectionKind::CaptureBoundary { mutable: false },
            tuple,
        )?;
        let captured = f.project(boundary, ProjectionKind::Field(Selection::Tuple(0)), item)?;
        f.read(captured, item)?;
        let (index, ty) = if same_field { (0, item) } else { (1, BYTE) };
        let physical = f.project(root, ProjectionKind::Field(Selection::Tuple(index)), ty)?;
        f.read(physical, ty)?;
        assert_eq!(inspect(&f).is_ok(), !same_field);
    }
    Ok(())
}
