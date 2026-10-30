//! Per-body source decisions retain exact identity and all ordinary selection debts.
use super::{
    ir::{Body, BodyKey, Rvalue},
    regions::{BuiltinRequirements, Contract, Proof},
};
use crate::{
    sema::model::{Declarations, SourceCopyRequirements},
    support::hash::{HashMap, HashSet},
    types::{
        GenericArgs, ItemId, Kind, TypeId,
        instance::Instance,
        parameters::{
            environment::Environment,
            predicates::{Bound, Predicate},
        },
        traits::Application,
    },
};
use std::sync::Arc;
mod carrier;
mod debt_transfer;
mod ephemeral_imports;
pub(crate) use ephemeral_imports::ImportedInputs;
pub(in crate::body) mod materialized;
mod reproof;
#[cfg(test)]
#[path = "../../tests/invariants/copy_carriers.rs"]
mod tests;

#[derive(Clone)]
pub(crate) struct Prepared {
    key: BodyKey,
    source: super::ir::BodyOrigin,
    instance: Option<Instance>,
    signatures: Option<(TypeId, TypeId)>,
    requirements: Arc<SourceCopyRequirements>,
    environment: Option<Arc<Environment>>,
    decisions: HashMap<TypeId, bool>,
}
impl Prepared {
    pub(crate) fn retained_bytes(&self) -> usize {
        let mut total = crate::support::retained::Allocation::default();
        total.map(&self.decisions);
        size_of::<Self>().saturating_add(total.bytes)
    }

    pub(crate) fn decision(
        &self,
        body: &Body,
        requirements: &Arc<SourceCopyRequirements>,
        environment: Option<&Arc<Environment>>,
        ty: TypeId,
    ) -> Result<bool, &'static str> {
        if self.source != body.source
            || self.key != body.key()
            || self.instance != body.instance
            || self.signatures != signatures(body)
            || !Arc::ptr_eq(&self.requirements, requirements)
            || !same_environment(self.environment.as_ref(), environment)
        {
            return Err("source Copy decisions belong to another body or environment");
        }
        if requirements.declaration.is_none() {
            return Err("source Copy language item is missing");
        }
        self.decisions
            .get(&ty)
            .copied()
            .ok_or("source Copy read has no exact selected requirement")
    }
}
fn signatures(body: &Body) -> Option<(TypeId, TypeId)> {
    body.owner.signatures()
}
fn same_environment(a: Option<&Arc<Environment>>, b: Option<&Arc<Environment>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}

pub(crate) fn prepare(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
) -> Result<(), String> {
    prepare_scoped(facts, body, contract, None)
}

pub(in crate::body) fn prepare_scoped(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    scope: Option<&materialized::InstanceScope>,
) -> Result<(), String> {
    prepare_using(facts, body, contract, scope, None)
        .map_err(crate::sema::dispatch::Failure::message)
}
pub(crate) fn prepare_original_selected(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    preparation: &mut dyn crate::sema::dispatch::Preparation,
) -> Result<(), crate::sema::dispatch::Failure> {
    prepare_using(facts, body, contract, None, Some(preparation))
}

