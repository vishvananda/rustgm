//! Effective API reachability uses completed types, never paths or function bodies.
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, DefKind, ScopeId},
        package::Error,
        stability::{Inventory, PublicBudget, visit_public_bindings, visit_public_crate_roots},
        unit::NodeKey,
    },
    sema::model::Declarations,
    support::{
        diagnostics::WarningCode,
        hash::{HashMap, HashSet},
        retained,
    },
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
    inventory: Option<&'a Inventory>,
    pending: Vec<Task>,
    seen: HashSet<Task>,
    inherent: HashMap<DefId, Vec<DefId>>,
}
const STEPS: usize = 1_000_000;
const BYTES: usize = 64 * 1024 * 1024;

struct Reachability {
    budget: Budget,
    declarations: HashSet<Task>,
    exported_scopes: HashSet<ScopeId>,
}

pub(super) fn check_declarations(report: &Report, facts: &mut Declarations) -> Result<(), Error> {
    let lint = facts.safety_policies.checks_unreachable_pub();
    if let Some(mut reached) = collect(report, facts, lint, STEPS, BYTES)?
        && lint
    {
        reached.lint_unreachable_pub(report, facts)?;
    }
    Ok(())
}

#[cfg(test)]
fn check(report: &Report, facts: &Declarations) -> Result<Cost, Error> {
    check_with_limits(report, facts, STEPS, BYTES)
}
#[cfg(test)]
fn check_with_limits(
    report: &Report,
    facts: &Declarations,
    steps: usize,
    bytes: usize,
) -> Result<Cost, Error> {
    Ok(collect(report, facts, false, steps, bytes)?
        .map_or_else(Cost::default, |reached| reached.budget.cost))
}

fn collect(
    report: &Report,
    facts: &Declarations,
    lint: bool,
    steps: usize,
    bytes: usize,
) -> Result<Option<Reachability>, Error> {
    let library = report.package.library_policy();
    let inventory = library.and_then(crate::resolve::stability::Library::staged_inventory);
    if inventory.is_none() && !lint {
        return Ok(None);
    }
    let mut budget = Budget {
        cost: Cost::default(),
        steps,
        bytes,
    };
    budget.charge(1, size_of::<Interfaces<'_>>())?;
    if let Some(library) = library.filter(|_| inventory.is_some()) {
        library.charge_retained(&mut budget)?;
    }
    let mut interfaces = Interfaces {
        report,
        facts,
        inventory,
        pending: Vec::new(),
        seen: HashSet::default(),
        inherent: HashMap::default(),
    };
    interfaces.index_inherent(&mut budget)?;
    let mut visit =
        |definition, budget: &mut Budget| interfaces.enqueue(Task::Declaration(definition), budget);
    let exported_scopes = if lint {
        let roots = report
            .package
            .crate_roots()
            .map(|(module, _)| report.resolution.module_scopes[module.0 as usize]);
        visit_public_crate_roots(&report.resolution, &mut budget, roots, &mut visit)?
    } else {
        visit_public_bindings(&report.resolution, &mut budget, &mut visit)?
    };
    while let Some(task) = interfaces.pending.pop() {
        budget.charge(1, 0)?;
        match task {
            Task::Declaration(definition) => interfaces.declaration(definition, &mut budget)?,
            Task::Schema(definition) => interfaces.schema(definition, &mut budget)?,
            Task::Type(ty) => interfaces.ty(ty, &mut budget)?,
        }
    }
    Ok(Some(Reachability {
        budget,
        declarations: interfaces.seen,
        exported_scopes,
    }))
}

impl Reachability {
    fn lint_unreachable_pub(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), Error> {
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.budget.charge(1, 0)?;
            let id =
                DefId(u32::try_from(index).map_err(|_| {
                    Error::unlocated("public declaration identity exceeds 32 bits")
                })?);
            if definition.access != Access::Public
                || definition.kind.is_local()
                || matches!(definition.kind, DefKind::Constructor | DefKind::Variant)
                || facts
                    .associated
                    .trait_owner(id)
                    .is_some_and(|owner| owner != id)
                || self.declarations.contains(&Task::Declaration(id))
            {
                continue;
            }
            if let Some(key) = definition.node
                && matches!(
                    report
                        .package
                        .unit(key.file)
                        .configured
                        .ast
                        .node(key.node)
                        .kind,
                    crate::syntax::ast::Kind::Item(_)
                )
            {
                unreachable_pub(report, facts, key)?;
            }
        }
        for import in &report.resolution.imports {
            self.budget.charge(1, 0)?;
            if import.access == Access::Public && !self.exported_scopes.contains(&import.scope) {
                unreachable_pub(report, facts, import.node)?;
            }
        }
        Ok(())
    }
}

