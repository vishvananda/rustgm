//! Actual initialized cleanup requests acquire ordinary scoped const trait proof.
use super::{Arc, Body, Contract, Declarations, Owner, Prepared, Purpose, State, Work, claims};
use crate::{
    body::regions::{BuiltinRequirements, Proof},
    types::{
        GenericArgs, ItemId,
        parameters::predicates::{Bound, Predicate},
        traits::Application,
    },
};
pub(crate) fn select(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    subjects: impl Iterator<Item = Result<super::Subject, &'static str>>,
    precise: bool,
    physical_cleanup: bool,
) -> Result<(), String> {
    let mut work = Work::new(1_000_000);
    let mut prepared = prepare(facts, body, contract, &mut work)?;
    let role = prepared.role;
    let environment = Arc::clone(&prepared.environment);
    let effect = prepared.effect();
    let mut selected = Requests::default();
    for subject in subjects {
        work.charge(1)?;
        let subject = subject?;
        let ty = subject.ty;
        prepared
            .claims
            .retain_declared(ty, subject.purpose, &mut work)?;
        if prepared
            .requirement(&facts.types, ty, subject.purpose, &mut work)
            .is_ok()
        {
            continue;
        }
        if precise && !subject.access {
            if body.source.source().is_none() {
                return Err(
                    "imported const precise live-drop definition policy is not implemented".into(),
                );
            }
            continue;
        }
        selected.push((ty, subject.purpose), &mut work)?;
    }
    if selected.values.is_empty() {
        return if physical_cleanup {
            publish(facts, contract, prepared, work)
        } else {
            Ok(())
        };
    }
    let first = crate::body::dispatch::copy_frontier(facts, contract)?;
    let goals = goals(facts, &selected.values, role, effect, &mut work)?;
    let proof = crate::sema::compiler_items::prove_cleanup_requirements(
        facts,
        Some(environment),
        contract.universals,
        first,
        &goals,
    )?;
    for &(ty, purpose) in &selected.values {
        prepared
            .claims
            .select(&mut facts.types, ty, purpose, effect, &mut work)?;
    }
    retain_proof(body, contract, &prepared, proof, &mut work)?;
    publish(facts, contract, prepared, work)
}
pub(super) fn retain_proof(
    body: &Body,
    contract: &mut Contract,
    prepared: &Prepared,
    proof: crate::types::lifetime_proof::Obligations,
    work: &mut Work,
) -> Result<(), String> {
    let growth = crate::support::retained::Growth::vector::<Proof>(
        contract.proofs.len(),
        contract.proofs.capacity(),
        1,
    );
    work.charge(growth.work)?;
    if contract
        .retained_bytes()
        .saturating_add(growth.bytes)
        .saturating_add(proof.allocation_bytes())
        .saturating_add(prepared.claims.retained_bytes())
        .saturating_add(size_of::<State>())
        > 64 * 1024 * 1024
    {
        return Err("selected const cleanup proof exceeds storage budget".into());
    }
    contract
        .proofs
        .try_reserve(1)
        .map_err(|_| "selected const cleanup proof allocation failed")?;
    contract.proofs.push(Proof::new(body.source, proof));
    Ok(())
}
/// Retention follows actual owner authentication above, even for an empty goal set.
/// Physical activity only decides whether that already checked owner must survive.
pub(super) fn publish(
    facts: &mut Declarations,
    contract: &mut Contract,
    prepared: Prepared,
    mut work: Work,
) -> Result<(), String> {
    work.charge(1)?;
    if contract
        .retained_bytes()
        .saturating_add(prepared.claims.retained_bytes())
        .saturating_add(super::required_allocation())
        > 64 * 1024 * 1024
    {
        return Err("defining cleanup owner exceeds storage budget".into());
    }
    let builtins = Arc::make_mut(
        contract
            .builtins
            .get_or_insert_with(|| Arc::new(BuiltinRequirements::empty())),
    );
    builtins.set_defining(Some(Arc::new(State::Prepared(prepared))));
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.steps);
    Ok(())
}

