//! Original aliases authenticate transport; concrete instances reselect their own reads.
use super::{Arc, Body, BodyKey, Contract, Declarations, Environment, Instance, TypeId};
use crate::{body::ir::BodyOrigin, types::substitute::Bound};

pub(in crate::body) struct Original {
    key: BodyKey,
    source: BodyOrigin,
    environment: Arc<Environment>,
    instance: Instance,
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
    let environment = contract
        .builtins
        .as_deref()
        .and_then(|input| input.environment.as_ref());
    let actual = body
        .source
        .source()
        .and_then(|source| facts.generics.builtin_requirements.get(&source))
        .and_then(|input| input.environment.as_ref());
    if environment.is_none_or(|environment| environment.body_aliases().is_none())
        && actual.is_none_or(|environment| environment.body_aliases().is_none())
    {
        return Ok(None);
    }
    let environment = environment
        .cloned()
        .ok_or("instance lost its original alias environment")?;
    super::carrier::check(facts, body, contract, Some(&environment))?;
    Ok(Some(Original {
        key: body.key(),
        source: body.source,
        environment,
        instance,
    }))
}

impl Original {
    pub(in crate::body) fn finish(
        self,
        facts: &mut Declarations,
        body: &Body,
        contract: &mut Contract,
    ) -> Result<Box<InstanceScope>, &'static str> {
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
        let environment = super::environment::fresh(facts, contract)?;
        let pending = contract
            .builtins
            .as_mut()
            .ok_or("instance lost its builtin inputs")?;
        let pending = Arc::make_mut(pending);
        if pending.environment.is_some() {
            return Err("instance retained its original alias environment");
        }
        pending.environment = Some(Arc::clone(&environment));
        Ok(Box::new(InstanceScope {
            original: self,
            signatures,
            universals: contract.universals,
            environment,
        }))
    }
}

impl InstanceScope {
    #[cfg(test)]
    pub(in crate::body) fn retained_bytes_for_test(&self) -> usize {
        let mut observer = crate::types::parameters::environment::Retained::default();
        size_of::<Self>().saturating_add(
            self.environment
                .retain_owned_from(&self.original.environment, &mut observer)
                .bytes,
        )
    }

    pub(in crate::body) fn check(
        &self,
        facts: &mut Declarations,
        body: &Body,
        contract: &Contract,
        environment: Option<&Arc<Environment>>,
    ) -> Result<(), &'static str> {
        facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(1);
        let actual = self
            .original
            .source
            .source()
            .and_then(|source| facts.generics.builtin_requirements.get(&source))
            .and_then(|input| input.environment.as_ref());
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
            || actual.is_none_or(|actual| !Arc::ptr_eq(actual, &self.original.environment))
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
        Ok(())
    }
}
