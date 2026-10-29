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
pub(super) struct Pending {
    member: DefId,
    subject: TypeId,
    signature: TypeId,
    selected: TypeId,
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
            pending.push(Pending {
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
    work: &mut ImportedOriginals,
) -> Result<(), String> {
    if pending.is_empty() {
        return Ok(());
    }
    let steps = Cell::new(0_usize);
    let bytes = Cell::new(0_usize);
    let base = work.retained_bytes();
    let pay = |count, allocation| {
        steps.set(steps.get().saturating_add(count));
        bytes.set(bytes.get().saturating_add(allocation));
        if work.steps.saturating_add(steps.get()) > crates::MAX_COMPONENTS
            || base.saturating_add(bytes.get()) > crates::MAX_BYTES
        {
            return Err("selected source trait proof exceeds its actual work/storage envelope");
        }
        Ok(())
    };
    let result = (|| {
        let mut dispatcher = Dispatcher::in_body(&facts.types, 0, 1, environment, &pay)?;
        for goal in pending {
            let arguments = facts
                .types
                .intern_arguments_observed::<true>(&[Argument::Type(goal.subject)], &pay)?;
            let requested = facts.types.intern_observed::<true>(
                Kind::FunctionItem {
                    item: ItemId(goal.member.0),
                    arguments,
                    signature: goal.signature,
                },
                &pay,
            )?;
            if dispatcher.select(facts, requested)? != goal.selected {
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
    })();
    work.steps = work.steps.saturating_add(steps.get());
    result
}