#[derive(Default)]
struct Requests {
    values: Vec<(crate::types::TypeId, Purpose)>,
    seen: crate::support::hash::HashSet<(crate::types::TypeId, Purpose)>,
    bytes: usize,
}
impl Requests {
    fn push(
        &mut self,
        request: (crate::types::TypeId, Purpose),
        work: &mut Work,
    ) -> Result<(), &'static str> {
        work.charge(1)?;
        if self.seen.contains(&request) {
            return Ok(());
        }
        if self.values.len() >= crate::support::crates::MAX_COMPONENTS {
            return Err("active const cleanup request budget exceeded");
        }
        let vector = crate::support::retained::Growth::vector::<(crate::types::TypeId, Purpose)>(
            self.values.len(),
            self.values.capacity(),
            1,
        );
        let index = crate::support::retained::Growth::map::<(crate::types::TypeId, Purpose), ()>(
            self.seen.len(),
            self.seen.capacity(),
            1,
        );
        work.charge(vector.work.saturating_add(index.work))?;
        self.bytes = self
            .bytes
            .checked_add(vector.bytes)
            .and_then(|b| b.checked_add(index.bytes))
            .filter(|&b| b <= 64 * 1024 * 1024)
            .ok_or("selected cleanup request storage budget exceeded")?;
        self.values
            .try_reserve(1)
            .map_err(|_| "selected cleanup request allocation failed")?;
        self.seen
            .try_reserve(1)
            .map_err(|_| "selected cleanup index allocation failed")?;
        self.seen.insert(request);
        self.values.push(request);
        Ok(())
    }
}

pub(super) fn goals(
    facts: &mut Declarations,
    selected: &[(crate::types::TypeId, Purpose)],
    destruct: claims::Role,
    effect: crate::support::constness::Constness,
    work: &mut Work,
) -> Result<Vec<(Purpose, Predicate)>, String> {
    work.charge(selected.len())?;
    if selected
        .len()
        .saturating_mul(size_of::<(Purpose, Predicate)>())
        > 64 * 1024 * 1024
    {
        return Err("selected cleanup goal storage budget exceeded".into());
    }
    let mut goals = Vec::new();
    goals
        .try_reserve_exact(selected.len())
        .map_err(|_| "cleanup goal allocation failed")?;
    for &(ty, purpose) in selected {
        let role = if purpose == Purpose::OwnDrop {
            facts
                .source_bindings
                .as_deref()
                .ok_or("own Drop lost its source catalog")?
                .validated_language(crate::support::bindings::LangItem::Drop)?
                .ok_or("own Drop requires its actual language declaration")?
                .definition
        } else {
            destruct.require()?
        };
        let application = facts.types.intern_trait_application(Application {
            item: ItemId(role.0),
            arguments: GenericArgs::EMPTY,
        })?;
        goals.push((
            purpose,
            Predicate {
                ty,
                bound: Bound::trait_application(application, effect),
            },
        ));
    }
    Ok(goals)
}

pub(super) fn prepare(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    work: &mut Work,
) -> Result<Prepared, String> {
    let Owner::Function {
        definition,
        signature,
        execution: None,
    } = body.owner
    else {
        return Err("const cleanup selection requires its actual source function".into());
    };
    if body.instance.is_some() {
        return Err("const cleanup specialization requires its original defining proof".into());
    }
    let default = facts
        .associated
        .trait_owner(definition)
        .is_some_and(|owner| facts.associated.constant_trait(owner) == Some(true));
    let actual = super::active_definition(facts, definition, body.source, &mut 0, default)?
        .ok_or("active const cleanup lost its actual source definition")?;
    super::validate_contract(actual, contract)?;
    if signature != actual.signature || contract.signature != Some(signature) {
        return Err("active const cleanup changed its actual signature".into());
    }
    let builtins = contract
        .builtins
        .as_deref()
        .ok_or("active const cleanup has no actual source policy")?;
    let environment = Arc::clone(
        builtins
            .environment
            .as_ref()
            .ok_or("active const cleanup lost its environment")?,
    );
    if let Some(expected) = facts.generics.body_environment(definition) {
        if !Arc::ptr_eq(expected, &environment) {
            return Err("active const cleanup changed its defining environment".into());
        }
    } else if facts.generics.caller_scope_pending(definition)
        || !contract.parameters.is_empty()
        || !contract.type_assumptions.is_empty()
    {
        return Err("active const cleanup requires its completed defining environment".into());
    }
    let role = claims::Role::current(facts, crate::support::bindings::LangItem::Destruct, work)?;
    let copy_role = claims::Role::current(facts, crate::support::bindings::LangItem::Copy, work)?;
    let mut prepared = Prepared {
        definition: actual,
        owned_signature: signature,
        const_context: body.const_context,
        environment: Arc::clone(&environment),
        role,
        copy_role,
        claims: claims::Claims::new(facts, &environment, role, copy_role, work)?,
    };
    if let Some(state @ State::Prepared(_)) = builtins.defining() {
        let prior = state.checked(body, builtins.environment.as_ref(), work)?;
        let effect = prepared.effect();
        prepared
            .claims
            .retain_selected(&mut facts.types, &prior.claims, effect, work)?;
    }
    Ok(prepared)
}
