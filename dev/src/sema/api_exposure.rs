//! Effective API reachability uses completed types, never paths or function bodies.
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, DefKind},
        package::Error,
        stability::{Inventory, PublicBudget, visit_public_bindings},
    },
    sema::model::Declarations,
    support::{hash::HashSet, retained},
    types::{
        GenericArgs, ItemId, Kind, TypeId,
        arguments::Argument,
        parameters::{clauses::Clause, predicates::Bound},
    },
};
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Task {
    Declaration(DefId),
    Schema(DefId),
    Type(TypeId),
}
#[derive(Clone, Copy, Default, Debug)]
pub(super) struct Cost {
    pub steps: usize,
    pub bytes: usize,
    pub types: usize,
    pub declarations: usize,
}
struct Budget {
    cost: Cost,
    steps: usize,
    bytes: usize,
}
impl PublicBudget for Budget {
    fn charge(&mut self, steps: usize, bytes: usize) -> Result<(), Error> {
        self.cost.steps = self.cost.steps.saturating_add(steps);
        self.cost.bytes = self.cost.bytes.saturating_add(bytes);
        if self.cost.steps > self.steps || self.cost.bytes > self.bytes {
            return Err(Error::unlocated(
                "library interface reachability exceeds work or storage budget",
            ));
        }
        Ok(())
    }
}
struct Interfaces<'a> {
    report: &'a Report,
    facts: &'a Declarations,
    inventory: &'a Inventory,
    pending: Vec<Task>,
    seen: HashSet<Task>,
}
pub(super) fn check(report: &Report, facts: &Declarations) -> Result<Cost, Error> {
    check_with_limits(report, facts, 1_000_000, 64 * 1024 * 1024)
}
fn check_with_limits(
    report: &Report,
    facts: &Declarations,
    steps: usize,
    bytes: usize,
) -> Result<Cost, Error> {
    let Some(inventory) = report
        .package
        .library_policy()
        .and_then(crate::resolve::stability::Library::inventory)
    else {
        return Ok(Cost::default());
    };
    let mut budget = Budget {
        cost: Cost::default(),
        steps,
        bytes,
    };
    budget.charge(1, size_of::<Interfaces<'_>>())?;
    report
        .package
        .library_policy()
        .ok_or_else(|| Error::unlocated("library policy disappeared"))?
        .charge_retained(&mut budget)?;
    let mut interfaces = Interfaces {
        report,
        facts,
        inventory,
        pending: Vec::new(),
        seen: HashSet::default(),
    };
    visit_public_bindings(&report.resolution, &mut budget, |definition, budget| {
        interfaces.enqueue(Task::Declaration(definition), budget)
    })?;
    while let Some(task) = interfaces.pending.pop() {
        budget.charge(1, 0)?;
        match task {
            Task::Declaration(definition) => interfaces.declaration(definition, &mut budget)?,
            Task::Schema(definition) => interfaces.schema(definition, &mut budget)?,
            Task::Type(ty) => interfaces.ty(ty, &mut budget)?,
        }
    }
    Ok(budget.cost)
}
impl Interfaces<'_> {
    fn enqueue(&mut self, task: Task, budget: &mut Budget) -> Result<(), Error> {
        budget.charge(1, 0)?;
        if self.seen.contains(&task) {
            return Ok(());
        }
        let table = retained::Growth::map::<Task, ()>(self.seen.len(), self.seen.capacity(), 1);
        let queue =
            retained::Growth::vector::<Task>(self.pending.len(), self.pending.capacity(), 1);
        budget.charge(
            table.work.saturating_add(queue.work),
            table.bytes.saturating_add(queue.bytes),
        )?;
        self.seen
            .try_reserve(1)
            .map_err(|_| Error::unlocated("library interface index allocation failed"))?;
        self.pending
            .try_reserve(1)
            .map_err(|_| Error::unlocated("library interface work allocation failed"))?;
        self.seen.insert(task);
        self.pending.push(task);
        Ok(())
    }
    fn declaration(&mut self, id: DefId, budget: &mut Budget) -> Result<(), Error> {
        budget.cost.declarations += 1;
        let actual = self
            .report
            .resolution
            .definitions
            .get(id.0 as usize)
            .ok_or_else(|| Error::unlocated("library interface has an invalid declaration"))?;
        let Some(node) = actual.node else {
            return Ok(());
        };
        if actual.access == Access::Public {
            if self.inventory.declaration(id).is_none() {
                return Err(Declarations::error(
                    self.report,
                    node,
                    "public declaration exposed by an API has missing library stability attribute",
                ));
            }
            self.enqueue(Task::Schema(id), budget)?;
        } else if actual.kind == DefKind::TypeAlias {
            // Private alias spelling does not hide its normalized public target.
            self.enqueue(Task::Schema(id), budget)?;
        }
        Ok(())
    }
    fn schema(&mut self, id: DefId, budget: &mut Budget) -> Result<(), Error> {
        if let Some(ty) = self.facts.definitions.get(id.0 as usize).copied().flatten() {
            self.enqueue(Task::Type(ty), budget)?;
        }
        for predicate in self.facts.generics.declared_predicates(id) {
            budget.charge(1, 0)?;
            self.clause(predicate.value, budget)?;
        }
        if let Some(&parent) = self.facts.generics.parents.get(&id) {
            self.enqueue(Task::Schema(parent), budget)?;
        }
        for &variant in self.facts.members.variants(ItemId(id.0)) {
            budget.charge(1, 0)?;
            for field in self
                .facts
                .members
                .fields(variant)
                .ok_or_else(|| Error::unlocated("library interface fields disappeared"))?
            {
                budget.charge(1, 0)?;
                if field.access == Access::Public {
                    self.enqueue(Task::Type(field.ty), budget)?;
                }
            }
        }
        Ok(())
    }
    fn arguments(&mut self, arguments: GenericArgs, budget: &mut Budget) -> Result<(), Error> {
        for argument in self
            .facts
            .types
            .arguments(arguments)
            .ok_or_else(|| Error::unlocated("library interface has invalid type arguments"))?
        {
            budget.charge(1, 0)?;
            match argument {
                Argument::Type(ty) => self.enqueue(Task::Type(ty), budget)?,
                Argument::Const(value) => {
                    let ty = self.facts.types.const_type(value).ok_or_else(|| {
                        Error::unlocated("library interface const argument has no type")
                    })?;
                    self.enqueue(Task::Type(ty), budget)?;
                }
                Argument::Lifetime(_) => {}
            }
        }
        Ok(())
    }
    fn clause(&mut self, clause: Clause, budget: &mut Budget) -> Result<(), Error> {
        let predicate = match clause {
            Clause::Predicate(predicate) => predicate,
            // This visits declaration identities, not quantified evidence or uses.
            Clause::ForAll(id) => {
                self.facts
                    .types
                    .quantified(id)
                    .ok_or_else(|| Error::unlocated("library interface binder is invalid"))?
                    .predicate
            }
        };
        self.enqueue(Task::Type(predicate.ty), budget)?;
        if let Some((id, _)) = predicate.bound.trait_parts() {
            self.trait_application(id, budget)?;
        } else if let Bound::ProjectionEqual(ty) = predicate.bound {
            self.enqueue(Task::Type(ty), budget)?;
        }
        Ok(())
    }
    fn trait_application(
        &mut self,
        id: crate::types::traits::ApplicationId,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let application =
            self.facts.types.trait_application(id).ok_or_else(|| {
                Error::unlocated("library interface trait application is invalid")
            })?;
        self.enqueue(Task::Declaration(DefId(application.item.0)), budget)?;
        self.arguments(application.arguments, budget)
    }
    fn ty(&mut self, ty: TypeId, budget: &mut Budget) -> Result<(), Error> {
        budget.cost.types += 1;
        match self
            .facts
            .types
            .kind(ty)
            .ok_or_else(|| Error::unlocated("library interface type is invalid"))?
        {
            Kind::Adt(item, arguments) => {
                self.enqueue(Task::Declaration(DefId(item.0)), budget)?;
                self.arguments(arguments, budget)?;
            }
            Kind::FunctionItem {
                item,
                arguments,
                signature,
            } => {
                self.enqueue(Task::Declaration(DefId(item.0)), budget)?;
                self.enqueue(Task::Type(signature), budget)?;
                self.arguments(arguments, budget)?;
            }
            Kind::Foreign(item) => self.enqueue(Task::Declaration(DefId(item.0)), budget)?,
            Kind::Reference { referent: ty, .. }
            | Kind::Pointer { pointee: ty, .. }
            | Kind::Slice(ty)
            | Kind::Array { element: ty, .. }
            | Kind::ArrayTerm { element: ty, .. }
            | Kind::Pattern { base: ty, .. } => self.enqueue(Task::Type(ty), budget)?,
            Kind::Tuple(list) => self.list(list, budget)?,
            Kind::Function {
                parameters, result, ..
            } => {
                self.enqueue(Task::Type(result), budget)?;
                self.list(parameters, budget)?;
            }
            Kind::Projection(application) => {
                self.enqueue(Task::Declaration(DefId(application.item.0)), budget)?;
                self.enqueue(Task::Type(application.subject), budget)?;
                self.trait_application(application.trait_application, budget)?;
                self.arguments(application.arguments, budget)?;
            }
            Kind::Dynamic {
                shape, arguments, ..
            } => {
                let shape =
                    self.facts.types.dynamic_shape(shape).ok_or_else(|| {
                        Error::unlocated("library interface object shape is invalid")
                    })?;
                if let Some(item) = shape.principal {
                    self.enqueue(Task::Declaration(DefId(item.0)), budget)?;
                }
                self.arguments(arguments, budget)?;
            }
            Kind::Opaque(_, arguments) => self.arguments(arguments, budget)?,
            Kind::Closure {
                signature,
                arguments,
                ..
            } => {
                self.enqueue(Task::Type(signature), budget)?;
                self.arguments(arguments, budget)?;
            }
            Kind::Never | Kind::Primitive(_) | Kind::Parameter(_) => {}
        }
        Ok(())
    }
    fn list(&mut self, list: crate::types::ListId, budget: &mut Budget) -> Result<(), Error> {
        for &ty in self
            .facts
            .types
            .list(list)
            .ok_or_else(|| Error::unlocated("library interface type list is invalid"))?
        {
            self.enqueue(Task::Type(ty), budget)?;
        }
        Ok(())
    }
}
#[cfg(test)]
#[path = "../../tests/invariants/visible_nominals.rs"]
mod tests;