pub(in crate::body) fn prepare_selected(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    scope: Option<&materialized::InstanceScope>,
    preparation: &mut dyn crate::sema::dispatch::Preparation,
) -> Result<(), crate::sema::dispatch::Failure> {
    prepare_using(facts, body, contract, scope, Some(preparation))
}
fn prepare_using(
    facts: &mut Declarations,
    body: &Body,
    contract: &mut Contract,
    scope: Option<&materialized::InstanceScope>,
    mut preparation: Option<&mut dyn crate::sema::dispatch::Preparation>,
) -> Result<(), crate::sema::dispatch::Failure> {
    if scope.is_some() && facts.source_bindings.is_none() {
        return Err("instance Copy scope lost its current source catalog".into());
    }
    super::defining_cleanups::begin(facts, body, contract)?;
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return super::defining_cleanups::prepare(facts, body, contract).map_err(Into::into);
    };
    catalog.require_source_destructors_prepared()?;
    if contract.key != body.key()
        || contract.source != body.source
        || contract.signature != signatures(body).map(|(_, owned)| owned)
    {
        return Err("source Copy contract belongs to another body".into());
    }
    let (declaration, goals) = if let Some(service) = &mut preparation {
        let types = read_types(facts, body)?;
        let declaration = if types.is_empty() {
            crate::sema::compiler_items::reserved_copy_declaration(facts)?
        } else {
            service
                .copy_declarations(facts)
                .map_err(crate::sema::dispatch::Failure::Preparation)?;
            crate::sema::compiler_items::copy_declaration(facts)?
        };
        (declaration, read_goals(facts, types, declaration)?)
    } else {
        let declaration = crate::sema::compiler_items::copy_declaration(facts)?;
        (declaration, read_requirements(facts, body, declaration)?)
    };
    let requirements = Arc::new(SourceCopyRequirements { declaration, goals });
    let environment = contract
        .builtins
        .as_deref()
        .and_then(|requirements| requirements.environment.clone())
        .map_or_else(
            || reproof::fresh(facts, contract).map(Some),
            |value| Ok(Some(value)),
        )?;
    if let Some(scope) = scope {
        scope.check(facts, body, contract, environment.as_ref())?;
    }
    let decisions = if declaration.is_some() && !requirements.goals.is_empty() {
        if scope.is_none() {
            carrier::check(facts, body, contract, environment.as_ref())?;
        }
        let first = super::dispatch::copy_frontier(facts, contract)?;
        let selected = if let Some(service) = &mut preparation {
            crate::sema::compiler_items::copy_decisions_prepared(
                facts,
                environment.clone(),
                contract.universals,
                first,
                &requirements.goals,
                &mut **service,
            )?
        } else {
            crate::sema::compiler_items::copy_decisions(
                facts,
                environment.clone(),
                contract.universals,
                first,
                &requirements.goals,
            )?
        };
        contract
            .proofs
            .push(Proof::new(body.source, selected.proof));
        selected.choices.into_iter().collect()
    } else {
        HashMap::default()
    };
    let prepared = Prepared {
        key: body.key(),
        source: body.source,
        instance: body.instance,
        signatures: signatures(body),
        requirements: Arc::clone(&requirements),
        environment: environment.clone(),
        decisions,
    };
    #[cfg(test)]
    tests::record(facts, body, &prepared);
    if let Some(builtins) = &mut contract.builtins {
        let builtins = Arc::make_mut(builtins);
        builtins.environment = environment;
        builtins.set_source_copy(requirements, prepared);
    } else {
        contract.builtins = Some(Arc::new(BuiltinRequirements::source(
            environment,
            requirements,
            prepared,
        )));
    }
    super::defining_cleanups::prepare(facts, body, contract)?;
    debt_transfer::prepare(facts, body, contract, scope).map_err(Into::into)
}

fn read_requirements(
    facts: &mut Declarations,
    body: &Body,
    declaration: Option<crate::resolve::model::DefId>,
) -> Result<Vec<Predicate>, String> {
    let types = read_types(facts, body)?;
    read_goals(facts, types, declaration)
}
fn read_types(facts: &mut Declarations, body: &Body) -> Result<Vec<TypeId>, String> {
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .checked_add(body.values.len())
        .ok_or("source Copy read discovery work exhausted")?;
    let mut types = HashSet::default();
    for value in &body.values {
        if matches!(value.kind, Rvalue::Read(_)) {
            types.insert(value.ty);
        }
        if let Rvalue::Repeat {
            mode: super::ir::RepeatMode::Once,
            ..
        } = value.kind
        {
            match facts.types.kind(value.ty) {
                Some(Kind::Array { element, length }) if length > 1 => {
                    types.insert(element);
                }
                Some(Kind::ArrayTerm { element, .. }) => {
                    types.insert(element);
                }
                Some(Kind::Array { .. }) => {}
                _ => return Err("source Copy repeat lacks an array type".into()),
            }
        }
        if types.len() > 1_000_000 {
            return Err("source Copy requirement budget exceeded".into());
        }
    }
    if let Some(assembly) = &body.assembly {
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .checked_add(assembly.operands.len())
            .ok_or("source Copy assembly discovery work exhausted")?;
        for ty in assembly.operands.iter().filter_map(|operand| operand.ty) {
            types.insert(ty);
            if types.len() > 1_000_000 {
                return Err("source Copy requirement budget exceeded".into());
            }
        }
    }
    let mut types: Vec<_> = types.into_iter().collect();
    types.sort_unstable_by_key(|ty| ty.0);
    Ok(types)
}
fn read_goals(
    facts: &mut Declarations,
    types: Vec<TypeId>,
    declaration: Option<crate::resolve::model::DefId>,
) -> Result<Vec<Predicate>, String> {
    Ok(if let Some(declaration) = declaration {
        let application = facts.types.intern_trait_application(Application {
            item: ItemId(declaration.0),
            arguments: GenericArgs::EMPTY,
        })?;
        types
            .into_iter()
            .map(|ty| Predicate {
                ty,
                bound: Bound::Trait(application),
            })
            .collect()
    } else {
        Vec::new()
    })
}

