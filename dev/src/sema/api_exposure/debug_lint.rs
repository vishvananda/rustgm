//! Reachable nominal types consume the checked Debug trait and impl identities.
use super::{Budget, PublicBudget, Reachability, Task};
use crate::{
    resolve::{
        expansion::Report,
        model::{DefId, DefKind},
        package::Error,
    },
    sema::model::Declarations,
    support::{
        diagnostics::WarningCode,
        hash::{HashMap, HashSet},
        retained::Growth,
    },
    types::{Kind, TypeId},
};

pub(super) struct Output {
    pub(super) other: Option<DefId>,
    pub(super) ty: TypeId,
}

#[derive(Default)]
pub(super) struct AssociatedTypes(HashMap<DefId, Vec<Output>>);

impl AssociatedTypes {
    pub(super) fn collect(facts: &Declarations, budget: &mut Budget) -> Result<Self, Error> {
        let mut outputs = Self::default();
        let Some(registry) = facts.generics.implementations.as_deref() else {
            return Ok(outputs);
        };
        budget.charge(1, size_of::<Self>())?;
        for implementation in registry.validated() {
            budget.charge(1, 0)?;
            let Some((application, _)) = implementation.predicate.bound.trait_parts() else {
                continue;
            };
            let Some(application) = facts.types.trait_application(application) else {
                return Err(Error::unlocated(
                    "effective visibility lost its implementation trait",
                ));
            };
            let trait_ = DefId(application.item.0);
            let subject = nominal(facts, implementation.predicate.ty);
            for &(_, definition) in &implementation.types {
                budget.charge(1, 0)?;
                let Some(ty) = facts
                    .definitions
                    .get(definition.0 as usize)
                    .copied()
                    .flatten()
                else {
                    continue;
                };
                outputs.push(trait_, Output { other: subject, ty }, budget)?;
                if let Some(subject) = subject.filter(|&subject| subject != trait_) {
                    outputs.push(
                        subject,
                        Output {
                            other: Some(trait_),
                            ty,
                        },
                        budget,
                    )?;
                }
            }
        }
        Ok(outputs)
    }

    pub(super) fn take(&mut self, owner: DefId) -> Option<Vec<Output>> {
        self.0.remove(&owner)
    }

    fn push(&mut self, owner: DefId, output: Output, budget: &mut Budget) -> Result<(), Error> {
        let existing = self.0.get(&owner);
        let mut growth = Growth::vector::<Output>(
            existing.map_or(0, Vec::len),
            existing.map_or(0, Vec::capacity),
            1,
        );
        if existing.is_none() {
            growth.include(Growth::map::<DefId, Vec<Output>>(
                self.0.len(),
                self.0.capacity(),
                1,
            ));
        }
        budget.charge(1 + growth.work, growth.bytes)?;
        self.0
            .try_reserve(usize::from(existing.is_none()))
            .map_err(|_| {
                Error::unlocated("effective visibility associated output index allocation failed")
            })?;
        let values = self.0.entry(owner).or_default();
        values.try_reserve(1).map_err(|_| {
            Error::unlocated("effective visibility associated output allocation failed")
        })?;
        values.push(output);
        Ok(())
    }
}

fn nominal(facts: &Declarations, ty: TypeId) -> Option<DefId> {
    match facts.types.kind(ty)? {
        Kind::Adt(item, _) | Kind::Foreign(item) => Some(DefId(item.0)),
        _ => None,
    }
}

impl Reachability {
    pub(super) fn lint_missing_debug(
        &mut self,
        report: &Report,
        facts: &mut Declarations,
    ) -> Result<(), Error> {
        let Some(debug) = report
            .resolution
            .source_bindings
            .as_deref()
            .and_then(crate::resolve::bindings::Reservations::debug_declaration)
        else {
            return Ok(());
        };
        let covered = debug_implementations(facts, debug.definition, &mut self.budget)?;
        for (index, definition) in report.resolution.definitions.iter().enumerate() {
            self.budget.charge(1, 0)?;
            if !matches!(definition.kind, DefKind::Struct | DefKind::Enum) {
                continue;
            }
            let id = DefId(
                u32::try_from(index)
                    .map_err(|_| Error::unlocated("Debug lint identity exceeds 32 bits"))?,
            );
            let Some(key) = definition.node else {
                continue;
            };
            if !self.declarations.contains(&Task::Declaration(id))
                || covered.contains(&id)
                || facts.safety_policies.missing_debug_implementations(key)
                    == crate::resolve::lint_policy::Level::Allow
            {
                continue;
            }
            let unit = report.package.unit(key.file);
            facts.must_use_diagnostic(
                report,
                key,
                unit.configured.ast.node(key.node).span.start,
                0,
                WarningCode::MissingDebugImplementations,
                "reachable type does not implement Debug".into(),
            )?;
        }
        Ok(())
    }
}

fn debug_implementations(
    facts: &Declarations,
    debug: DefId,
    budget: &mut Budget,
) -> Result<HashSet<DefId>, Error> {
    let mut covered = HashSet::default();
    let Some(registry) = facts.generics.implementations.as_deref() else {
        return Ok(covered);
    };
    for implementation in registry.validated() {
        budget.charge(1, 0)?;
        let Some((application, _)) = implementation.predicate.bound.trait_parts() else {
            continue;
        };
        let Some(application) = facts.types.trait_application(application) else {
            return Err(Error::unlocated(
                "Debug lint lost its checked implementation trait",
            ));
        };
        if application.item.0 != debug.0
            || !registry
                .polarity(&facts.types, &facts.associated, implementation.owner)
                .map_err(Error::unlocated)?
                .is_positive()
        {
            continue;
        }
        if let Some(id) = nominal(facts, implementation.predicate.ty)
            && !covered.contains(&id)
        {
            let growth = Growth::map::<DefId, ()>(covered.len(), covered.capacity(), 1);
            budget.charge(1 + growth.work, growth.bytes)?;
            covered
                .try_reserve(1)
                .map_err(|_| Error::unlocated("Debug lint implementation set allocation failed"))?;
            covered.insert(id);
        }
    }
    Ok(covered)
}
