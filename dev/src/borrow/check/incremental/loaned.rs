//! A selected deferred body finishes ordinary loans before its single owned transform.
use super::{Environment, Error, Verifier, cstage::Completed, glue, regions};
use crate::{
    body::ir::{Body, Owner},
    borrow::{check::cstage, loans::seal::Seal, templates},
    types::Kind,
};
use std::ptr::NonNull;

impl Verifier {
    /// This completion is consumed while the actual application remains borrowed.
    pub(crate) fn prove_async_application<S: std::hash::BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        templates: &templates::Checked<'_>,
        body: &Body,
        input: &crate::body::regions::Contract,
        contract: &regions::Body,
        catalog: &mut glue::Catalog<'_>,
    ) -> Result<Completed, Error> {
        let error = |message: &str| Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        Seal::require_arenas(environment).map_err(error)?;
        require_family(environment.types, templates, body, input, contract).map_err(error)?;
        super::super::require_body_promotions(body)?;
        let prepared = cstage::prepare(
            environment,
            &mut self.reused,
            body,
            contract,
            catalog,
            super::super::structure::Use::Own,
            None,
        )?;
        let targets = super::super::static_targets::prepare(contract).map_err(error)?;
        let drops = self
            .drops
            .as_mut()
            .ok_or_else(|| error("async application lost its actual cleanup environment"))?;
        let (plan, seal) = cstage::finish_sealed(
            environment,
            &mut self.reused,
            drops,
            prepared,
            contract,
            None,
            cstage::RegionUse::Initial,
        )?;
        Ok(Completed {
            body: NonNull::from(body),
            plan: plan.map(Box::new),
            targets,
            seal: Some(Box::new(seal)),
        })
    }
}

impl Completed {
    /// The finite scalar transform cannot erase an actual cleanup or static mapping.
    pub(crate) fn consume_empty_application<S: std::hash::BuildHasher>(
        self,
        environment: &Environment<'_, S>,
        body: &Body,
        contract: &regions::Body,
    ) -> Result<(), &'static str> {
        if self.body != NonNull::from(body) || !matches!(body.owner, Owner::AsyncApplication { .. })
        {
            return Err("async completion belongs to another actual immutable application");
        }
        self.seal
            .as_ref()
            .ok_or("async completion lacks its actual retained seal")?
            .validate(environment, body, contract, None)?;
        if self.plan.is_some() || self.targets.is_some() {
            return Err("async poll requires its actual cleanup and callable-target transform");
        }
        Ok(())
    }
}

fn require_family(
    types: &crate::types::Store,
    templates: &templates::Checked<'_>,
    body: &Body,
    input: &crate::body::regions::Contract,
    contract: &regions::Body,
) -> Result<(), &'static str> {
    let Owner::AsyncApplication {
        identity,
        future,
        signature,
    } = body.owner
    else {
        return Err("deferred proof requires its actual selected AsyncApplication");
    };
    templates.future_store(types)?;
    let (template, original, original_regions) = templates
        .async_body(identity)
        .ok_or("deferred proof lost its actual checked source family")?;
    if !matches!(types.kind(future), Some(Kind::Future { identity:actual, .. }) if actual==identity)
        || !matches!(template.body.owner, Owner::AsyncBody {identity:actual, ..} if actual==identity)
        || body.instance.is_some()
        || body.const_context
        || body.source != template.body.source
        || original.key != template.body.key()
        || original_regions.key != original.key
        || original_regions.source != template.body.source
        || contract.key != body.key()
        || contract.source != body.source
        || contract.instance.is_some()
        || input.key != body.key()
        || input.source != body.source
        || input.signature != Some(signature)
    {
        return Err("deferred proof changed its checked owner, application or signature");
    }
    Ok(())
}
