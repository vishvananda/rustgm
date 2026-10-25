//! Original aliases authenticate transport; concrete instances reselect their own reads.
use super::{Arc, Body, BodyKey, Contract, Declarations, Environment, Instance, TypeId};
use crate::{body::ir::BodyOrigin, types::substitute::Bound};

pub(in crate::body) struct Original {
    key: BodyKey,
    source: BodyOrigin,
    environment: Arc<Environment>,
    instance: Instance,
    arena: crate::types::arena::Identity,
}

pub(in crate::body) struct InstanceScope {
    original: Original,
    signatures: (TypeId, TypeId),
    universals: u32,
    environment: Arc<Environment>,
}

pub(in crate::body) fn capture(
    facts: &mut Declarations,
    body: &Body,
    contract: &Contract,
    instance: Instance,
) -> Result<Option<Original>, &'static str> {
    if facts.source_bindings.is_none() {
        return Ok(None);
    }
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
    let environment = contract
        .builtins
        .as_deref()
        .and_then(|input| input.environment.as_ref());
    let actual = body
        .source
        .source()
        .and_then(|source| facts.generics.builtin_requirements.get(&source))
        .and_then(|input| input.environment.as_ref());
    let aliases = environment.is_some_and(|environment| environment.body_aliases().is_some())
        || actual.is_some_and(|environment| environment.body_aliases().is_some());
    let predicates = contract
        .builtins
        .as_deref()
        .map_or(&[][..], |input| input.predicates.as_slice());
    facts.body_work.canonical_steps = facts
        .body_work
        .canonical_steps
        .saturating_add(predicates.len());
    if predicates.len() > crate::support::crates::MAX_COMPONENTS {
        return Err("instance trait scope requirement budget exceeded");
    }
    let traits = predicates.iter().any(|(_, goal)| {
        matches!(
            goal.bound,
            crate::types::parameters::predicates::Bound::Trait(_)
        )
    });
    if !aliases && !traits {
        return Ok(None);
    }
    let environment = environment
        .cloned()
        .ok_or("instance lost its original alias environment")?;
    super::carrier::check(facts, body, contract, Some(&environment))?;
    if !aliases {
        let crate::body::ir::Owner::Function {
            definition,
            signature,
            execution: None,
        } = body.owner
        else {
            return Err("ordinary trait instance scope requires its actual function original");
        };
        let header = facts
            .function_header(definition)?
            .ok_or("instance trait scope lost its original function header")?;
        if header.signature != signature || header.universals != contract.universals {
            return Err("instance trait scope changed its original signature or universals");
        }
        let current = original_environment(facts, body.key(), body.source, false)?;
        if !Arc::ptr_eq(current, &environment) {
            return Err("instance trait scope changed its original environment");
        }
    }
    let bytes = size_of::<Original>()
        .saturating_add(size_of::<InstanceScope>())
        .saturating_add(crate::types::arena::Identity::allocation_bytes());
    if facts.constant_work.template_bytes.saturating_add(bytes) > 64 * 1024 * 1024 {
        return Err("instance source scope exceeds aggregate storage budget");
    }
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
    let arena = facts.types.arena_identity();
    Ok(Some(Original {
        key: body.key(),
        source: body.source,
        environment,
        instance,
        arena,
    }))
}

impl Original {
    pub(in crate::body) fn finish(
        self,
        facts: &mut Declarations,
        body: &Body,
        contract: &mut Contract,
    ) -> Result<Box<InstanceScope>, &'static str> {
        if !facts.types.has_identity(&self.arena) {
            return Err("instance source scope belongs to another type arena");
        }
        let signatures = body
            .owner
            .signatures()
            .ok_or("instance lost its signature pair")?;
        if body.instance != Some(self.instance)
            || body.source != self.source
            || body.key() != self.key
            || contract.key != self.key
            || contract.source != self.source
            || contract.signature != Some(signatures.1)
        {
            return Err("materialization changed its authenticated original identity");
        }
        let Some(crate::types::Kind::FunctionItem {
            item, signature, ..
        }) = facts.types.kind(self.instance.ty())
        else {
            return Err("alias materialization requires a full function item");
        };
        if self.key != BodyKey::Declaration(crate::resolve::model::DefId(item.0))
            || signature != signatures.0
        {
            return Err("alias materialization changed its selected public signature");
        }
        let environment = super::reproof::fresh(facts, contract)?;
        let pending = contract
            .builtins
            .as_mut()
            .ok_or("instance lost its builtin inputs")?;
        let pending = Arc::make_mut(pending);
        if pending.environment.is_some() {
            return Err("instance retained its original alias environment");
        }
        pending.environment = Some(Arc::clone(&environment));
        let scope = InstanceScope {
            original: self,
            signatures,
            universals: contract.universals,
            environment,
        };
        scope.check_storage(facts)?;
        Ok(Box::new(scope))
    }
}

