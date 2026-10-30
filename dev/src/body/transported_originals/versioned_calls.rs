//! One transient member-link index binds selected CFG calls to fresh trait dispatch.
use super::ImportedOriginals;
use crate::{
    body::ir::{Body, Rvalue},
    resolve::model::DefId,
    sema::{crates::SourceOriginal, dispatch::Dispatcher, model::Declarations},
    support::{
        crates::{
            self,
            generics::{self as g, source as s},
            owner_catalog::DeclarationRef,
        },
        hash::HashMap,
        retained,
    },
    types::{ItemId, Kind, TypeId, arguments::Argument, parameters::environment::Environment},
};
use std::{cell::Cell, sync::Arc};
mod applied_members;
#[cfg(test)]
#[path = "../../../tests/invariants/envelope_payments.rs"]
mod tests;
#[derive(Clone, Copy)]
struct Member {
    owner: DeclarationRef,
    declaration: DeclarationRef,
    subject: g::TypeRef,
}
pub(super) struct Index<'a> {
    rows: s::ValidatedView<'a>,
    members: HashMap<DeclarationRef, Member>,
    bytes: usize,
}
#[derive(Clone, Copy, Default)]
pub(super) enum Pending {
    #[default]
    Unused,
    Primitive {
        member: DefId,
        subject: TypeId,
        signature: TypeId,
        selected: TypeId,
    },
    Applied {
        member: DefId,
        selected: TypeId,
        recorded: Option<crate::types::parameters::predicates::Predicate>,
    },
}
impl<'a> Index<'a> {
    pub(super) fn new(
        rows: s::ValidatedView<'a>,
        work: &mut ImportedOriginals,
        allowance: &super::decoded_bodies::Allowance,
    ) -> Result<Self, &'static str> {
        let mut result = Self {
            rows,
            members: HashMap::default(),
            bytes: 0,
        };
        if !rows.descriptor().profile().feature_predicates() {
            return Ok(result);
        }
        let growth = retained::new_map::<DeclarationRef, Member>(rows.member_links.len());
        work.charge(
            rows.implementations
                .len()
                .saturating_add(rows.member_links.len()),
        )?;
        if allowance
            .header_bytes
            .saturating_add(work.retained_bytes())
            .saturating_add(growth)
            > crates::MAX_BYTES
            || work.retained_bytes().saturating_add(growth) > allowance.templates
            || allowance.header_work.saturating_add(work.steps) > crates::MAX_COMPONENTS
        {
            return Err("selected source member index exceeds its actual storage envelope");
        }
        result.bytes = growth;
        work.payload = work.payload.saturating_add(growth);
        result.members.reserve(rows.member_links.len());
        for header in &rows.implementations {
            for link in header.members.slice(&rows.member_links)? {
                let (Some(actual), Some(expected)) = (link.implementation, link.trait_member)
                else {
                    return Err("v4 selected methods require actual override/member links");
                };
                if expected.artifact != s::ArtifactRef(0)
                    || result
                        .members
                        .insert(
                            actual,
                            Member {
                                owner: header.self_definition,
                                declaration: expected.declaration,
                                subject: header.self_type,
                            },
                        )
                        .is_some()
                {
                    return Err("selected source member has a foreign or repeated actual owner");
                }
            }
        }
        Ok(result)
    }
    pub(super) fn release(self, work: &mut ImportedOriginals) {
        work.payload = work.payload.saturating_sub(self.bytes);
    }
    pub(super) fn prepare(
        &self,
        view: &SourceOriginal<'_>,
        facts: &Declarations,
        body: &Body,
        work: &mut ImportedOriginals,
    ) -> Result<Vec<Pending>, &'static str> {
        if !std::ptr::eq(&raw const *self.rows, &raw const *view.rows()) {
            return Err("selected source member index belongs to another actual descriptor");
        }
        if self.rows.descriptor().profile().conversion_roles() {
            return applied_members::prepare(self, view, facts, body, work);
        }
        let mut pending = Vec::new();
        for (record, value) in view.raw().values.iter().zip(&body.values) {
            work.charge(1)?;
            let s::ValueKind::Function(reference) = record.kind else {
                continue;
            };
            let target = self.rows.function_applications[reference.0 as usize]
                .declaration
                .declaration;
            let Some(member) = self.members.get(&target) else {
                continue;
            };
            let Some(s::HeaderRef::Function(index)) = self.rows.header(target)? else {
                return Err("selected member lost its actual function header");
            };
            if self.rows.functions[index as usize].parent != Some(member.owner) {
                return Err("selected member changed its actual parent implementation");
            }
            let definition = view.declaration(member.declaration, &mut work.steps)?;
            let signature = facts
                .function_header(definition)?
                .ok_or("selected method lacks its installed trait header")?
                .signature;
            if !matches!(value.kind, Rvalue::Function(_))
                || !matches!(facts.types.kind(value.ty),Some(Kind::FunctionItem{signature:actual,..}) if actual==signature)
            {
                return Err("selected method requires its exact closed primitive trait signature");
            }
            pending.push(Pending::Primitive {
                member: definition,
                subject: view.ty(member.subject, &mut work.steps)?,
                signature,
                selected: value.ty,
            });
        }
        Ok(pending)
    }
}
pub(super) fn allocation(values: usize) -> usize {
    values.saturating_mul(4 * size_of::<Pending>())
}
pub(super) fn prove(
    facts: &mut Declarations,
    environment: Arc<Environment>,
    pending: &[Pending],
    body: &Body,
    contract: &crate::body::regions::Contract,
    work: &mut ImportedOriginals,
    allowance: &super::decoded_bodies::Allowance,
) -> Result<(), String> {
    if pending.is_empty() {
        return Ok(());
    }
    paying(work, allowance, |pay| {
        pay(pending.len(), 0)?;
        let applied = pending.iter().any(|p| matches!(p, Pending::Applied { .. }));
        let (universals, first) = if applied {
            if contract.key != body.key() || contract.source != body.source {
                return Err("selected source dispatch changed its actual body contract".into());
            }
            pay(
                contract
                    .obligations
                    .len()
                    .saturating_add(contract.proofs.len())
                    .saturating_add(1),
                0,
            )?;
            (
                contract.universals,
                crate::body::dispatch::imported_region_frontier(facts, contract)?,
            )
        } else {
            (0, 1)
        };
        let mut dispatcher =
            Dispatcher::in_body(&facts.types, universals, first, environment, &pay)?;
        for goal in pending {
            let Some((requested, selected)) = request(facts, *goal, &pay)? else {
                continue;
            };
            if dispatcher.select(facts, requested)? != selected {
                return Err(
                    "fresh source trait dispatch selected another implementation function".into(),
                );
            }
        }
        if !dispatcher.finish().is_empty() {
            return Err(
                "closed primitive source dispatch retained unsupported lifetime debts".into(),
            );
        }
        Ok(())
    })
}

