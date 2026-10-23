//! Only the solver can pair an immutable input with its actual completed proof.
use super::{Body, Error};
use crate::{
    body::{
        ir::Body as Cfg,
        regions::{Contract, Contracts},
    },
    types::{Store, variance::Variances},
};
use std::sync::Arc;

pub(crate) struct Retained {
    lineage: Lineage,
    result: Body,
}
pub(crate) struct Lineage {
    group: Arc<[Arc<Contract>]>,
    ordinal: usize,
}
impl Lineage {
    pub(crate) fn input(&self) -> &Arc<Contract> {
        &self.group[self.ordinal]
    }
    pub(crate) fn len(&self) -> usize {
        self.group.len()
    }
    pub(crate) fn ordinal(&self) -> usize {
        self.ordinal
    }
    pub(crate) fn same_group(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.group, &other.group)
    }
}
impl Retained {
    pub(crate) fn input(&self) -> &Arc<Contract> {
        self.lineage.input()
    }
    pub(crate) fn result(&self) -> &Body {
        &self.result
    }
    pub(crate) fn into_parts(self) -> (Lineage, Body) {
        (self.lineage, self.result)
    }
}

pub(crate) fn check_retained_group(
    types: &Store,
    variances: &Variances,
    contracts: &Contracts<'_>,
    inputs: Vec<(Arc<Contract>, &Cfg)>,
) -> Result<Vec<Retained>, Error> {
    let error = |message: &str| Error {
        source: None,
        message: message.into(),
    };
    let bytes = inputs
        .len()
        .saturating_mul(size_of::<Retained>() + size_of::<Arc<Contract>>());
    if inputs.len() > 1_000_000 || bytes > 64 * 1024 * 1024 {
        return Err(error(
            "retained conditional proof group exceeds storage budget",
        ));
    }
    let results = super::check_nested_bodies(
        types,
        variances,
        contracts,
        inputs.iter().map(|(input, cfg)| Ok((input.as_ref(), *cfg))),
    )?;
    if results.bodies.len() != inputs.len() {
        return Err(error("retained conditional proof lost its group coverage"));
    }
    let group: Arc<[Arc<Contract>]> = inputs.into_iter().map(|(input, _)| input).collect();
    results
        .bodies
        .into_iter()
        .enumerate()
        .map(|(ordinal, result)| {
            let input = &group[ordinal];
            if result.key != input.key || result.source != input.source || result.instance.is_some()
            {
                return Err(error(
                    "retained conditional proof changed its original identity",
                ));
            }
            Ok(Retained {
                lineage: Lineage {
                    group: Arc::clone(&group),
                    ordinal,
                },
                result,
            })
        })
        .collect()
}
