//! A sealed proof retains the exact immutable templates checked under their assumptions.
use super::{Environment, Error, loans, regions};
use crate::{
    body::{
        ir::{BodyKey, Owner, Selection},
        regions::{Contract, Contracts},
        templates::{Template, Templates},
    },
    types::{ItemId, TypeId, closure::ClosureId, variance::Variances},
};
use std::{collections::HashMap, hash::BuildHasher};

pub struct Checked<'a> {
    templates: &'a Templates,
    index: HashMap<ItemId, usize>,
    closures: HashMap<ClosureId, usize>,
    anonymous: HashMap<BodyKey, usize>,
    regions: regions::Analysis,
    loans: Vec<loans::Analysis>,
    contracts: Vec<Contract>,
}
impl Checked<'_> {
    pub(crate) fn is_empty(&self) -> bool {
        self.templates.bodies.is_empty()
    }
    #[must_use]
    pub fn get(&self, item: ItemId) -> Option<&Template> {
        self.index
            .get(&item)
            .map(|&index| &self.templates.bodies[index])
    }
    #[must_use]
    pub const fn regions(&self) -> &regions::Analysis {
        &self.regions
    }
    #[must_use]
    pub fn loans(&self) -> &[loans::Analysis] {
        &self.loans
    }
    #[must_use]
    pub fn input(&self, item: ItemId) -> Option<&Contract> {
        self.index.get(&item).map(|&index| &self.contracts[index])
    }
    #[must_use]
    pub fn closure(&self, identity: ClosureId) -> Option<(&Template, &Contract)> {
        let &index = self.closures.get(&identity)?;
        Some((&self.templates.bodies[index], &self.contracts[index]))
    }
    pub(super) fn contract(&self, body: &crate::body::ir::Body) -> Option<&regions::Body> {
        let &index = match body.key() {
            BodyKey::Declaration(definition) => self.index.get(&ItemId(definition.0)),
            BodyKey::Closure(identity) => self.closures.get(&identity),
            key @ BodyKey::Anonymous(_) => self.anonymous.get(&key),
            BodyKey::InitializerInstance(_)
            | BodyKey::AnonymousInstance(_)
            | BodyKey::ClosureAdapter(_) => None,
        }?;
        (self.templates.bodies[index].body.key() == body.key()).then(|| &self.regions.bodies[index])
    }
}

/// Check every template, including unused bodies, before permitting proof transfer.
/// The token borrows the original CFGs so they cannot change after verification.
/// # Errors
/// Rejects incomplete promotion, duplicate owners and invalid lifetime/ownership/loan proofs.
pub fn verify<'a, S: BuildHasher>(
    environment: &mut Environment<'_, S>,
    variances: &Variances,
    contracts: &Contracts<'_>,
    templates: &'a Templates,
    custom: &[ItemId],
    position: impl Fn(TypeId, Selection) -> Result<(u32, u32), &'static str>,
) -> Result<Checked<'a>, Error> {
    let mut index = HashMap::with_capacity(templates.bodies.len());
    let mut closures = HashMap::new();
    let mut anonymous = HashMap::new();
    for (ordinal, template) in templates.bodies.iter().enumerate() {
        let body = &template.body;
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        let duplicate = match body.owner {
            Owner::Function { definition, .. } | Owner::Initializer { definition, .. } => {
                index.insert(ItemId(definition.0), ordinal).is_some()
            }
            Owner::Closure { identity, .. } => closures.insert(identity, ordinal).is_some(),
            Owner::AnonymousConstant => anonymous.insert(body.key(), ordinal).is_some(),
            Owner::InitializerInstance { .. }
            | Owner::AnonymousInstance(_)
            | Owner::ClosureAdapter { .. } => {
                return Err(error("template requires a declared owner"));
            }
        };
        if body.instance.is_some() || duplicate {
            return Err(error("template proof requires distinct original owners"));
        }
        if body.promotions.is_none()
            && body
                .values
                .iter()
                .any(|v| matches!(v.kind, crate::body::ir::Rvalue::Borrow { .. }))
        {
            return Err(error(
                "template proof requires completed promotion analysis",
            ));
        }
    }
    let regions = regions::check_templates(environment.types, variances, contracts, templates)
        .map_err(|e| Error {
            node: e.source,
            message: e.message,
        })?;
    let loans = loans::templates::verify(environment, templates, &regions, custom, position)?;
    let mut inputs: HashMap<_, _> = contracts.bodies.iter().map(|c| (c.key, c)).collect();
    let contracts = templates
        .bodies
        .iter()
        .map(|template| {
            inputs
                .remove(&template.body.key())
                .cloned()
                .ok_or_else(|| Error {
                    node: Some(template.body.source),
                    message: "checked template lost its input contract".into(),
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Checked {
        templates,
        index,
        closures,
        anonymous,
        regions,
        loans,
        contracts,
    })
}
