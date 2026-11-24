//! Resolve version-fourteen constant operands in their numbered defining body.
use super::{Cursor, Declarations, Demand, Error, Recorded, ReservationWork, error, g, s};
use crate::{
    resolve::model::DefId,
    sema::{
        associated::OwnerKind,
        crates::typed_headers::{
            predicate_remap::translation::feature_names::paid,
            recipe_cursor::{Port, const_argument},
        },
        declaration_inputs::{self, Inputs},
        values::Instance,
    },
    support::{
        crates::owner_catalog::{BodyRef, DeclarationRef},
        hash::HashMap,
        retained,
    },
    types::{
        GenericArgs, Store, TypeId,
        arguments::Argument,
        parameters::{clauses::Clause, predicates::Predicate},
        substitute::Bound,
    },
};

#[derive(Default)]
pub(super) struct Operands {
    arguments: HashMap<(BodyRef, u32, u32), GenericArgs>,
    goals: HashMap<BodyRef, Vec<Predicate>>,
}

impl Operands {
    pub(super) fn prepare(
        &mut self,
        body: BodyRef,
        (declaration, arguments, ty): (DeclarationRef, g::Range, g::TypeRef),
        port: &mut Recorded<'_, '_, '_>,
        cursor: &mut Cursor,
        store: &mut Store,
        work: &mut ReservationWork,
    ) -> Result<Use, Error> {
        let key = (body, arguments.start, arguments.end);
        work.charge(1)?;
        let mapped = if let Some(&mapped) = self.arguments.get(&key) {
            mapped
        } else {
            let opened = open(port, cursor, store, arguments, work)?;
            let before = retained::map(&self.arguments);
            let growth = retained::Growth::map::<(BodyRef, u32, u32), GenericArgs>(
                self.arguments.len(),
                self.arguments.capacity(),
                1,
            );
            work.charge(growth.work)?;
            work.reserve(growth.bytes)?;
            self.arguments.insert(key, opened);
            work.retained = work
                .retained
                .saturating_sub(growth.bytes)
                .saturating_add(retained::map(&self.arguments).saturating_sub(before));
            opened
        };
        Ok(Use {
            ty,
            instance: Instance {
                definition: port.definition(declaration, work)?,
                arguments: mapped,
            },
        })
    }

    pub(super) fn arguments(&self, body: BodyRef, span: g::Range) -> Result<GenericArgs, Error> {
        self.arguments
            .get(&(body, span.start, span.end))
            .copied()
            .ok_or_else(|| error("source associated constant lost its actual body argument list"))
    }

    pub(super) fn finish(
        &mut self,
        body: BodyRef,
        uses: Uses,
        cursor: &Cursor,
        facts: &mut Declarations,
        work: &mut ReservationWork,
    ) -> Result<(), Error> {
        if uses.0.is_empty() {
            return Ok(());
        }
        work.charge(1)?;
        if self.goals.contains_key(&body) {
            work.retained = work.retained.saturating_sub(retained::vector(&uses.0));
            return Ok(());
        }
        let growth = retained::Growth::map::<BodyRef, Vec<Predicate>>(
            self.goals.len(),
            self.goals.capacity(),
            1,
        );
        work.charge(growth.work)?;
        work.reserve(growth.bytes)?;
        let goals = uses.applied_goals(cursor, facts, work)?;
        let before = retained::map(&self.goals);
        self.goals.insert(body, goals);
        work.retained = work
            .retained
            .saturating_sub(growth.bytes)
            .saturating_add(retained::map(&self.goals).saturating_sub(before));
        Ok(())
    }

    pub(super) fn goals(&self, body: BodyRef) -> &[Predicate] {
        self.goals.get(&body).map_or(&[], Vec::as_slice)
    }
}

pub(super) struct Use {
    ty: g::TypeRef,
    instance: Instance,
}

#[derive(Default)]
pub(super) struct Uses(Vec<Use>);
impl Uses {
    pub(super) fn push(&mut self, operand: Use, work: &mut ReservationWork) -> Result<(), Error> {
        let before = retained::vector(&self.0);
        let growth = retained::Growth::vector::<Use>(self.0.len(), self.0.capacity(), 1);
        work.charge(growth.work)?;
        work.reserve(growth.bytes)?;
        self.0.push(operand);
        work.retained = work
            .retained
            .saturating_sub(growth.bytes)
            .saturating_add(retained::vector(&self.0).saturating_sub(before));
        Ok(())
    }

    fn applied_goals(
        self,
        cursor: &Cursor,
        facts: &mut Declarations,
        work: &mut ReservationWork,
    ) -> Result<Vec<Predicate>, Error> {
        let bytes = retained::vector(&self.0);
        work.reserve(self.0.len().saturating_mul(size_of::<Predicate>()))?;
        let mut goals = Vec::with_capacity(self.0.len());
        for operand in self.0 {
            let result = applied_contract(facts, operand.instance, &mut goals, work)?;
            if result != cursor.child(operand.ty)? {
                return Err(error(
                    "source associated constant read changed its canonical applied result",
                ));
            }
        }
        work.retained = work.retained.saturating_sub(bytes);
        Ok(goals)
    }
}