/// Original proof components and displaced source rows keep their exact bodies.
/// Index bodies once, lazily for defining requirements outside source mode.
pub(crate) fn originals<'a, 'b>(
    facts: &mut Declarations,
    bodies: impl IntoIterator<Item = &'a Body>,
    contracts: impl IntoIterator<Item = &'b mut Contract>,
) -> Result<(), String> {
    originals_using(
        facts,
        bodies,
        contracts,
        &OriginalIndexBudget::new(1_000_000, 64 * 1024 * 1024),
    )
}

/// Temporary index costs have no declaration/header authority. Failed attempted
/// work and capacity payments remain sticky for this one construction owner.
pub(crate) struct OriginalIndexBudget {
    steps: std::cell::Cell<usize>,
    bytes: std::cell::Cell<usize>,
    work_limit: usize,
    byte_limit: usize,
}
impl OriginalIndexBudget {
    pub(crate) const fn new(work_limit: usize, byte_limit: usize) -> Self {
        Self {
            steps: std::cell::Cell::new(0),
            bytes: std::cell::Cell::new(0),
            work_limit,
            byte_limit,
        }
    }
    #[cfg(test)]
    pub(crate) fn costs(&self) -> (usize, usize) {
        (self.steps.get(), self.bytes.get())
    }
    fn charge(&self, work: usize, bytes: usize) -> Result<(), &'static str> {
        self.steps.set(self.steps.get().saturating_add(work));
        self.bytes.set(self.bytes.get().saturating_add(bytes));
        if self.steps.get() > self.work_limit || self.bytes.get() > self.byte_limit {
            return Err("original body index exceeds its scratch work/storage budget");
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn originals_with_budget<'a, 'b>(
    facts: &mut Declarations,
    bodies: impl IntoIterator<Item = &'a Body>,
    contracts: impl IntoIterator<Item = &'b mut Contract>,
    budget: &OriginalIndexBudget,
) -> Result<(), String> {
    originals_using(facts, bodies, contracts, budget)
}

fn originals_using<'a, 'b>(
    facts: &mut Declarations,
    bodies: impl IntoIterator<Item = &'a Body>,
    contracts: impl IntoIterator<Item = &'b mut Contract>,
    budget: &OriginalIndexBudget,
) -> Result<(), String> {
    let source = facts.source_bindings.is_some();
    let mut bodies = Some(bodies.into_iter());
    let mut index = if source {
        Some(original_index(
            bodies.take().ok_or("missing original bodies")?,
            &mut facts.body_work.canonical_steps,
            None,
        )?)
    } else {
        None
    };
    for contract in contracts {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        if !source && !contract.requires_defining_cleanup_preparation() {
            continue;
        }
        if index.is_none() {
            let mut attempted = 0;
            let prepared = original_index(
                bodies
                    .take()
                    .ok_or("original body index was consumed twice")?,
                &mut attempted,
                Some(&|work, bytes| budget.charge(work, bytes)),
            );
            facts.body_work.canonical_steps =
                facts.body_work.canonical_steps.saturating_add(attempted);
            index = Some(prepared?);
        }
        let body = index
            .as_mut()
            .ok_or("missing original body index")?
            .remove(&contract.key)
            .ok_or("source Copy original contract has no exact body")?;
        prepare(facts, body, contract)?;
    }
    if source && index.is_some_and(|index| !index.is_empty()) {
        return Err("source Copy original body has no exact contract".into());
    }
    Ok(())
}

fn original_index<'a>(
    bodies: impl Iterator<Item = &'a Body>,
    steps: &mut usize,
    payment: Option<crate::support::retained::Payment<'_>>,
) -> Result<HashMap<BodyKey, &'a Body>, String> {
    let mut index = HashMap::default();
    for body in bodies {
        *steps = steps.saturating_add(1);
        if let Some(payment) = payment {
            payment(1, 0)?;
        }
        if index.len() >= 1_000_000 || index.contains_key(&body.key()) {
            return Err("source Copy original body index is duplicate or exhausted".into());
        }
        let growth = crate::support::retained::Growth::map::<BodyKey, &Body>(
            index.len(),
            index.capacity(),
            1,
        );
        *steps = steps.saturating_add(growth.work);
        if let Some(payment) = payment {
            payment(growth.work, growth.bytes)?;
        }
        if growth.bytes > 64 * 1024 * 1024 {
            return Err("source Copy original body index exceeds storage budget".into());
        }
        index
            .try_reserve(1)
            .map_err(|_| "source Copy original body index allocation failed")?;
        index.insert(body.key(), body);
    }
    Ok(index)
}
