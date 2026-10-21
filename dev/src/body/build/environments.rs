//! Logical captured bindings project into the owning closure's physical fields.
use super::{Builder, Error, LocalKind, NodeId, NodeKey, Owner, Place, Type, TypeId};
use crate::{
    body::ir::{LocalId, ProjectionKind, Selection},
    sema::closures::{
        Environment,
        captures::{Capture, Mode, PathId, Step},
    },
};
pub(super) struct Places {
    fields: Vec<Place>,
    mentions: std::collections::HashSet<PathId>,
    pub(super) validated: std::collections::HashSet<(PathId, PathId, u32)>,
}
pub(super) enum Access {
    Ordinary,
    Uncaptured,
    Place(Place),
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Formation {
    Runtime,
    Initialization,
}
impl Builder<'_> {
    pub(super) fn capture_environment(&self) -> Option<&Environment> {
        let Owner::Closure { identity, .. } = self.body.owner else {
            return None;
        };
        self.facts.closures.environment(identity)
    }
    pub(super) fn has_captured_place(&self, node: NodeId) -> bool {
        self.capture_environment().is_some_and(|environment| {
            environment.accesses.contains_key(&NodeKey {
                node,
                ..self.body.source
            })
        })
    }
    pub(super) fn prepare_capture_receiver(&mut self) -> Result<(), Error> {
        let Owner::Closure {
            identity,
            execution: signature,
            ..
        } = self.body.owner
        else {
            return Ok(());
        };
        let facts = self.facts;
        let source = facts
            .closures
            .identity(identity)
            .ok_or_else(|| self.error(self.body.source.node, "missing captured closure body"))?;
        let Some(environment) = facts.closures.environment(identity) else {
            return if signature.is_none() {
                Ok(())
            } else {
                Err(self.error(source.body.node, "missing captured environment plan"))
            };
        };
        let mentions = environment.mentions.iter().copied().collect();
        let Some(signature) = signature else {
            if !environment.mentions.is_empty() {
                self.capture_places = Some(Box::new(Places {
                    fields: Vec::new(),
                    mentions,
                    validated: std::collections::HashSet::new(),
                }));
            }
            return Ok(());
        };
        let execution = crate::body::invocation_entries::validate_execution(&facts.types, *source)
            .map_err(|message| self.error(source.body.node, message))?;
        if execution.signature != signature {
            return Err(self.error(
                source.body.node,
                "closure execution signature differs from source contract",
            ));
        }
        let base = self.capture_receiver(source.body.node, execution)?;
        let Some(Type::Closure {
            identity: selected,
            environment: stored,
            ..
        }) = facts.types.kind(execution.self_ty)
        else {
            return Err(self.error(
                source.body.node,
                "execution receiver has no nominal closure Self",
            ));
        };
        if selected != identity {
            return Err(self.error(
                source.body.node,
                "execution receiver selects a different closure",
            ));
        }
        let Some(Type::Tuple(fields)) = facts.types.kind(stored) else {
            return Err(self.error(source.body.node, "closure environment is not a tuple"));
        };
        let fields = facts
            .types
            .list(fields)
            .ok_or_else(|| self.error(source.body.node, "missing closure environment fields"))?;
        if fields != environment.field_types || fields.len() != environment.plan.captures().len() {
            return Err(self.error(
                source.body.node,
                "closure environment field coverage differs from capture plan",
            ));
        }
        let mut places = Vec::with_capacity(fields.len());
        for (index, (&ty, capture)) in fields.iter().zip(environment.plan.captures()).enumerate() {
            places.push(self.capture_prefix(
                source.body.node,
                environment,
                *capture,
                (index, ty),
                base,
            )?);
        }
        self.capture_places = Some(Box::new(Places {
            fields: places,
            mentions,
            validated: std::collections::HashSet::new(),
        }));
        Ok(())
    }
    fn capture_receiver(
        &mut self,
        node: NodeId,
        execution: crate::sema::closures::Execution,
    ) -> Result<Place, Error> {
        let Some(Type::Function { parameters, .. }) = self.facts.types.kind(execution.signature)
        else {
            return Err(self.error(node, "missing execution function signature"));
        };
        let receiver = *self
            .facts
            .types
            .list(parameters)
            .and_then(|list| list.first())
            .ok_or_else(|| self.error(node, "missing hidden closure receiver"))?;
        let local = self.local(node, receiver, LocalKind::Parameter(0), None, true)?;
        if local != LocalId(0) {
            return Err(self.error(node, "hidden closure receiver is not first"));
        }
        if receiver == execution.self_ty {
            Ok(Place::Local(local))
        } else {
            let base = self.dereference_place(node, Place::Local(local))?;
            if self.place_type(node, base)? != execution.self_ty {
                return Err(self.error(node, "hidden receiver differs from canonical closure Self"));
            }
            Ok(base)
        }
    }
    fn capture_prefix(
        &mut self,
        node: NodeId,
        environment: &Environment,
        capture: Capture,
        field: (usize, TypeId),
        base: Place,
    ) -> Result<Place, Error> {
        let (index, ty) = field;
        let index = self.index(node, index)?;
        let mut place = self.project(
            node,
            ty,
            base,
            ProjectionKind::Field(Selection::Tuple(index)),
        )?;
        if capture.mode != Mode::Value {
            place = self.dereference_place(node, place)?;
        }
        let logical = *environment
            .path_types
            .get(capture.path.index())
            .ok_or_else(|| self.error(node, "missing capture path type"))?;
        if self.place_type(node, place)? != logical {
            return Err(self.error(node, "stored capture differs from its logical binding"));
        }
        let mutable = self.capture_binding_mutability(node, environment, capture.path)?;
        self.project(
            node,
            logical,
            place,
            ProjectionKind::CaptureBoundary { mutable },
        )
    }
    fn capture_binding_mutability(
        &self,
        node: NodeId,
        environment: &Environment,
        path: PathId,
    ) -> Result<bool, Error> {
        let mut current = path;
        let mut remaining = environment.plan.path_count();
        while let Some((parent, step)) = environment.plan.parent(current) {
            remaining = remaining
                .checked_sub(1)
                .ok_or_else(|| self.error(node, "capture binding ancestry exceeds path forest"))?;
            // Only a source dereference clears binding replacement restrictions.
            // The stored capture reference was handled before this logical boundary.
            if matches!(step, Step::Dereference { .. }) {
                return Ok(true);
            }
            current = parent;
        }
        let root = environment
            .plan
            .root(current)
            .ok_or_else(|| self.error(node, "missing capture source binding"))?;
        environment
            .binding_mutability
            .get(&root)
            .copied()
            .ok_or_else(|| self.error(node, "missing capture binding authority"))
    }
    pub(super) fn captured_place(&mut self, node: NodeId) -> Result<Access, Error> {
        let facts = self.facts;
        let Owner::Closure { identity, .. } = self.body.owner else {
            return Ok(Access::Ordinary);
        };
        let Some(environment) = facts.closures.environment(identity) else {
            return Ok(Access::Ordinary);
        };
        let Some(&path) = environment.accesses.get(&NodeKey {
            node,
            ..self.body.source
        }) else {
            return Ok(Access::Ordinary);
        };
        self.capture_path_place(node, environment, path)
    }
    pub(super) fn capture_path_place(
        &mut self,
        node: NodeId,
        environment: &Environment,
        path: PathId,
    ) -> Result<Access, Error> {
        self.capture_path_place_for(node, environment, path, Formation::Runtime)
    }
    fn capture_path_place_for(
        &mut self,
        node: NodeId,
        environment: &Environment,
        path: PathId,
        formation: Formation,
    ) -> Result<Access, Error> {
        let Some(index) = environment.plan.covering_field(path) else {
            return if self
                .capture_places
                .as_deref()
                .is_some_and(|places| places.mentions.contains(&path))
            {
                Ok(Access::Uncaptured)
            } else {
                Err(self.error(node, "captured access has no covering environment field"))
            };
        };
        let capture = environment
            .plan
            .captures()
            .get(index as usize)
            .ok_or_else(|| self.error(node, "invalid covering capture field"))?;
        let mut place = self
            .capture_places
            .as_deref()
            .and_then(|places| places.fields.get(index as usize))
            .copied()
            .ok_or_else(|| self.error(node, "missing physical capture prefix"))?;
        let mut suffix = Vec::new();
        let mut current = path;
        while current != capture.path {
            if suffix.len() >= environment.plan.path_count() {
                return Err(self.error(node, "capture suffix exceeds path forest"));
            }
            let (parent, step) = environment
                .plan
                .parent(current)
                .ok_or_else(|| self.error(node, "capture prefix does not cover selected access"))?;
            suffix.push((current, step));
            current = parent;
        }
        for (path, step) in suffix.into_iter().rev() {
            let expected = *environment
                .path_types
                .get(path.index())
                .ok_or_else(|| self.error(node, "missing captured projection type"))?;
            place = self.formation_projection(node, expected, place, step, formation)?;
        }
        Ok(Access::Place(place))
    }
}
impl Builder<'_> {
    fn formation_place(
        &mut self,
        node: NodeId,
        environment: &Environment,
        path: PathId,
        formation: Formation,
    ) -> Result<Option<Place>, Error> {
        let facts = self.facts;
        if let Owner::Closure { identity, .. } = self.body.owner
            && let Some(parent) = facts.closures.environment(identity)
            && let Some(&mapped) = parent.nested_accesses.get(&(
                NodeKey {
                    node,
                    ..self.body.source
                },
                path,
            ))
        {
            let expected = environment
                .formation_types
                .get(path.index())
                .copied()
                .ok_or_else(|| self.error(node, "missing nested capture formation type"))?;
            if parent.path_types.get(mapped.index()).copied() != Some(expected) {
                return Err(
                    self.error(node, "nested capture mapping differs from parent path type")
                );
            }
            return match self.capture_path_place_for(node, parent, mapped, formation)? {
                Access::Place(place) if self.place_type(node, place)? == expected => {
                    Ok(Some(place))
                }
                Access::Uncaptured if formation == Formation::Initialization => Ok(None),
                _ => Err(self.error(
                    node,
                    "nested capture mapping has no matching physical place",
                )),
            };
        }
        let mut suffix = Vec::new();
        let mut current = path;
        while let Some((parent, step)) = environment.plan.parent(current) {
            if suffix.len() >= environment.plan.path_count() {
                return Err(self.error(node, "capture formation exceeds path forest"));
            }
            suffix.push((current, step));
            current = parent;
        }
        let root = environment
            .plan
            .root(current)
            .ok_or_else(|| self.error(node, "missing capture formation root"))?;
        let mut place = self
            .scratch
            .definitions
            .get(root.0 as usize)
            .copied()
            .flatten()
            .map(Place::Local)
            .ok_or_else(|| {
                self.error(
                    node,
                    "nested capture formation requires a mapped outer environment place",
                )
            })?;
        let root_ty = *environment
            .formation_types
            .get(current.index())
            .ok_or_else(|| self.error(node, "missing parent capture root type"))?;
        if self.place_type(node, place)? != root_ty {
            return Err(self.error(node, "parent capture root differs from formation type"));
        }
        for (path, step) in suffix.into_iter().rev() {
            let expected = *environment
                .formation_types
                .get(path.index())
                .ok_or_else(|| self.error(node, "missing parent capture projection type"))?;
            place = self.formation_projection(node, expected, place, step, formation)?;
        }
        Ok(Some(place))
    }
    fn formation_projection(
        &mut self,
        node: NodeId,
        expected: TypeId,
        base: Place,
        step: Step,
        formation: Formation,
    ) -> Result<Place, Error> {
        let projection = match step {
            Step::Field(field) | Step::UnionField(field) => {
                let base_ty = self.place_type(node, base)?;
                let union = match (field, self.facts.types.kind(base_ty)) {
                    (Selection::Tuple(index), Some(Type::Tuple(fields)))
                        if self
                            .facts
                            .types
                            .list(fields)
                            .is_some_and(|fields| (index as usize) < fields.len()) =>
                    {
                        false
                    }
                    (Selection::Nominal(field), Some(Type::Adt(owner, _))) => {
                        let actual = self
                            .facts
                            .members
                            .variant(field)
                            .and_then(|id| self.facts.members.variants.get(id.0 as usize))
                            .map(|variant| variant.owner);
                        if actual != Some(owner) {
                            return Err(self.error(node, "capture projection field owner changed"));
                        }
                        self.facts
                            .aggregates
                            .get(owner)
                            .ok_or_else(|| self.error(node, "missing capture projection shape"))?
                            .kind
                            == crate::types::aggregate::Kind::Union
                    }
                    _ => {
                        return Err(self
                            .error(node, "capture projection field has a different parent type"));
                    }
                };
                if union != matches!(step, Step::UnionField(_)) {
                    return Err(self.error(node, "capture projection field capability changed"));
                }
                self.projection(node, expected, base, ProjectionKind::Field(field))?
            }
            Step::Dereference { mutable } | Step::RawDereference { mutable } => {
                let raw = matches!(step, Step::RawDereference { .. });
                let capability = match self.facts.types.kind(self.place_type(node, base)?) {
                    Some(Type::Pointer {
                        mutable: actual, ..
                    }) if raw => Some(actual),
                    Some(Type::Reference {
                        mutable: actual, ..
                    }) if !raw => Some(actual),
                    _ => None,
                };
                if capability != Some(mutable) {
                    return Err(
                        self.error(node, "capture projection dereference capability changed")
                    );
                }
                self.dereference_projection(node, base)?
            }
        };
        let place = Place::Projection(projection);
        if self.place_type(node, place)? != expected {
            return Err(self.error(
                node,
                "capture formation projection differs from parent type",
            ));
        }
        if formation == Formation::Runtime {
            self.statement(node, super::Action::Project(projection))?;
        }
        Ok(place)
    }
    pub(super) fn environment_value(
        &mut self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<super::ValueId, Error> {
        let facts = self.facts;
        let Some(Type::Closure {
            identity,
            environment: stored,
            ..
        }) = facts.types.kind(ty)
        else {
            return Err(self.error(node, "closure value lacks a canonical closure type"));
        };
        let Some(Type::Tuple(fields)) = facts.types.kind(stored) else {
            return Err(self.error(node, "closure environment is not a tuple"));
        };
        let fields = facts
            .types
            .list(fields)
            .ok_or_else(|| self.error(node, "missing selected environment fields"))?;
        let environment = facts.closures.environment(identity);
        if let Some(environment) = environment {
            if fields.len() != environment.plan.captures().len() {
                return Err(self.error(
                    node,
                    "selected environment field count differs from capture plan",
                ));
            }
            for &path in &environment.mentions {
                if let Some(place) =
                    self.formation_place(node, environment, path, Formation::Initialization)?
                {
                    self.statement(node, super::Action::CheckInitialized(place))?;
                }
            }
        } else if !fields.is_empty() {
            return Err(self.error(node, "missing selected capture environment"));
        }
        let mut values = Vec::with_capacity(fields.len());
        if let Some(environment) = environment {
            for (index, (&field, capture)) in
                fields.iter().zip(environment.plan.captures()).enumerate()
            {
                let actual = *environment
                    .formation_fields
                    .get(index)
                    .ok_or_else(|| self.error(node, "missing capture formation field"))?;
                let place = self
                    .formation_place(node, environment, capture.path, Formation::Runtime)?
                    .ok_or_else(|| {
                        self.error(node, "runtime capture formation requires a physical place")
                    })?;
                let value = if capture.mode == Mode::Value {
                    if self.place_type(node, place)? != actual {
                        return Err(
                            self.error(node, "owned capture differs from selected formation place")
                        );
                    }
                    self.evaluate(node, actual, super::Rvalue::Read(place))?
                } else {
                    let Some(Type::Reference {
                        referent, mutable, ..
                    }) = facts.types.kind(actual)
                    else {
                        return Err(self.error(node, "borrowed capture field is not a reference"));
                    };
                    if referent != self.place_type(node, place)?
                        || mutable != (capture.mode != Mode::Shared)
                    {
                        return Err(self.error(
                            node,
                            "borrowed capture differs from selected formation place",
                        ));
                    }
                    let kind = if capture.mode == Mode::UniqueImmutable {
                        crate::body::ir::BorrowKind::UniqueImmutable
                    } else {
                        crate::body::ir::BorrowKind::Immediate
                    };
                    self.evaluate(node, actual, super::Rvalue::Borrow { place, kind })?
                };
                // Capture publication retains the parent-to-field coercion
                // for the ordinary lifetime proof, including alias equality.
                let value = if actual == field {
                    value
                } else {
                    self.evaluate(node, field, super::Rvalue::Constrain(value))?
                };
                values.push(value);
            }
        }
        let start = self.index(node, self.body.arguments.len())?;
        self.body.arguments.extend(values);
        let end = self.index(node, self.body.arguments.len())?;
        self.evaluate(
            node,
            ty,
            super::Rvalue::Aggregate {
                variant: None,
                fields: super::ValueList { start, end },
            },
        )
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/replay_capabilities.rs"]
mod tests;
