use rustgm::support::hash::HashMap;
use rustgm::{
    body::ir::*,
    borrow::ownership,
    resolve::{model::DefId, unit::FileId},
    sema::model::Scalar,
    syntax::ast::NodeId,
    types::{ItemId, Kind, Primitive, Store, TypeId, aggregate, copy::Copies},
};

const BYTE: TypeId = TypeId::primitive(Primitive::U8);
struct Fixture {
    types: Store,
    definitions: aggregate::Definitions,
    body: Body,
}
impl Fixture {
    fn new() -> Self {
        Self {
            types: Store::default(),
            definitions: aggregate::Definitions::default(),
            body: Body {
                assembly: None,
                const_context: false,
                mutable_statics: None,
                static_types: None,
                instance: None,
                owned: None,
                storage: None,
                promotions: None,
                resolved_callables: None,
                closure_calls: None,
                temporaries: Vec::new(),
                guards: Vec::new(),
                guard_places: Vec::new(),
                projections: Vec::new(),
                owner: Owner::Function {
                    definition: DefId(0),
                    signature: TypeId::UNIT,
                    execution: None,
                },
                source: NodeKey {
                    file: FileId(0),
                    node: NodeId::new(0),
                }
                .into(),
                result: TypeId::UNIT,
                locals: Vec::new(),
                values: vec![Value {
                    ty: TypeId::primitive(Primitive::Bool),
                    kind: Rvalue::Scalar(ScalarId(0)),
                }],
                scalars: vec![Scalar::Boolean(true)],
                arguments: Vec::new(),
                statements: vec![Statement {
                    source: NodeId::new(0).into(),
                    action: Action::Evaluate(ValueId(0)),
                }],
                blocks: Vec::new(),
            },
        }
    }
    fn action(&mut self, action: Action) {
        self.body.statements.push(Statement {
            source: NodeId::new(0).into(),
            action,
        });
    }
    fn local(&mut self, ty: TypeId, kind: LocalKind) -> Result<LocalId, &'static str> {
        let id = LocalId(u32::try_from(self.body.locals.len()).map_err(|_| "locals")?);
        self.body.locals.push(Local {
            scope: rustgm::body::scopes::Id(0),
            ty,
            kind,
            definition: None,
            mutable: true,
            source: NodeId::new(0).into(),
            storage_parent: id.0.checked_sub(1).map(LocalId),
        });
        self.action(Action::StorageLive(id));
        Ok(id)
    }
    fn tuple(&mut self) -> Result<TypeId, &'static str> {
        let fields = self.types.intern_list(&[BYTE, BYTE])?;
        self.types.intern(Kind::Tuple(fields))
    }
    fn noncopy(&mut self) -> Result<TypeId, &'static str> {
        let item = ItemId(0);
        let fields = self.types.intern_list(&[BYTE])?;
        self.definitions.insert(
            item,
            aggregate::Kind::Struct,
            aggregate::Repr::default(),
            &[aggregate::Variant {
                fields,
                discriminant: 0,
            }],
        )?;
        self.types
            .intern(Kind::Adt(item, rustgm::types::GenericArgs::EMPTY))
    }
    fn project(
        &mut self,
        base: Place,
        kind: ProjectionKind,
        ty: TypeId,
    ) -> Result<Place, &'static str> {
        let id =
            ProjectionId(u32::try_from(self.body.projections.len()).map_err(|_| "projections")?);
        self.body.projections.push(Projection { base, kind, ty });
        self.action(Action::Project(id));
        Ok(Place::Projection(id))
    }
    fn field(&mut self, base: Place, index: u32) -> Result<Place, &'static str> {
        self.project(base, ProjectionKind::Field(Selection::Tuple(index)), BYTE)
    }
    fn guard(
        &mut self,
        tested: &[Place],
        view: Option<(TypeId, Place)>,
    ) -> Result<(GuardId, Option<LocalId>), &'static str> {
        let id = GuardId(u32::try_from(self.body.guards.len()).map_err(|_| "guards")?);
        let tested_start = u32::try_from(self.body.guard_places.len()).map_err(|_| "places")?;
        self.body.guard_places.extend_from_slice(tested);
        let tested_end = u32::try_from(self.body.guard_places.len()).map_err(|_| "places")?;
        let views_start = u32::try_from(self.body.locals.len()).map_err(|_| "locals")?;
        self.action(Action::GuardStart(id));
        let view = if let Some((ty, place)) = view {
            let local = self.local(ty, LocalKind::Guard(id))?;
            self.action(Action::BindView { view: local, place });
            Some(local)
        } else {
            None
        };
        let views_end = u32::try_from(self.body.locals.len()).map_err(|_| "locals")?;
        self.body.guards.push(Guard {
            tested_start,
            tested_end,
            views_start,
            views_end,
        });
        Ok((id, view))
    }
    fn read(&mut self, place: Place, ty: TypeId) -> Result<(), &'static str> {
        let id = ValueId(u32::try_from(self.body.values.len()).map_err(|_| "values")?);
        self.body.values.push(Value {
            ty,
            kind: Rvalue::Read(place),
        });
        self.action(Action::Evaluate(id));
        Ok(())
    }
    fn write(&mut self, place: Place) -> Result<(), &'static str> {
        let ty = match place {
            Place::Local(id) => self.body.locals[id.0 as usize].ty,
            Place::Projection(id) => self.body.projections[id.0 as usize].ty,
            _ => return Err("fixture write has no type"),
        };
        let value = ValueId(u32::try_from(self.body.values.len()).map_err(|_| "values")?);
        self.read(place, ty)?;
        self.action(Action::Store { place, value });
        Ok(())
    }
    fn block(&mut self, terminator: Terminator) -> Result<(), &'static str> {
        let start = self.body.blocks.last().map_or(0, |b| b.end);
        let end = u32::try_from(self.body.statements.len()).map_err(|_| "statements")?;
        self.body.blocks.push(Block {
            start,
            end,
            source: NodeId::new(0).into(),
            terminator,
        });
        Ok(())
    }
    fn check(mut self) -> Result<(), &'static str> {
        if self.body.blocks.is_empty() {
            self.block(Terminator::Unreachable)?;
        }
        let program = Program {
            drops: None,
            bodies: vec![self.body],
            region_equalities: Vec::new(),
            coercions: Vec::new(),
            outlives: Vec::new(),
            body_regions: Vec::new(),
        };
        ownership::verify(
            &self.types,
            &self.definitions,
            &HashMap::default(),
            &mut Copies::default(),
            &program,
        )
        .map(|_| ())
        .map_err(|e| e.message)
    }
}
#[test]
fn views_protect_source_places_until_the_last_guard_use() -> Result<(), &'static str> {
    for (final_use, read_after, accepted) in [
        (true, false, false),
        (false, true, false),
        (false, false, true),
    ] {
        let mut f = Fixture::new();
        let input = Place::Local(f.local(BYTE, LocalKind::Parameter(0))?);
        let (guard, view) = f.guard(&[], Some((BYTE, input)))?;
        f.write(input)?;
        if read_after {
            f.read(Place::Local(view.ok_or("view")?), BYTE)?;
        }
        if final_use {
            f.action(Action::GuardUse(guard));
        }
        assert_eq!(f.check().is_ok(), accepted);
    }
    let mut f = Fixture::new();
    let input = Place::Local(f.local(BYTE, LocalKind::Parameter(0))?);
    let (_, view) = f.guard(&[], Some((BYTE, input)))?;
    f.read(Place::Local(view.ok_or("view")?), BYTE)?;
    f.write(input)?;
    f.check()
}
#[test]
fn shallow_tests_allow_payload_writes_and_protect_ancestors() -> Result<(), &'static str> {
    for (protect_field, write_field, accepted) in [
        (false, true, true),
        (false, false, false),
        (true, false, false),
        (true, true, true),
    ] {
        let mut f = Fixture::new();
        let tuple = f.tuple()?;
        let input = Place::Local(f.local(tuple, LocalKind::Parameter(0))?);
        let left = f.field(input, 0)?;
        let right = f.field(input, 1)?;
        let (guard, _) = f.guard(&[if protect_field { left } else { input }], None)?;
        f.write(if write_field { right } else { input })?;
        f.action(Action::GuardUse(guard));
        assert_eq!(f.check().is_ok(), accepted);
    }
    Ok(())
}
#[test]
fn views_require_initialization_and_disallow_moves_or_writes() -> Result<(), &'static str> {
    let mut f = Fixture::new();
    let input = Place::Local(f.local(BYTE, LocalKind::Binding)?);
    f.guard(&[], Some((BYTE, input)))?;
    assert_eq!(
        f.check(),
        Err("guard binding may be uninitialized or moved")
    );
    let mut f = Fixture::new();
    let ty = f.noncopy()?;
    let input = Place::Local(f.local(ty, LocalKind::Parameter(0))?);
    let (_, view) = f.guard(&[], Some((ty, input)))?;
    f.read(Place::Local(view.ok_or("view")?), ty)?;
    assert_eq!(
        f.check(),
        Err("cannot move a non-Copy value out of a guard binding")
    );
    let mut f = Fixture::new();
    let input = Place::Local(f.local(BYTE, LocalKind::Parameter(0))?);
    let (_, view) = f.guard(&[], Some((BYTE, input)))?;
    f.write(Place::Local(view.ok_or("view")?))?;
    assert_eq!(f.check(), Err("cannot assign through a guard binding"));
    Ok(())
}
#[test]
fn nested_views_retain_projected_physical_identity() -> Result<(), &'static str> {
    for (write_other, tested_only, accepted) in [
        (false, false, false),
        (true, false, true),
        (false, true, false),
        (true, true, true),
    ] {
        let mut f = Fixture::new();
        let tuple = f.tuple()?;
        let input = Place::Local(f.local(tuple, LocalKind::Parameter(0))?);
        let left = f.field(input, 0)?;
        let right = f.field(input, 1)?;
        let (_, outer) = f.guard(&[], Some((tuple, input)))?;
        let field = f.field(Place::Local(outer.ok_or("view")?), 0)?;
        let (inner, _) = if tested_only {
            f.guard(&[field], None)?
        } else {
            f.guard(&[], Some((BYTE, field)))?
        };
        f.write(if write_other { right } else { left })?;
        f.action(Action::GuardUse(inner));
        assert_eq!(f.check().is_ok(), accepted);
    }
    Ok(())
}
#[test]
fn liveness_joins_paths_without_protecting_diverging_mutations() -> Result<(), &'static str> {
    let mut f = Fixture::new();
    let input = Place::Local(f.local(BYTE, LocalKind::Parameter(0))?);
    let (guard, _) = f.guard(&[input], None)?;
    f.block(Terminator::Branch {
        condition: ValueId(0),
        yes: BlockId(1),
        no: BlockId(2),
    })?;
    f.write(input)?;
    f.block(Terminator::Unreachable)?;
    f.action(Action::GuardUse(guard));
    f.block(Terminator::Unreachable)?;
    f.check()?;
    let mut f = Fixture::new();
    let input = Place::Local(f.local(BYTE, LocalKind::Parameter(0))?);
    let (guard, _) = f.guard(&[input], None)?;
    f.block(Terminator::Jump(BlockId(1)))?;
    f.write(input)?;
    f.block(Terminator::Branch {
        condition: ValueId(0),
        yes: BlockId(1),
        no: BlockId(2),
    })?;
    f.action(Action::GuardUse(guard));
    f.block(Terminator::Unreachable)?;
    assert!(f.check().is_err());
    Ok(())
}
#[test]
fn array_view_ranges_keep_disjoint_and_overlapping_elements_distinct() -> Result<(), &'static str> {
    for (index, accepted) in [(0, true), (1, false), (2, false), (3, true)] {
        let mut f = Fixture::new();
        let array = f.types.intern(Kind::Array {
            element: BYTE,
            length: 4,
        })?;
        let middle = f.types.intern(Kind::Array {
            element: BYTE,
            length: 2,
        })?;
        let input = Place::Local(f.local(array, LocalKind::Parameter(0))?);
        let source = f.project(input, ProjectionKind::Subarray { start: 1, end: 3 }, middle)?;
        let (guard, _) = f.guard(&[], Some((middle, source)))?;
        let target = f.project(input, ProjectionKind::ConstantIndex(index), BYTE)?;
        f.write(target)?;
        f.action(Action::GuardUse(guard));
        assert_eq!(f.check().is_ok(), accepted);
    }
    Ok(())
}

#[path = "guard_views/capture_permissions.rs"]
mod capture_permissions;
