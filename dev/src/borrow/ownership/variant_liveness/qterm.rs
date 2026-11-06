//! Value qualification expressions are indexed once; they confer no place permission.
use super::{
    parts::{Parts, Range, compact},
    tags::{Coordinate, Tags},
};
use crate::{
    body::ir::{Action, Body, LocalId, LocalKind, Place, Rvalue, ValueId, ValueList},
    borrow::ownership::{Paths, paths::Access},
    support::{hash::HashMap, retained::Growth},
    types::{Kind, Store, TypeId},
};
#[derive(Clone, Copy)]
pub(super) struct Input<'a> {
    pub types: &'a Store,
    pub paths: &'a Paths,
    pub body: &'a Body,
}
#[derive(Clone, Copy)]
pub(super) enum Source {
    Clear,
    Type(TypeId),
    Place(Place),
    Value(ValueId),
    Fields(ValueList),
}
#[derive(Clone, Copy)]
pub(super) enum Term {
    Read(Coordinate),
    /// A past unmaterialized value may qualify, filtered by its actual projection type.
    Projected(Coordinate),
    Type(TypeId),
    Parameter {
        root: TypeId,
        projected: TypeId,
    },
    Unknown,
}
#[derive(Clone, Copy)]
enum Initializer {
    One(ValueId),
    Several,
}
#[derive(Default)]
pub(super) struct Aliases {
    initializers: HashMap<LocalId, Initializer>,
}
impl Aliases {
    pub(super) fn observe(
        &mut self,
        parts: &Parts,
        input: Input<'_>,
        action: Action,
    ) -> Result<(), &'static str> {
        let Action::Store {
            place: Place::Local(id),
            value,
        } = action
        else {
            return Ok(());
        };
        let local = input
            .body
            .locals
            .get(id.0 as usize)
            .ok_or("qualification alias local is missing")?;
        if local.mutable
            || !matches!(
                input.types.kind(local.ty),
                Some(Kind::Reference { .. } | Kind::Pointer { .. })
            )
        {
            return Ok(());
        }
        parts.pay(Growth::map::<LocalId, Initializer>(
            self.initializers.len(),
            self.initializers.capacity(),
            1,
        ))?;
        self.initializers
            .try_reserve(1)
            .map_err(|_| "qualification alias index allocation failed")?;
        self.initializers
            .entry(id)
            .and_modify(|v| *v = Initializer::Several)
            .or_insert(Initializer::One(value));
        Ok(())
    }
}
#[derive(Clone, Copy)]
enum State {
    Pending,
    Ready(Term),
}
#[derive(Default)]
pub(super) struct Compiler {
    states: HashMap<ValueId, State>,
    work: Vec<ValueId>,
}
#[derive(Default)]
pub(super) struct Terms {
    pub values: Vec<Term>,
}
impl Terms {
    pub(super) fn compile(
        &mut self,
        tags: &Tags,
        input: Input<'_>,
        aliases: &Aliases,
        compiler: &mut Compiler,
        source: Source,
    ) -> Result<Range, &'static str> {
        let start = compact(self.values.len())?;
        match source {
            Source::Clear => {}
            Source::Type(ty) => self.push(&tags.parts, Term::Type(ty))?,
            Source::Place(place) => {
                self.push(&tags.parts, Compiler::place(tags, input, place, true)?)?;
            }
            Source::Value(value) => {
                self.push(&tags.parts, compiler.operand(tags, input, aliases, value)?)?;
            }
            Source::Fields(fields) => {
                for &value in input
                    .body
                    .arguments
                    .get(fields.start as usize..fields.end as usize)
                    .ok_or("qualification constructor arguments are missing")?
                {
                    self.push(&tags.parts, compiler.operand(tags, input, aliases, value)?)?;
                }
            }
        }
        Ok(Range {
            start,
            end: compact(self.values.len())?,
        })
    }
    fn push(&mut self, parts: &Parts, term: Term) -> Result<(), &'static str> {
        parts.pay(Growth::vector::<Term>(
            self.values.len(),
            self.values.capacity(),
            1,
        ))?;
        self.values
            .try_reserve(1)
            .map_err(|_| "qualification expression allocation failed")?;
        self.values.push(term);
        Ok(())
    }
}
impl Compiler {
    fn operand(
        &mut self,
        tags: &Tags,
        input: Input<'_>,
        aliases: &Aliases,
        value: ValueId,
    ) -> Result<Term, &'static str> {
        if let Some(root) = input
            .paths
            .temporary_path(value)
            .and_then(|p| tags.locations.get(&p))
        {
            return Ok(Term::Read(root.coordinate));
        }
        self.untracked(tags, input, aliases, value)
    }
    fn untracked(
        &mut self,
        tags: &Tags,
        input: Input<'_>,
        aliases: &Aliases,
        mut value: ValueId,
    ) -> Result<Term, &'static str> {
        let term = loop {
            tags.parts.charge(1)?;
            match self.states.get(&value) {
                Some(State::Ready(term)) => break *term,
                Some(State::Pending) => return Err("cyclic qualification value dependency"),
                None => {}
            }
            tags.parts.pay(Growth::map::<ValueId, State>(
                self.states.len(),
                self.states.capacity(),
                1,
            ))?;
            tags.parts.pay(Growth::vector::<ValueId>(
                self.work.len(),
                self.work.capacity(),
                1,
            ))?;
            self.states
                .try_reserve(1)
                .map_err(|_| "qualification value index allocation failed")?;
            self.work
                .try_reserve(1)
                .map_err(|_| "qualification value frontier allocation failed")?;
            self.states.insert(value, State::Pending);
            self.work.push(value);
            let source = input
                .body
                .values
                .get(value.0 as usize)
                .ok_or("qualification operand is missing")?;
            match expression(tags, input, aliases, *source)? {
                Step::Next(next) => value = next,
                Step::Ready(term) => break term,
            }
        };
        while let Some(value) = self.work.pop() {
            self.states.insert(value, State::Ready(term));
        }
        Ok(term)
    }
    fn place(
        tags: &Tags,
        input: Input<'_>,
        place: Place,
        current: bool,
    ) -> Result<Term, &'static str> {
        let ty = input
            .body
            .place_type(place)
            .ok_or("qualification read type is missing")?;
        let info = input.paths.place(place)?;
        if current && let Some(located) = tags.place(input.paths, place)? {
            return Ok(Term::Read(located.coordinate));
        }
        if info.access != Access::Owned {
            return Ok(Term::Type(ty));
        }
        let local = info
            .path
            .and_then(|p| input.paths.entries.get(p.0 as usize))
            .and_then(|p| {
                if let crate::borrow::ownership::Root::Local(id) = p.root {
                    input.body.locals.get(id.0 as usize)
                } else {
                    None
                }
            });
        if let Some(local) =
            local.filter(|l| !l.mutable && matches!(l.kind, LocalKind::Parameter(_)))
        {
            return Ok(Term::Parameter {
                root: local.ty,
                projected: ty,
            });
        }
        if let Some(located) = tags.place(input.paths, place)? {
            return Ok(Term::Projected(located.coordinate));
        }
        Ok(Term::Unknown)
    }
}

