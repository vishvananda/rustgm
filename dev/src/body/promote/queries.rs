//! Bounded DAG queries cache successful and runtime-only values across borrow roots.
use super::{Engine, Error, Failure, Key, Ready, State};
use crate::body::ir::{Place, ProjectionKind, Rvalue, ValueId, ValueList};
impl Engine<'_, '_> {
    pub(super) fn demand(&mut self, root: Key) -> Result<Option<Ready>, Error> {
        self.work.push((root, false));
        while let Some((key, finish)) = self.work.pop() {
            self.charge(1)?;
            match self.states.get(&key) {
                Some(State::Runtime | State::Ready(_)) => continue,
                Some(State::Active) if !finish => {
                    return Err("cycle in promotion value dependencies".into());
                }
                _ => {}
            }
            if finish {
                let value = match self.finish(key) {
                    Ok(value) => value,
                    Err(Error::Evaluation(Failure::Fault(_))) => {
                        let deferred = matches!(self.body.owner, crate::body::ir::Owner::Initializer { definition, .. } if self.evaluator.facts().associated.owner(definition).is_some());
                        deferred.then_some(Ready {
                            value: super::Evaluated::Deferred,
                            immediate: false,
                            constant_assignment: false,
                        })
                    }
                    Err(error) => return Err(error),
                };
                self.states
                    .insert(key, value.map_or(State::Runtime, State::Ready));
            } else {
                self.states.insert(key, State::Active);
                self.work.push((key, true));
                if !self.dependencies(key)? {
                    self.states.insert(key, State::Runtime);
                }
            }
        }
        self.ready(root)
    }
    pub(super) fn ready(&self, key: Key) -> Result<Option<Ready>, Error> {
        match self.states.get(&key) {
            Some(State::Ready(value)) => Ok(Some(*value)),
            Some(State::Runtime) => Ok(None),
            _ => Err("missing completed promotion dependency".into()),
        }
    }
    fn push(&mut self, key: Key) -> Result<(), Error> {
        self.charge(1)?;
        self.work.push((key, false));
        Ok(())
    }
    fn place_dependency(&mut self, place: Place) -> Result<bool, Error> {
        if let Some(key) = Self::place_key(place) {
            self.push(key)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
    fn arguments(&mut self, list: ValueList) -> Result<bool, Error> {
        let arguments = self
            .body
            .arguments
            .get(list.start as usize..list.end as usize)
            .ok_or("invalid promotion argument range")?;
        for &value in arguments.iter().rev() {
            self.push(Key::Value(value))?;
        }
        Ok(true)
    }
    fn dependencies(&mut self, key: Key) -> Result<bool, Error> {
        match key {
            Key::Value(id) => match self.value(id)?.kind {
                Rvalue::Read(place) => self.place_dependency(place),
                Rvalue::Cast {
                    kind: crate::sema::model::Cast::PointerToInteger,
                    ..
                }
                | Rvalue::Invoke { .. }
                | Rvalue::Assembly { .. }
                | Rvalue::AssemblyOutput { .. }
                | Rvalue::Intrinsic { .. }
                | Rvalue::Address(_)
                | Rvalue::IsVariant { .. }
                | Rvalue::Length(_)
                | Rvalue::TextEquals { .. }
                | Rvalue::Validate(_)
                | Rvalue::Union { .. } => Ok(false),
                Rvalue::Borrow { place, .. } => {
                    if !self.borrow_allowed(self.value(id)?.ty)? {
                        return Ok(false);
                    }
                    self.place_dependency(self.reborrow_base(place)?)
                }
                Rvalue::Aggregate { fields, .. } => self.arguments(fields),
                Rvalue::Call {
                    callee, arguments, ..
                } => {
                    if self.constructor(callee)?.is_none() {
                        return Ok(false);
                    }
                    self.push(Key::Value(callee))?;
                    self.arguments(arguments)
                }
                Rvalue::Repeat { .. } if self.empty_repeat(id)? => Ok(true),
                Rvalue::Repeat { value: operand, .. }
                | Rvalue::Constrain(operand)
                | Rvalue::Unsize { value: operand, .. }
                | Rvalue::Reify(operand)
                | Rvalue::Unary { operand, .. }
                | Rvalue::Cast { operand, .. } => {
                    self.push(Key::Value(operand))?;
                    Ok(true)
                }
                Rvalue::Binary { left, right, .. } => {
                    self.push(Key::Value(right))?;
                    self.push(Key::Value(left))?;
                    Ok(true)
                }

                Rvalue::InlineInstance(instance) => {
                    Ok(crate::sema::values::Reader::new(self.evaluator.facts())
                        .inline_instance(instance, self.value(id)?.ty)
                        .is_some())
                }
                Rvalue::InlineConstant(source) => {
                    Ok(crate::sema::values::Reader::new(self.evaluator.facts())
                        .inline(source)
                        .is_some())
                }
                Rvalue::Unit
                | Rvalue::Scalar(_)
                | Rvalue::Text(_)
                | Rvalue::Constant(_)
                | Rvalue::AssociatedConstant(_)
                | Rvalue::SourceAssociated(_)
                | Rvalue::ConstArgument(_)
                | Rvalue::Function(_) => Ok(true),
            },
            Key::Projection(id) => {
                let projection = self.projection(id)?;
                match projection.kind {
                    ProjectionKind::Dereference { .. }
                    | ProjectionKind::UnionField(_)
                    | ProjectionKind::RawDereference { .. }
                    | ProjectionKind::Subarray { .. }
                    | ProjectionKind::Subslice { .. }
                    | ProjectionKind::FromEnd { .. } => Ok(false),
                    ProjectionKind::Index { index, .. }
                    | ProjectionKind::IndexParameter { index, .. }
                    | ProjectionKind::SliceIndex { index } => {
                        self.push(Key::Value(index))?;
                        self.place_dependency(self.projection(id)?.base)
                    }
                    ProjectionKind::CaptureBoundary { .. }
                    | ProjectionKind::ConstantIndex(_)
                    | ProjectionKind::Field(_) => self.place_dependency(projection.base),
                }
            }
        }
    }
    fn finish(&mut self, key: Key) -> Result<Option<Ready>, Error> {
        match key {
            Key::Value(id) => self.finish_value(id),
            Key::Projection(id) => self.finish_projection(id).map(|v| {
                v.map(|value| Ready {
                    value,
                    immediate: false,
                    constant_assignment: false,
                })
            }),
        }
    }
    pub(super) fn value(&self, id: ValueId) -> Result<&crate::body::ir::Value, Error> {
        self.body
            .values
            .get(id.0 as usize)
            .ok_or_else(|| "missing promotion value".into())
    }
}
