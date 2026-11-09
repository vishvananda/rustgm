//! Iterative ownership-aware type recipes finish children before their parents.
use super::{Context, Declarations, Key, Kind, Stage, TypeId, g, retained, s, signatures};

#[derive(Clone, Copy)]
enum Task {
    Enter(Key, Context),
    Reference(Key, Context),
    Nominal(Key, Context),
    Function(Key, Context, signatures::Parts),
    Tuple(Key, Context, crate::types::ListId),
}

impl Stage {
    pub(crate) fn ty(
        &mut self,
        facts: &Declarations,
        ty: TypeId,
        context: Context,
    ) -> Result<g::TypeRef, &'static str> {
        self.graph.roots = self.graph.roots.saturating_add(1);
        let root = self.type_key(facts, ty, context)?;
        if let Some(&completed) = self.graph.types.get(&root) {
            self.graph.hits = self.graph.hits.saturating_add(1);
            return Ok(completed);
        }
        let mut pending = Vec::new();
        self.type_work(&mut pending, Task::Enter(root, context))?;
        while let Some(task) = pending.pop() {
            self.charge(1)?;
            let (key, context) = match task {
                Task::Enter(key, context)
                | Task::Reference(key, context)
                | Task::Nominal(key, context)
                | Task::Function(key, context, _)
                | Task::Tuple(key, context, _) => (key, context),
            };
            if self.graph.types.contains_key(&key) {
                self.graph.hits = self.graph.hits.saturating_add(1);
                continue;
            }
            let kind = facts
                .types
                .kind(key.ty)
                .ok_or("source type graph has an unknown child")?;
            let row = match task {
                Task::Enter(_, _) => match kind {
                    Kind::Reference {
                        referent: child, ..
                    }
                    | Kind::Pointer { pointee: child, .. }
                    | Kind::Slice(child)
                    | Kind::Array { element: child, .. }
                    | Kind::ArrayTerm { element: child, .. }
                    | Kind::Pattern { base: child, .. }
                        if matches!(kind, Kind::Reference { .. }) || self.body_nodes() =>
                    {
                        // A pointer, slice, array or pattern row, like a
                        // reference, follows its one completed child.
                        self.type_work(&mut pending, Task::Reference(key, context))?;
                        let child = self.type_key(facts, child, context)?;
                        self.type_work(&mut pending, Task::Enter(child, context))?;
                        continue;
                    }
                    Kind::Adt(_, arguments) if self.nominal_transport() => {
                        self.type_work(&mut pending, Task::Nominal(key, context))?;
                        let arguments = facts
                            .types
                            .arguments(arguments)
                            .ok_or("source nominal argument list is unknown")?;
                        for argument in arguments {
                            if let crate::types::arguments::Argument::Type(ty) = argument {
                                let child = self.type_key(facts, ty, context)?;
                                self.type_work(&mut pending, Task::Enter(child, context))?;
                            }
                        }
                        continue;
                    }
                    Kind::Function { .. } => {
                        self.enter_function(facts, key, context, &mut pending)?;
                        continue;
                    }
                    Kind::Tuple(elements) if key.ty != TypeId::UNIT && self.body_nodes() => {
                        self.type_work(&mut pending, Task::Tuple(key, context, elements))?;
                        let elements = facts
                            .types
                            .list(elements)
                            .ok_or("source tuple element list is unknown")?;
                        for &element in elements.iter().rev() {
                            let child = self.type_key(facts, element, context)?;
                            self.type_work(&mut pending, Task::Enter(child, context))?;
                        }
                        continue;
                    }
                    _ => self.type_row(facts, key, kind, context)?,
                },
                Task::Reference(_, _) | Task::Nominal(_, _) => {
                    self.type_row(facts, key, kind, context)?
                }
                Task::Function(_, _, parts) => s::Type::Function(self.signature_roots(
                    facts,
                    parts,
                    context,
                    signatures::Roots::Completed,
                )?),
                Task::Tuple(_, _, elements) => self.tuple_row(facts, elements, context)?,
            };
            self.reserve(4 * size_of::<s::Type>() + retained::new_map::<Key, g::TypeRef>(1))?;
            let id = g::TypeRef(
                u32::try_from(self.rows.types.len()).map_err(|_| "source type table overflow")?,
            );
            self.rows.types.push(row);
            self.graph.types.insert(key, id);
            if key.context.is_some() {
                self.graph.contextual_nodes = self.graph.contextual_nodes.saturating_add(1);
            }
        }
        self.graph
            .types
            .get(&root)
            .copied()
            .ok_or("source type root did not complete")
    }

    /// Completed element recipes become one contiguous run of `roots`.
    fn tuple_row(
        &mut self,
        facts: &Declarations,
        elements: crate::types::ListId,
        context: Context,
    ) -> Result<s::Type, &'static str> {
        let elements = facts
            .types
            .list(elements)
            .ok_or("source tuple element list is unknown")?;
        self.reserve(elements.len().saturating_mul(4 * size_of::<g::TypeRef>()))?;
        let start =
            u32::try_from(self.rows.roots.len()).map_err(|_| "source tuple roots overflow")?;
        for &element in elements {
            self.charge(1)?;
            let child = self.type_key(facts, element, context)?;
            let completed = self
                .graph
                .types
                .get(&child)
                .copied()
                .ok_or("source tuple element was not completed before its tuple")?;
            self.rows.roots.push(completed);
        }
        let end =
            u32::try_from(self.rows.roots.len()).map_err(|_| "source tuple roots overflow")?;
        Ok(s::Type::Tuple(g::Range { start, end }))
    }

    fn type_work(&mut self, pending: &mut Vec<Task>, value: Task) -> Result<(), &'static str> {
        self.charge(1)?;
        if pending.len() == pending.capacity() {
            self.scratch(
                pending
                    .len()
                    .saturating_add(1)
                    .saturating_mul(4)
                    .saturating_mul(size_of::<Task>()),
            )?;
        }
        pending.push(value);
        Ok(())
    }

    fn enter_function(
        &mut self,
        facts: &Declarations,
        key: Key,
        parent: Context,
        pending: &mut Vec<Task>,
    ) -> Result<(), &'static str> {
        let parts = signatures::Parts::actual(facts, key.ty)?;
        if parts.bound_regions != 0
            || !parts.ordinary()
            || facts.types.needs_region_context(key.ty) != Some(false)
        {
            return Err("source anonymous callable requires its supported closed Rust frame");
        }
        self.bind_type_arena(facts)?;
        let context = self.frame(
            parent.owner,
            None,
            s::BinderKind::CallableType,
            parts.bound_regions,
        )?;
        self.type_work(pending, Task::Function(key, context, parts))?;
        let result = self.type_key(facts, parts.result, context)?;
        self.type_work(pending, Task::Enter(result, context))?;
        let parameters = facts
            .types
            .list(parts.parameters)
            .ok_or("source callable input list is unknown")?;
        for &parameter in parameters.iter().rev() {
            let child = self.type_key(facts, parameter, context)?;
            self.type_work(pending, Task::Enter(child, context))?;
        }
        Ok(())
    }

    fn type_row(
        &mut self,
        facts: &Declarations,
        key: Key,
        kind: Kind,
        context: Context,
    ) -> Result<s::Type, &'static str> {
        Ok(match kind {
            Kind::Projection(application) if self.compiler_operations || self.body_nodes() => {
                self.projection_recipe(facts, application, context)?
            }
            Kind::Opaque(id, captures) if self.compiler_operations => {
                self.opaque_future(facts, id, captures, context)?
            }
            Kind::Adt(item, arguments) if self.nominal_transport() => s::Type::Nominal {
                declaration: self.address(crate::resolve::model::DefId(item.0))?,
                arguments: self.arguments(facts, arguments, context)?,
            },
            Kind::Parameter(parameter) => s::Type::Parameter(
                *self
                    .graph
                    .parameters
                    .get(&parameter)
                    .ok_or("source type uses an unreserved actual parameter slot")?,
            ),
            Kind::Reference {
                mutable,
                region,
                referent,
            } => s::Type::Reference {
                mutable,
                region: self.region(region, context)?,
                referent: self.completed_child(facts, referent, context)?,
            },
            Kind::FunctionItem {
                item, arguments, ..
            } => s::Type::FunctionItem(self.function_application(facts, item, arguments, context)?),
            Kind::Pointer { mutable, pointee } if self.body_nodes() => s::Type::RawPointer {
                mutable,
                pointee: self.completed_child(facts, pointee, context)?,
            },
            Kind::Slice(element) if self.body_nodes() => {
                s::Type::Slice(self.completed_child(facts, element, context)?)
            }
            Kind::Pattern { base, constraint } if self.body_nodes() => {
                self.pattern_row(facts, base, constraint, context)?
            }
            Kind::Array { element, length } if self.body_nodes() => {
                let element = self.completed_child(facts, element, context)?;
                let usize = TypeId::primitive(crate::types::Primitive::Usize);
                let ty = self.ty(facts, usize, context)?;
                let bits = u128::from(length);
                self.array_row(element, s::ConstTerm::Value(s::ConstValue { ty, bits }))?
            }
            Kind::ArrayTerm { element, length } if self.body_nodes() => {
                let element = self.completed_child(facts, element, context)?;
                let term = self.const_term(facts, length, context)?;
                self.array_row(element, term)?
            }
            Kind::Primitive(crate::types::Primitive::Str) if self.body_nodes() => s::Type::Str,
            Kind::Never if self.body_nodes() => s::Type::Never,
            Kind::Primitive(_) | Kind::Tuple(_)
                if key.ty == TypeId::UNIT || matches!(kind, Kind::Primitive(_)) =>
            {
                s::Type::Scalar(
                    crate::sema::crates::export_scalar(facts, key.ty)
                        .map_err(|_| "source scalar type is outside the supported profile")?,
                )
            }
            _ => return Err("source type form requires an explicitly supported transport row"),
        })
    }

    /// Version thirteen: a pattern row over its completed base. A range's
    /// canonical endpoints are appended in type-row order. A non-null pattern
    /// over a type parameter is fenced: only core's `DispatchFromDyn` impl
    /// header names one, and no export reaches it yet.
    fn pattern_row(
        &mut self,
        facts: &Declarations,
        base: TypeId,
        constraint: crate::types::Constraint,
        context: Context,
    ) -> Result<s::Type, &'static str> {
        let constraint = match constraint {
            crate::types::Constraint::NonNull => {
                match facts.types.kind(base) {
                    Some(Kind::Pointer { .. }) => {}
                    Some(Kind::Parameter(_)) => {
                        return Err(
                            "source non-null pattern over a type parameter is not transported",
                        );
                    }
                    _ => return Err("source non-null pattern requires a raw pointer base"),
                }
                s::PatternConstraint::NonNull
            }
            crate::types::Constraint::ScalarRange(id) => {
                let range = facts
                    .types
                    .scalar_range(id)
                    .filter(|range| range.base == base)
                    .ok_or("source scalar pattern lost its canonical interval")?;
                self.reserve(4 * size_of::<s::PatternRange>())?;
                let ranges = &mut self.cold()?.pattern_ranges;
                let reference = s::PatternRangeRef(
                    u32::try_from(ranges.len()).map_err(|_| "source pattern range overflow")?,
                );
                ranges.push(s::PatternRange {
                    start: range.start,
                    end: range.end,
                });
                s::PatternConstraint::Range(reference)
            }
        };
        Ok(s::Type::Pattern {
            base: self.completed_child(facts, base, context)?,
            constraint,
        })
    }

    /// Version thirteen: an array row over its completed element. Its length
    /// term is appended in type-row order, after any row the term's own type
    /// needed, so the row that follows is the one that names it.
    fn array_row(
        &mut self,
        element: g::TypeRef,
        length: s::ConstTerm,
    ) -> Result<s::Type, &'static str> {
        self.reserve(4 * size_of::<s::ConstTerm>())?;
        let lengths = &mut self.cold()?.array_lengths;
        let reference = s::ArrayLengthRef(
            u32::try_from(lengths.len()).map_err(|_| "source array length overflow")?,
        );
        lengths.push(length);
        Ok(s::Type::Array {
            element,
            length: reference,
        })
    }

    /// The single child of a reference, pointer, slice or array row, which
    /// the iterative walk completed before its parent.
    fn completed_child(
        &mut self,
        facts: &Declarations,
        child: TypeId,
        context: Context,
    ) -> Result<g::TypeRef, &'static str> {
        let child = self.type_key(facts, child, context)?;
        self.graph
            .types
            .get(&child)
            .copied()
            .ok_or("source referent was not completed before its parent")
    }
}
