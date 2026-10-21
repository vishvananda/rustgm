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
#[derive(Clone, Copy)]
enum Original<'a> {
    Ordinary,
    Factory(crate::body::primitive_methods::Permit),
    Adapter(&'a Proved, &'a crate::body::adapters::Template),
}
impl Original<'_> {
    fn validate_source(&self, types: &crate::types::Store, body: &Body) -> Result<(), Error> {
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        let adapter_source = match self {
            Original::Ordinary | Original::Factory(_) => None,
            Original::Adapter(source, template) => Some((source, template)),
        };
        match (body.owner, adapter_source) {
            (Owner::ClosureAdapter { .. }, Some((source, template))) => {
                template
                    .validate(types, source.body(), body)
                    .map_err(error)?;
            }
            (Owner::ClosureAdapter { .. }, None) => {
                return Err(error(
                    "receiver adapter requires its checked original source proof",
                ));
            }
            (_, Some(_)) => {
                return Err(error(
                    "adapter source proof cannot authorize an ordinary owner",
                ));
            }
            (_, None) => {}
        }
        Ok(())
    }
    fn validate_instance(
        &self,
        types: &crate::types::Store,
        templates: &templates::Checked<'_>,
        body: &Body,
    ) -> Result<(), Error> {
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        if matches!(self, Self::Factory(_)) && body.instance.is_none() {
            return Err(error("factory proof requires its exact selected instance"));
        }
        if let Some(instance) = body.instance {
            let selected = match (body.owner, types.kind(instance.ty())) {
                (Owner::ClosureAdapter { adapter, .. }, Some(Kind::Closure { .. })) => {
                    adapter.instance() == instance && matches!(self, Self::Adapter(..))
                }
                (
                    Owner::Function {
                        definition,
                        signature,
                    },
                    Some(Kind::FunctionItem {
                        item,
                        signature: actual,
                        ..
                    }),
                ) => definition.0 == item.0 && signature == actual,
                (
                    Owner::Closure {
                        identity,
                        signature,
                        ..
                    },
                    Some(Kind::Closure {
                        identity: actual,
                        signature: selected,
                        ..
                    }),
                ) => identity == actual && signature == selected,
                _ => false,
            };
            if !selected || types.contains_parameters(instance.ty()) != Some(false) {
                return Err(error(
                    "instance proof has a different owner or unresolved parameters",
                ));
            }
            if let Self::Factory(factory) = self {
                if factory.instance() != instance {
                    return Err(error(
                        "factory proof belongs to a different selected instance",
                    ));
                }
            } else if !matches!(self, Self::Adapter(..)) && templates.contract(body).is_none() {
                return Err(error("instance has no checked original template"));
            }
        }
        Ok(())
    }
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
        factory: Option<crate::body::primitive_methods::Permit>,
    ) -> Result<Proved, Error> {
        self.prove_inner(
            environment,
            templates,
            contract,
            body,
            catalog,
            factory.map_or(Original::Ordinary, Original::Factory),
        )
    }
    pub(crate) fn prove_adapter<S: BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        templates: &templates::Checked<'_>,
        contract: &regions::Body,
        body: Body,
        catalog: Option<&mut glue::Catalog<'_>>,
        source: (&Proved, &crate::body::adapters::Template),
    ) -> Result<Proved, Error> {
        self.prove_inner(
            environment,
            templates,
            contract,
            body,
            catalog,
            Original::Adapter(source.0, source.1),
        )
    }
    fn prove_inner<S: BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        templates: &templates::Checked<'_>,
        contract: &regions::Body,
        body: Body,
        catalog: Option<&mut glue::Catalog<'_>>,
        original: Original<'_>,
    ) -> Result<Proved, Error> {
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        original.validate_source(environment.types, &body)?;
        super::require_body_promotions(&body)?;
        if contract.key != body.key()
            || contract.source != body.source
            || contract.instance != body.instance
        {
            return Err(error("instance proof belongs to another body"));
        }
        original.validate_instance(environment.types, templates, &body)?;
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