fn open(
    port: &mut Recorded<'_, '_, '_>,
    cursor: &mut Cursor,
    store: &mut Store,
    arguments: g::Range,
    work: &mut ReservationWork,
) -> Result<GenericArgs, Error> {
    let rows = port.rows();
    let inputs = arguments.slice(&rows.arguments).map_err(error)?;
    work.charge(inputs.len().saturating_add(1))?;
    let bytes = inputs.len().saturating_mul(size_of::<Argument>());
    work.reserve(bytes)?;
    let mut mapped = Vec::with_capacity(inputs.len());
    for &argument in inputs {
        let actual = match argument {
            s::Argument::Type(root) => match cursor.demand(port, store, root, work)? {
                Demand::Ready(root) => Argument::Type(root),
                Demand::Blocked(_) => {
                    return Err(error(
                        "source associated constant requires its actual argument recipe",
                    ));
                }
            },
            s::Argument::Region(reference) => match port.region(reference, work)? {
                Demand::Ready(region) => Argument::Lifetime(region),
                Demand::Blocked(_) => {
                    return Err(error(
                        "source associated constant requires its actual region domain",
                    ));
                }
            },
            s::Argument::Const(term) => const_argument(port, store, term, work)?,
        };
        mapped.push(actual);
    }
    let result = paid(work, |pay| {
        store.intern_arguments_observed::<true>(&mapped, &pay)
    });
    work.retained = work.retained.saturating_sub(bytes);
    result
}

fn applied_contract(
    facts: &mut Declarations,
    instance: Instance,
    goals: &mut Vec<Predicate>,
    work: &mut ReservationWork,
) -> Result<TypeId, Error> {
    let (trait_goal, inherent_owner) = constant_owner(facts, instance)?;
    let premises = match inherent_owner {
        Some(owner) => {
            declaration_inputs::own_view(&facts.generics, facts.crates.as_deref(), owner)
                .map_err(error)?
        }
        None => Inputs::new(&[], &[]),
    };
    let declared = facts
        .definitions
        .get(instance.definition.0 as usize)
        .copied()
        .flatten()
        .ok_or_else(|| error("source associated constant lost its declared result type"))?;
    let count = facts
        .types
        .arguments(instance.arguments)
        .ok_or_else(|| error("source constant operand lost its canonical argument list"))?
        .len();
    work.charge(count.saturating_mul(4).saturating_add(1))?;
    let bytes = count
        .saturating_add(1)
        .saturating_mul(4 * size_of::<Argument>());
    let before = retained::vector(goals);
    let growth = retained::Growth::vector::<Predicate>(
        goals.len(),
        goals.capacity(),
        premises
            .len()
            .saturating_add(usize::from(trait_goal.is_some())),
    );
    work.charge(growth.work)?;
    work.reserve(bytes.saturating_add(growth.bytes))?;
    let actual = instance.selected_arguments(facts).map_err(error)?;
    let result = paid(work, |pay| {
        let mut bound = Bound::parameters_captured_charged(
            &facts.types,
            &actual.bindings,
            actual.regions,
            &pay,
        )?;
        let result = bound.apply_charged(&mut facts.types, declared, &pay)?;
        if let Some(goal) = trait_goal {
            goals.push(bound.predicate_charged(&mut facts.types, goal, &pay)?);
        }
        for premise in premises.iter() {
            let Clause::Predicate(goal) = premise.value else {
                return Err("source inherited constant needs its wider premise profile");
            };
            goals.push(bound.predicate_charged(&mut facts.types, goal, &pay)?);
        }
        Ok(result)
    });
    work.retained = work
        .retained
        .saturating_sub(bytes.saturating_add(growth.bytes))
        .saturating_add(retained::vector(goals).saturating_sub(before));
    result
}

fn constant_owner(
    facts: &Declarations,
    instance: Instance,
) -> Result<(Option<Predicate>, Option<DefId>), Error> {
    if let Some(owner) = facts.associated.trait_owner(instance.definition) {
        let goal = *facts
            .generics
            .trait_self_bounds
            .get(&owner)
            .ok_or_else(|| error("source constant operand lost its trait's canonical self goal"))?;
        return Ok((Some(goal), None));
    }
    let owner = facts
        .associated
        .owner(instance.definition)
        .and_then(|id| facts.associated.get(id))
        .copied()
        .filter(|owner| matches!(owner.kind, OwnerKind::ImportedInherent(_)))
        .ok_or_else(|| error("source constant operand lost its actual inherent declaration"))?;
    facts.associated.require_imported(owner).map_err(error)?;
    Ok((None, Some(owner.self_definition)))
}
