//! Owned immutable bodies let demand discovery append proofs without revisiting earlier CFGs.
use super::{Checked, Environment, Error};
use crate::{
    body::{
        drops,
        ir::{Body, Owner, Program},
    },
    borrow::{
        destruction::{self, glue},
        regions, templates,
    },
    types::{Kind, outlives::LiveRegions},
};
use std::hash::BuildHasher;

pub(crate) struct Proved {
    body: Body,
    plan: Option<destruction::Plan>,
    destruction: bool,
    static_targets: Option<Box<crate::types::substitute::Bound>>,
}
impl Proved {
    pub(crate) fn resolve_callables(
        &mut self,
        types: &mut crate::types::Store,
    ) -> Result<(), &'static str> {
        if let Some(mut fold) = self.static_targets.take() {
            super::static_targets::resolve(types, &mut self.body, &mut fold)?;
        }
        Ok(())
    }
    pub(crate) const fn body(&self) -> &Body {
        &self.body
    }
    pub(crate) fn plan(&self) -> Option<&destruction::Plan> {
        self.plan.as_ref()
    }
}

pub(crate) struct Verifier {
    components: LiveRegions,
    drops: Option<crate::types::destruction::regions::Regions>,
}
impl Verifier {
    pub(crate) fn new(methods: Option<&[drops::Method]>) -> Result<Self, &'static str> {
        Ok(Self {
            components: LiveRegions::default(),
            drops: methods
                .map(|methods| {
                    crate::types::destruction::regions::Regions::new(methods.iter().map(|m| m.item))
                })
                .transpose()?,
        })
    }
    pub(crate) fn prove<S: BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        templates: &templates::Checked<'_>,
        contract: &regions::Body,
        body: Body,
        catalog: Option<&mut glue::Catalog<'_>>,
    ) -> Result<Proved, Error> {
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        super::require_body_promotions(&body)?;
        if contract.key != body.key()
            || contract.source != body.source
            || contract.instance != body.instance
        {
            return Err(error("instance proof belongs to another body"));
        }
        if let Some(instance) = body.instance {
            let Some(Kind::FunctionItem {
                item, signature, ..
            }) = environment.types.kind(instance.ty())
            else {
                return Err(error("instance proof requires a function item"));
            };
            if !matches!(body.owner, Owner::Function { definition, signature: actual }
                if definition.0 == item.0 && actual == signature)
                || environment.types.contains_parameters(instance.ty()) != Some(false)
            {
                return Err(error(
                    "instance proof has a different owner or unresolved parameters",
                ));
            }
            if templates.contract(&body).is_none() {
                return Err(error("instance has no checked original template"));
            }
        }
        let plan = match (&mut self.drops, catalog) {
            (Some(drops), Some(catalog)) => super::prove_cleanup(
                environment,
                &mut self.components,
                drops,
                &body,
                contract,
                catalog,
            )?,
            (None, None) => {
                super::prove_body::<false, _>(environment, &mut self.components, &body, contract)?;
                None
            }
            _ => {
                return Err(error(
                    "instance cleanup proof has a different destructor environment",
                ));
            }
        };
        Ok(Proved {
            static_targets: super::static_targets::prepare(contract).map_err(error)?,
            body,
            plan,
            destruction: self.drops.is_some(),
        })
    }
}

/// Consume immutable proved bodies in their final order, then lend the existing
/// executable token. No caller can mutate the owned bodies between proof and use.
pub(crate) fn with_checked<T>(
    mut program: Program,
    bodies: Vec<Proved>,
    recipes: Option<glue::Recipes>,
    use_checked: impl FnOnce(&Checked<'_>) -> T,
) -> Result<T, Error> {
    if !program.bodies.is_empty()
        || program.drops.is_some() != recipes.is_some()
        || program
            .drops
            .as_ref()
            .is_some_and(|pending| pending.bodies.len() != bodies.len())
        || bodies
            .iter()
            .any(|body| body.destruction != recipes.is_some())
    {
        return Err(Error {
            node: None,
            message: "instance assembly contains unproved bodies or inconsistent cleanup".into(),
        });
    }
    let mut plans = Vec::with_capacity(bodies.len());
    for proved in bodies {
        program.bodies.push(proved.body);
        plans.push(proved.plan);
    }
    let destruction = recipes.map(|recipes| Box::new(destruction::Proof { plans, recipes }));
    Ok(use_checked(&Checked {
        program: &program,
        destruction,
    }))
}
