//! Iterative ownership-aware type recipes finish children before their parents.
use super::{Context, Declarations, Key, Kind, Stage, TypeId, g, retained, s, signatures};

#[derive(Clone, Copy)]
enum Task {
    Enter(Key, Context),
    Reference(Key, Context),
    Function(Key, Context, signatures::Parts),
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
                | Task::Function(key, context, _) => (key, context),
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
                    Kind::Reference { referent, .. } => {
                        self.type_work(&mut pending, Task::Reference(key, context))?;
                        let child = self.type_key(facts, referent, context)?;
                        self.type_work(&mut pending, Task::Enter(child, context))?;
                        continue;
                    }
                    Kind::Function { .. } => {
                        self.enter_function(facts, key, context, &mut pending)?;
                        continue;
                    }
                    _ => self.type_row(facts, key, kind, context)?,
                },
                Task::Reference(_, _) => self.type_row(facts, key, kind, context)?,
                Task::Function(_, _, parts) => s::Type::Function(self.signature_roots(
                    facts,
                    parts,
                    context,
                    signatures::Roots::Completed,
                )?),
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
            } => {
                let child = self.type_key(facts, referent, context)?;
                let referent = self
                    .graph
                    .types
                    .get(&child)
                    .copied()
                    .ok_or("source referent was not completed before its parent")?;
                s::Type::Reference {
                    mutable,
                    region: self.region(region, context)?,
                    referent,
                }
            }
            Kind::FunctionItem {
                item, arguments, ..
            } => s::Type::FunctionItem(self.function_application(facts, item, arguments, context)?),
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
}