fn unreachable_pub(report: &Report, facts: &mut Declarations, key: NodeKey) -> Result<(), Error> {
    if facts.safety_policies.unreachable_pub(key) == crate::resolve::lint_policy::Level::Allow {
        return Ok(());
    }
    let unit = report.package.unit(key.file);
    facts.must_use_diagnostic(
        report,
        key,
        unit.configured.ast.node(key.node).span.start,
        0,
        WarningCode::UnreachablePub,
        "public item is unreachable from the crate root".into(),
    )
}

impl Interfaces<'_> {
    fn index_inherent(&mut self, budget: &mut Budget) -> Result<(), Error> {
        let facts = self.facts;
        for (index, owner) in facts.associated.owners().iter().enumerate() {
            budget.charge(1, 0)?;
            if owner.source.is_none()
                || !matches!(
                    owner.kind,
                    super::associated::OwnerKind::Impl {
                        trait_type: None,
                        ..
                    }
                )
            {
                continue;
            }
            let schema = facts
                .associated
                .schema(owner.self_definition)
                .ok_or_else(|| Error::unlocated("inherent interface lost its completed target"))?;
            let target = match facts.types.kind(schema.self_type) {
                Some(Kind::Adt(item, _)) => Some(DefId(item.0)),
                Some(_) => None,
                None => return Err(Error::unlocated("inherent interface target is missing")),
            };
            let owner = super::associated::OwnerId(
                u32::try_from(index)
                    .map_err(|_| Error::unlocated("inherent interface owner overflow"))?,
            );
            for &member in facts
                .associated
                .owner_members(owner)
                .ok_or_else(|| Error::unlocated("inherent interface members are missing"))?
            {
                budget.charge(1, 0)?;
                if self
                    .report
                    .resolution
                    .definitions
                    .get(member.0 as usize)
                    .is_none_or(|member| member.access != Access::Public)
                {
                    continue;
                }
                if let Some(target) = target {
                    self.retain_inherent(target, member, budget)?;
                } else {
                    self.enqueue(Task::Declaration(member), budget)?;
                }
            }
        }
        Ok(())
    }
    fn retain_inherent(
        &mut self,
        target: DefId,
        member: DefId,
        budget: &mut Budget,
    ) -> Result<(), Error> {
        let old = self.inherent.get(&target);
        let mut growth = retained::Growth::vector::<DefId>(
            old.map_or(0, Vec::len),
            old.map_or(0, Vec::capacity),
            1,
        );
        if old.is_none() {
            growth.include(retained::Growth::map::<DefId, Vec<DefId>>(
                self.inherent.len(),
                self.inherent.capacity(),
                1,
            ));
        }
        budget.charge(growth.work.saturating_add(1), growth.bytes)?;
        self.inherent
            .try_reserve(usize::from(old.is_none()))
            .map_err(|_| Error::unlocated("inherent interface index allocation failed"))?;
        let rows = self.inherent.entry(target).or_default();
        rows.try_reserve(1)
            .map_err(|_| Error::unlocated("inherent interface row allocation failed"))?;
        rows.push(member);
        Ok(())
    }
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
        if let Some(members) = self.inherent.remove(&id) {
            budget.charge(members.len().saturating_add(1), 0)?;
            for member in members {
                self.enqueue(Task::Declaration(member), budget)?;
            }
        }
        if actual.access == Access::Public {
            self.check_stability(id, node)?;
            self.enqueue(Task::Schema(id), budget)?;
        } else if actual.kind == DefKind::TypeAlias {
            // Private alias spelling does not hide its normalized public target.
            self.enqueue(Task::Schema(id), budget)?;
        }
        Ok(())
    }

    fn check_stability(&self, id: DefId, node: NodeKey) -> Result<(), Error> {
        let Some(inventory) = self.inventory else {
            return Ok(());
        };
        if self.report.package.has_guests() && !self.report.package.staged_crate(node.file) {
            return Ok(());
        }
        if inventory.declaration(id).is_none() {
            return Err(Declarations::error(
                self.report,
                node,
                "public declaration exposed by an API has missing library stability attribute",
            ));
        }
        if self.report.resolution.definitions[id.0 as usize].kind == DefKind::Function
            && self.facts.is_const_function(id)
            && inventory.const_declaration(id).is_none()
        {
            return Err(Declarations::error(
                self.report,
                node,
                "public const member exposed by an API has missing const library stability attribute",
            ));
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
            Clause::UnstableFeature(_) => return Ok(()),
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
            Kind::Opaque(_, arguments) | Kind::Future { arguments, .. } => {
                self.arguments(arguments, budget)?;
            }
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