fn paying<T>(
    work: &mut ImportedOriginals,
    allowance: &super::decoded_bodies::Allowance,
    query: impl FnOnce(retained::Payment<'_>) -> Result<T, String>,
) -> Result<T, String> {
    let steps = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let base = work.retained_bytes();
    let pay = |count, allocation| {
        steps.set(steps.get().saturating_add(count));
        bytes.set(bytes.get().saturating_add(allocation));
        if allowance
            .header_work
            .saturating_add(work.steps)
            .saturating_add(steps.get())
            > crates::MAX_COMPONENTS
            || allowance
                .header_bytes
                .saturating_add(base)
                .saturating_add(bytes.get())
                > crates::MAX_BYTES
            || base.saturating_add(bytes.get()) > allowance.templates
        {
            return Err("selected source trait proof exceeds its actual work/storage envelope");
        }
        Ok(())
    };
    let result = query(&pay);
    work.steps = work.steps.saturating_add(steps.get());
    // Canonical interning survives the query. Retain its prepaid allocation
    // bound even on failure; temporary argument/dispatcher scratch deliberately
    // remains a conservative part of the same finite storage envelope.
    work.payload = work.payload.saturating_add(bytes.get());
    result
}

fn request(
    facts: &mut Declarations,
    goal: Pending,
    pay: retained::Payment<'_>,
) -> Result<Option<(TypeId, TypeId)>, &'static str> {
    Ok(Some(match goal {
        Pending::Unused => return Ok(None),
        Pending::Primitive {
            member,
            subject,
            signature,
            selected,
        } => {
            let arguments = facts
                .types
                .intern_arguments_observed::<true>(&[Argument::Type(subject)], &|s, b| pay(s, b))?;
            (
                facts.types.intern_observed::<true>(
                    Kind::FunctionItem {
                        item: ItemId(member.0),
                        arguments,
                        signature,
                    },
                    &|s, b| pay(s, b),
                )?,
                selected,
            )
        }
        Pending::Applied {
            member,
            selected,
            recorded,
        } => (
            crate::sema::dispatch::requested_callable(facts, member, selected, recorded, pay)?,
            selected,
        ),
    }))
}
