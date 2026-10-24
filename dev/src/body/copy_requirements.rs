//! Per-body source decisions retain exact identity and all ordinary selection debts.
use super::{
    ir::{Body, BodyKey, Rvalue},
    regions::{BuiltinRequirements, Contract, Proof},
};
use crate::{
    sema::model::{Declarations, SourceCopyRequirements},
    support::{
        bindings::LangItem,
        hash::{HashMap, HashSet},
    },
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
    super::defining_cleanups::begin(facts, body, contract)?;
    let Some(catalog) = facts.source_bindings.as_deref() else {
        return super::defining_cleanups::prepare(facts, body, contract);
    };
    catalog.require_source_destructors_prepared()?;
    if contract.key != body.key()
        || contract.source != body.source
        || contract.signature != signatures(body).map(|(_, owned)| owned)
    {
        return Err("source Copy contract belongs to another body".into());
    }
    let declaration = catalog
        .validated_language(LangItem::Copy)?
        .map(|d| d.definition);
    let goals = read_requirements(facts, body, declaration)?;
    let requirements = Arc::new(SourceCopyRequirements { declaration, goals });
    let existing = contract
        .builtins
        .as_deref()
        .and_then(|r| r.environment.clone());
    let environment = if existing.is_some() {
        existing
    } else {
        let parameters = contract
            .parameters
            .iter()
            .filter(|&&id| {
                matches!(
                    facts.types.parameter(id).map(|p| p.kind),
                    Some(crate::types::parameters::ParameterKind::Type { .. })
                )
            })
            .map(|&id| {
                (
                    id,
                    crate::types::parameters::environment::Abilities::default(),
                )
            })
            .collect::<Vec<_>>();
        Some(Arc::new(
            Environment::new(&facts.types, &parameters)?
                .with_clauses(&facts.types, &contract.type_assumptions)?,
        ))
    };
    let decisions = if declaration.is_some() && !requirements.goals.is_empty() {
        let first = super::dispatch::copy_frontier(facts, contract)?;
        let selected = crate::sema::compiler_items::copy_decisions(
            facts,
            environment.clone(),
            contract.universals,
            first,
            &requirements.goals,
        )?;
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
    super::defining_cleanups::prepare(facts, body, contract)
}

fn read_requirements(
    facts: &mut Declarations,
    body: &Body,
    declaration: Option<crate::resolve::model::DefId>,
) -> Result<Vec<Predicate>, String> {
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
    let mut types: Vec<_> = types.into_iter().collect();
    types.sort_unstable_by_key(|ty| ty.0);
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
    let source = facts.source_bindings.is_some();
    let mut bodies = Some(bodies.into_iter());
    let mut index = if source {
        Some(original_index(
            bodies.take().ok_or("missing original bodies")?,
            &mut facts.body_work.canonical_steps,
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
            index = Some(original_index(
                bodies
                    .take()
                    .ok_or("original body index was consumed twice")?,
                &mut facts.body_work.canonical_steps,
            )?);
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
) -> Result<HashMap<BodyKey, &'a Body>, String> {
    let mut index = HashMap::default();
    for body in bodies {
        *steps = steps.saturating_add(1);
        if index.len() >= 1_000_000 || index.insert(body.key(), body).is_some() {
            return Err("source Copy original body index is duplicate or exhausted".into());
        }
    }
    Ok(index)
}
