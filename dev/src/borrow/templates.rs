//! A sealed proof retains the exact immutable templates checked under their assumptions.
use super::{Environment, Error, loans, regions};
use crate::{
    body::{
        ir::Selection,
        regions::{Contract, Contracts},
        templates::{Template, Templates},
    },
    types::{ItemId, TypeId, variance::Variances},
};
use std::{collections::HashMap, hash::BuildHasher};

pub struct Checked<'a> {
    templates: &'a Templates,
    index: HashMap<ItemId, usize>,
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
    pub(super) fn contract(&self, body: &crate::body::ir::Body) -> Option<&regions::Body> {
        let crate::body::ir::Owner::Function { definition, .. } = body.owner else {
            return None;
        };
        let &index = self.index.get(&ItemId(definition.0))?;
        (self.templates.bodies[index].body.source == body.source)
            .then(|| &self.regions.bodies[index])
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
    for (ordinal, template) in templates.bodies.iter().enumerate() {
        let body = &template.body;
        let error = |message: &str| Error {
            node: Some(body.source),
            message: message.into(),
        };
        let crate::body::ir::Owner::Function { definition, .. } = body.owner else {
            return Err(error("template owner is not a function"));
        };
        if body.instance.is_some() || index.insert(ItemId(definition.0), ordinal).is_some() {
            return Err(error(
                "template proof requires distinct original function owners",
            ));
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
    let mut inputs: HashMap<_, _> = contracts.bodies.iter().map(|c| (c.source, c)).collect();
    let contracts = templates
        .bodies
        .iter()
        .map(|template| {
            inputs
                .remove(&template.body.source)
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
        regions,
        loans,
        contracts,
    })
}