enum Step {
    Next(ValueId),
    Ready(Term),
}
fn expression(
    tags: &Tags,
    input: Input<'_>,
    aliases: &Aliases,
    source: crate::body::ir::Value,
) -> Result<Step, &'static str> {
    let term =
        match source.kind {
            Rvalue::Unit | Rvalue::Scalar(_) | Rvalue::Text(_) | Rvalue::Function(_) => {
                Term::Type(TypeId::UNIT)
            }
            Rvalue::Constrain(next) | Rvalue::Reify(next) => return Ok(Step::Next(next)),
            Rvalue::Read(place) => return place_expression(tags, input, aliases, place),
            Rvalue::Borrow { place, .. } | Rvalue::Address(place) => {
                if let Place::Projection(id) = place {
                    let projection = input
                        .body
                        .projections
                        .get(id.0 as usize)
                        .ok_or("qualification reborrow projection is missing")?;
                    if matches!(
                        projection.kind,
                        crate::body::ir::ProjectionKind::Dereference { .. }
                    ) && input.body.place_type(projection.base).is_some_and(|ty| {
                        matches!(input.types.kind(ty), Some(Kind::Reference { .. }))
                    }) {
                        return place_expression(tags, input, aliases, projection.base);
                    }
                }
                Term::Unknown
            }
            Rvalue::Call { .. } if !matches!(input.types.kind(source.ty), Some(Kind::Adt(..))) => {
                Term::Type(source.ty)
            }
            _ => Term::Unknown,
        };
    Ok(Step::Ready(term))
}
fn place_expression(
    tags: &Tags,
    input: Input<'_>,
    aliases: &Aliases,
    place: Place,
) -> Result<Step, &'static str> {
    if let Place::Local(id) = place
        && let Some(initializer) = aliases.initializers.get(&id)
    {
        return Ok(match initializer {
            Initializer::One(next) => Step::Next(*next),
            Initializer::Several => Step::Ready(Term::Unknown),
        });
    }
    Ok(Step::Ready(Compiler::place(tags, input, place, false)?))
}

impl Terms {
    pub(super) fn evaluate<I: crate::borrow::ownership::initialization::Maybe>(
        &self,
        range: Range,
        tags: &Tags,
        queries: &super::query::Queries,
        state: &mut crate::borrow::ownership::state::State<I>,
    ) -> Result<bool, &'static str> {
        let mut result = false;
        for term in self
            .values
            .get(range.start as usize..range.end as usize)
            .ok_or("qualification expression extent changed")?
        {
            state.charge(1)?;
            result |= match *term {
                Term::Read(coordinate) => {
                    let node = tags
                        .nodes
                        .get(coordinate.0 as usize)
                        .ok_or("qualification source coordinate is missing")?;
                    let root = tags
                        .roots
                        .get(node.root as usize)
                        .ok_or("qualification source root is missing")?;
                    queries.mask(coordinate)? && state.possible(root.bad)?
                }
                Term::Projected(coordinate) => queries.mask(coordinate)?,
                Term::Type(ty) => queries.may(ty)?,
                Term::Parameter { root, projected } => {
                    queries.may(root)? && queries.may(projected)?
                }
                Term::Unknown => true,
            };
        }
        Ok(result)
    }
}
