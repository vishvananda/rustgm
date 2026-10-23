//! Paid-parent authority owns the actual proved body and its solver-paired input.
use super::{Environment, Error, Proved, Verifier};
use crate::support::hash::HashSet;
use crate::{
    body::{
        conditional_inputs::{Pending, Port},
        ir::Body,
        regions::{Contract, Kind},
    },
    borrow::{destruction::glue, regions, templates},
};
use std::{hash::BuildHasher, rc::Rc, sync::Arc};

pub(crate) struct OwnedParent {
    inner: Rc<Inner>,
}
pub(crate) struct ScopeOwner {
    inner: Rc<Inner>,
}
struct Inner {
    paired: super::PairedProved,
    pending: Pending,
}
pub(crate) struct PaidInline<'a> {
    parent: &'a Inner,
    port: &'a Port,
}
impl PaidInline<'_> {
    pub(crate) fn port(&self) -> &Port {
        self.port
    }
    pub(crate) fn conditions(
        &self,
        input: &Contract,
        body: &Body,
    ) -> Result<Option<&[crate::types::outlives::Pair]>, &'static str> {
        if self.parent.input().key != self.parent.body().key()
            || self.parent.input().source != self.parent.body().source
        {
            return Err("paid parent lost its immutable owner lineage");
        }
        if input.key != crate::body::ir::BodyKey::AnonymousInstance(self.port.instance)
            || input.source != self.port.instance.source().into()
            || body.key() != input.key
            || body.source != input.source
            || body.instance.is_some()
            || !input
                .output
                .as_ref()
                .is_some_and(|link| Arc::ptr_eq(link, &self.port.link))
        {
            return Err("conditional child proof belongs to another paid interface");
        }
        Ok(self.port.conditions.as_deref())
    }
}
impl Inner {
    pub(crate) fn body(&self) -> &Body {
        self.paired.body()
    }
    pub(crate) fn pending(&self) -> &Pending {
        &self.pending
    }
    fn input(&self) -> &Contract {
        self.paired.input()
    }
    fn validate(&self) -> Result<(), &'static str> {
        self.pending.validate_body(self.body(), self.input())
    }
    fn paid<'a>(&'a self, port: &'a Port) -> Result<PaidInline<'a>, &'static str> {
        if !self
            .pending
            .ports()
            .get(port.ordinal())
            .is_some_and(|own| std::ptr::eq(own, port))
        {
            return Err("conditional port does not belong to this actual paid parent");
        }
        Ok(PaidInline { parent: self, port })
    }
}
impl OwnedParent {
    pub(crate) const fn storage_overhead() -> usize {
        size_of::<Inner>() + 2 * size_of::<usize>()
    }

    pub(crate) fn body(&self) -> &Body {
        self.inner.body()
    }
    pub(crate) fn paired(&self) -> &super::PairedProved {
        &self.inner.paired
    }
    pub(super) fn scope_owner(&self) -> ScopeOwner {
        ScopeOwner {
            inner: Rc::clone(&self.inner),
        }
    }
    pub(crate) fn into_proved(self) -> Result<Proved, (Self, &'static str)> {
        match Rc::try_unwrap(self.inner) {
            Ok(inner) => Ok(inner.paired.into_proved()),
            Err(inner) => Err((
                Self { inner },
                "paid parent still has an active ownership guard",
            )),
        }
    }
}
impl ScopeOwner {
    pub(crate) fn validate(&self) -> Result<(), &'static str> {
        self.inner.validate()
    }
    pub(crate) fn body(&self) -> &Body {
        self.inner.body()
    }
    pub(crate) fn input(&self) -> &Contract {
        self.inner.input()
    }
    pub(crate) fn pending(&self) -> &Pending {
        self.inner.pending()
    }
    pub(crate) fn paid<'a>(&'a self, port: &'a Port) -> Result<PaidInline<'a>, &'static str> {
        self.inner.paid(port)
    }
}

impl Verifier {
    pub(crate) fn prove_parent<S: BuildHasher>(
        &mut self,
        environment: &mut Environment<'_, S>,
        templates: &templates::Checked<'_>,
        retained: regions::Retained,
        body: Body,
        pending: (Pending, &mut usize),
        catalog: Option<&mut glue::Catalog<'_>>,
    ) -> Result<OwnedParent, Error> {
        let (pending, work) = pending;
        *work = work.saturating_add(pending.ports().len());
        let error = |message: &str| Error {
            node: Some(body.source.into()),
            message: message.into(),
        };
        pending
            .validate_body(&body, retained.input())
            .map_err(error)?;
        validate_ports(templates, &pending, &retained, work).map_err(error)?;
        let paired = self.prove_paired(environment, templates, retained, body, catalog)?;
        Ok(OwnedParent {
            inner: Rc::new(Inner { paired, pending }),
        })
    }
}
fn validate_ports(
    templates: &templates::Checked<'_>,
    pending: &Pending,
    retained: &regions::Retained,
    work: &mut usize,
) -> Result<(), &'static str> {
    if crate::support::retained::new_map::<
        (crate::body::ir::NodeKey, crate::types::outlives::Pair),
        (),
    >(retained.input().obligations.len())
        > 64 * 1024 * 1024
    {
        return Err("conditional parent debt index exceeds storage budget");
    }
    let goals: HashSet<_> = retained
        .input()
        .obligations
        .iter()
        .filter_map(|goal| {
            *work = work.saturating_add(1);
            if let Kind::Outlives(pair) = goal.kind {
                Some((goal.source, pair))
            } else {
                None
            }
        })
        .collect();
    for port in pending.ports() {
        *work = work.saturating_add(1);
        let (original, summary) = templates.conditional_original(port.instance.source())?;
        let (parent, requirements) = summary
            .parent_goals(original)?
            .ok_or("paid child original is not conditional")?;
        if parent != retained.input().source
            || !original
                .output
                .as_ref()
                .is_some_and(|link| Arc::ptr_eq(link, &port.link))
            || !same_conditions(
                summary.execution_conditions(original)?,
                port.conditions.as_ref(),
            )
            || port.slots.len() != port.link.bridges.len()
        {
            return Err("paid child changed its checked original interface");
        }
        for (&(slot, actual), bridge) in port.slots.iter().zip(&port.link.bridges) {
            *work = work.saturating_add(1);
            if slot != bridge.slot
                || actual != bridge.parent
                || retained.result().identities.get(actual).is_none()
            {
                return Err("paid child mapping is not owned by its actual parent proof");
            }
        }
        for requirement in requirements {
            *work = work.saturating_add(1);
            let Kind::Outlives(pair) = requirement.kind else {
                return Err("paid child formation transport is not implemented");
            };
            if !goals.contains(&(requirement.source, pair)) {
                return Err("conditional child debt is absent from its actual parent contract");
            }
        }
    }
    Ok(())
}
fn same_conditions(
    a: Option<&Arc<[crate::types::outlives::Pair]>>,
    b: Option<&Arc<[crate::types::outlives::Pair]>>,
) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Arc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