impl InstanceScope {
    fn allocation(&self) -> crate::support::retained::Allocation {
        let mut observer = crate::types::parameters::environment::Retained::default();
        let mut allocation = self
            .environment
            .retain_owned_from(&self.original.environment, &mut observer);
        allocation.add(size_of::<Self>());
        allocation.add(crate::types::arena::Identity::allocation_bytes());
        allocation
    }
    fn check_storage(&self, facts: &mut Declarations) -> Result<(), &'static str> {
        let allocation = self.allocation();
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(allocation.visits);
        if facts
            .constant_work
            .template_bytes
            .saturating_add(allocation.bytes)
            > 64 * 1024 * 1024
        {
            return Err("instance source scope exceeds aggregate storage budget");
        }
        Ok(())
    }
    #[cfg(test)]
    pub(in crate::body) fn retained_bytes(&self) -> usize {
        self.allocation().bytes
    }

    pub(in crate::body) fn check(
        &self,
        facts: &mut Declarations,
        body: &Body,
        contract: &Contract,
        environment: Option<&Arc<Environment>>,
    ) -> Result<(), &'static str> {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        if !facts.types.has_identity(&self.original.arena) {
            return Err("instance Copy scope belongs to another materialization or environment");
        }
        let aliases = self.original.environment.body_aliases().is_some();
        let actual = original_environment(facts, self.original.key, self.original.source, aliases)?;
        if facts.source_bindings.is_none()
            || body.instance != Some(self.original.instance)
            || body.key() != self.original.key
            || body.source != self.original.source
            || body.owner.signatures() != Some(self.signatures)
            || !matches!(
                body.owner,
                crate::body::ir::Owner::Function {
                    execution: Some(_),
                    ..
                }
            )
            || contract.key != self.original.key
            || contract.source != self.original.source
            || contract.signature != Some(self.signatures.1)
            || contract.universals != self.universals
            || !contract.parameters.is_empty()
            || !Arc::ptr_eq(actual, &self.original.environment)
            || environment.is_none_or(|actual| !Arc::ptr_eq(actual, &self.environment))
            || self.environment.body_aliases().is_some()
            || Arc::ptr_eq(&self.environment, &self.original.environment)
        {
            return Err("instance Copy scope belongs to another materialization or environment");
        }
        Ok(())
    }

    pub(in crate::body) fn rewrite(
        &mut self,
        facts: &mut Declarations,
        body: &Body,
        contract: &mut Contract,
        fold: &mut Bound,
    ) -> Result<(), &'static str> {
        self.check(
            facts,
            body,
            contract,
            contract
                .builtins
                .as_deref()
                .and_then(|input| input.environment.as_ref()),
        )?;
        let before = fold.steps;
        let mapped = self.environment.substitute_regions(&mut facts.types, fold);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(fold.steps.saturating_sub(before));
        let environment = Arc::new(mapped?);
        let before = fold.steps;
        let signature = fold.apply(&mut facts.types, self.signatures.1);
        facts.body_work.canonical_steps = facts
            .body_work
            .canonical_steps
            .saturating_add(fold.steps.saturating_sub(before));
        self.signatures.1 = signature?;
        let pending = contract
            .builtins
            .as_mut()
            .ok_or("rewritten instance lost its builtin inputs")?;
        Arc::make_mut(pending).environment = Some(Arc::clone(&environment));
        self.environment = environment;
        self.check_storage(facts)
    }
}

fn original_environment(
    facts: &Declarations,
    key: BodyKey,
    source: BodyOrigin,
    aliases: bool,
) -> Result<&Arc<Environment>, &'static str> {
    if let Some(input) = source
        .source()
        .and_then(|source| facts.generics.builtin_requirements.get(&source))
    {
        return input
            .environment
            .as_ref()
            .ok_or("instance source scope lost its actual builtin environment");
    }
    if aliases {
        return Err("instance source scope lost its actual alias ledger");
    }
    let BodyKey::Declaration(definition) = key else {
        return Err("nonalias trait scope requires its actual function owner");
    };
    let header = facts
        .function_header(definition)?
        .ok_or("instance source scope lost its original header")?;
    if header.body != Some(source) {
        return Err("instance source scope changed its actual original header");
    }
    header
        .environment
        .ok_or("instance source scope lost its original header environment")
}
