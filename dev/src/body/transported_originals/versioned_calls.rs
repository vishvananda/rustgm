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
pub(super) struct Member {
    owner: DeclarationRef,
    declaration: DeclarationRef,
    subject: g::TypeRef,
}
pub(super) struct Index<'a> {
    rows: s::ValidatedView<'a>,
    members: HashMap<DeclarationRef, Member>,
    bytes: usize,
}
#[derive(Clone, Copy)]
pub(crate) struct Primitive {
    member: DefId,
    subject: TypeId,
    signature: TypeId,
    selected: TypeId,
}
#[derive(Clone, Copy)]
pub(crate) struct Applied {
    member: DefId,
    selected: TypeId,
    recorded: Option<crate::types::parameters::predicates::Predicate>,
}
pub(crate) enum Pending {
    Primitive(Vec<Primitive>),
    Applied {
        rows: Vec<Option<Applied>>,
        normalize_projections: bool,
    },
}
#[derive(Clone, Copy)]
enum Operand {
    Primitive(Primitive),
    Applied(Applied),
}
impl Pending {
    fn is_empty(&self) -> bool {
        match self {
            Self::Primitive(rows) => rows.is_empty(),
            Self::Applied { rows, .. } => rows.is_empty(),
        }
    }
    fn len(&self) -> usize {
        match self {
            Self::Primitive(rows) => rows.len(),
            Self::Applied { rows, .. } => rows.len(),
        }
    }
    fn operands(&self) -> impl Iterator<Item = Operand> + '_ {
        let (primitive, applied) = match self {
            Self::Primitive(rows) => (rows.as_slice(), &[][..]),
            Self::Applied { rows, .. } => (&[][..], rows.as_slice()),
        };
        primitive
            .iter()
            .copied()
            .map(Operand::Primitive)
            .chain(applied.iter().copied().flatten().map(Operand::Applied))
    }
}
// Provided and inherited members keep the body in its defining owner.
fn outside_implementation(
    rows: s::ValidatedView<'_>,
    link: &s::MemberLink,
    specializes: bool,
) -> Result<bool, &'static str> {
    let Some(expected) = link.trait_member.filter(|expected| {
        rows.descriptor().profile().declaration_policies() && expected.artifact == s::ArtifactRef(0)
    }) else {
        return Ok(false);
    };
    let declaration = rows.header(expected.declaration)?;
    Ok(matches!(declaration, Some(s::HeaderRef::Constant(_)))
        || link.implementation.is_none()
            && (rows.transported_default(declaration)
                || specializes
                    && matches!(
                        declaration,
                        Some(s::HeaderRef::Function(_) | s::HeaderRef::AssociatedType(_))
                    )))
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
        let ancestry = rows.cold().specialization.as_deref();
        let per_header = ancestry.map_or(1, |graph| graph.lookup_work() + 1);
        work.charge(
            rows.implementations
                .len()
                .saturating_mul(per_header)
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
            // A version-thirteen inherent impl links no trait member; a call
            // to its method is an ordinary application of the method's header.
            if header.trait_application.is_none() {
                continue;
            }
            let specializes =
                ancestry.is_some_and(|graph| graph.parent(header.self_definition).is_some());
            for link in header.members.slice(&rows.member_links)? {
                if outside_implementation(rows, link, specializes)? {
                    continue;
                }
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
    ) -> Result<Pending, &'static str> {
        if !std::ptr::eq(&raw const *self.rows, &raw const *view.rows()) {
            return Err("selected source member index belongs to another actual descriptor");
        }
        if self.rows.descriptor().profile().conversion_roles() {
            return applied_members::prepare(self, view, facts, body, work).map(|rows| {
                Pending::Applied {
                    rows,
                    normalize_projections: self.rows.descriptor().profile().declaration_policies(),
                }
            });
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
                .declaration_header(definition)?
                .ok_or("selected method lacks its installed trait header")?
                .signature;
            if !matches!(value.kind, Rvalue::Function(_))
                || !matches!(facts.types.kind(value.ty),Some(Kind::FunctionItem{signature:actual,..}) if actual==signature)
            {
                return Err("selected method requires its exact closed primitive trait signature");
            }
            pending.push(Primitive {
                member: definition,
                subject: view.ty(member.subject, &mut work.steps)?,
                signature,
                selected: value.ty,
            });
        }
        Ok(Pending::Primitive(pending))
    }
}
pub(super) fn lookup_member(
    rows: s::ValidatedView<'_>,
    target: DeclarationRef,
) -> Result<Option<Member>, &'static str> {
    let Some(s::HeaderRef::Function(h)) = rows.header(target)? else {
        return Ok(None);
    };
    let Some(owner) = rows.functions[h as usize].parent else {
        return Ok(None);
    };
    let Some(s::HeaderRef::Implementation(impl_idx)) = rows.header(owner)? else {
        return Ok(None);
    };
    let header = &rows.implementations[impl_idx as usize];
    if header.trait_application.is_none() {
        return Ok(None);
    }
    let specializes = rows
        .cold()
        .specialization
        .as_deref()
        .is_some_and(|graph| graph.parent(header.self_definition).is_some());
    for link in header.members.slice(&rows.member_links)? {
        if outside_implementation(rows, link, specializes)? {
            continue;
        }
        if link.implementation == Some(target) {
            let Some(expected) = link.trait_member else {
                return Err("v4 selected methods require actual override/member links");
            };
            if expected.artifact != s::ArtifactRef(0) {
                return Err("selected source member has a foreign or repeated actual owner");
            }
            return Ok(Some(Member {
                owner: header.self_definition,
                declaration: expected.declaration,
                subject: header.self_type,
            }));
        }
    }
    Ok(None)
}

pub(crate) fn prepare_single(
    facts: &Declarations,
    view: &SourceOriginal<'_>,
    body: &Body,
    steps: &mut usize,
) -> Result<Pending, &'static str> {
    let rows = view.rows();
    if !rows.descriptor().profile().feature_predicates() {
        return Ok(Pending::Primitive(Vec::new()));
    }
    let mut work = ImportedOriginals::default();
    let result = if rows.descriptor().profile().conversion_roles() {
        applied_members::prepare_single(view, facts, body, &mut work).map(|applied| {
            Pending::Applied {
                rows: applied,
                normalize_projections: rows.descriptor().profile().declaration_policies(),
            }
        })
    } else {
        let mut pending = Vec::new();
        for (record, value) in view.raw().values.iter().zip(&body.values) {
            work.charge(1)?;
            let s::ValueKind::Function(reference) = record.kind else {
                continue;
            };
            let target = rows.function_applications[reference.0 as usize]
                .declaration
                .declaration;
            let Some(member) = lookup_member(rows, target)? else {
                continue;
            };
            let definition = view.declaration(member.declaration, &mut work.steps)?;
            let signature = facts
                .declaration_header(definition)?
                .ok_or("selected method lacks its installed trait header")?
                .signature;
            pending.push(Primitive {
                member: definition,
                subject: view.ty(member.subject, &mut work.steps)?,
                signature,
                selected: value.ty,
            });
        }
        Ok(Pending::Primitive(pending))
    };
    *steps = steps.saturating_add(work.steps);
    result
}

pub(crate) fn prove_single(
    facts: &mut Declarations,
    environment: Arc<Environment>,
    pending: &Pending,
    body: &Body,
    contract: &mut crate::body::regions::Contract,
) -> Result<(), String> {
    if pending.is_empty() || !crate::support::sealed::self_check() {
        return Ok(());
    }
    let mut work = ImportedOriginals::default();
    let allowance = super::decoded_bodies::Allowance {
        header_work: 0,
        header_bytes: 0,
        templates: crates::MAX_BYTES,
    };
    let result = prove(
        facts,
        environment,
        pending,
        body,
        contract,
        &mut work,
        &allowance,
    );
    facts.body_work.canonical_steps = facts.body_work.canonical_steps.saturating_add(work.steps);
    result
}

pub(super) fn allocation(values: usize, conversions: bool) -> usize {
    values.saturating_mul(
        4 * if conversions {
            size_of::<Option<Applied>>()
        } else {
            size_of::<Primitive>()
        },
    )
}
pub(super) fn prove(
    facts: &mut Declarations,
    environment: Arc<Environment>,
    pending: &Pending,
    body: &Body,
    contract: &mut crate::body::regions::Contract,
    work: &mut ImportedOriginals,
    allowance: &super::decoded_bodies::Allowance,
) -> Result<(), String> {
    if pending.is_empty() || !crate::support::sealed::self_check() {
        return Ok(());
    }
    paying(work, allowance, |pay| {
        pay(pending.len(), 0)?;
        let applied = matches!(pending, Pending::Applied { .. });
        let normalize_projections = matches!(
            pending,
            Pending::Applied {
                normalize_projections: true,
                ..
            }
        );
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
        for goal in pending.operands() {
            let (requested, selected) = request(facts, goal, &pay)?;
            let actual = dispatcher.select(facts, requested)?;
            // The packet's function-item recipe reconstructs its written
            // declaration signature. Reuse this dispatcher's exact completed
            // normalization of that override before comparing full identities.
            let selected = if normalize_projections {
                dispatcher.select(facts, selected)?
            } else {
                selected
            };
            if actual != selected {
                return Err(
                    "fresh source trait dispatch selected another implementation function".into(),
                );
            }
        }
        let proof = dispatcher.finish();
        if applied {
            let alias_only = proof.regions == (first..first.saturating_add(universals))
                && proof.equalities.len() == universals as usize
                && proof.outlives.is_empty()
                && proof.types.is_empty()
                && !proof.has_quantified();
            if normalize_projections
                && !alias_only
                && proof.self_contained_in(&facts.types, universals)?
            {
                // A closed override's binder normalization is a fresh query
                // proof. Discharge it universally here; it adds no caller-local
                // debts to the immutable transported body contract.
                crate::sema::discharge::quantified(
                    facts,
                    &proof,
                    universals,
                    "fresh source override normalization is not universally valid",
                )?;
            } else if !alias_only {
                let extra = proof
                    .regions
                    .len()
                    .saturating_add(proof.equalities.len())
                    .saturating_add(proof.outlives.len())
                    .saturating_add(proof.types.len());
                pay(
                    extra,
                    extra.saturating_mul(4 * size_of::<crate::body::regions::Obligation>()),
                )?;
                crate::body::dispatch::retain_proof(body, contract, proof);
            }
        } else if !proof.is_empty() {
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
    goal: Operand,
    pay: retained::Payment<'_>,
) -> Result<(TypeId, TypeId), &'static str> {
    Ok(match goal {
        Operand::Primitive(Primitive {
            member,
            subject,
            signature,
            selected,
        }) => {
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
        Operand::Applied(Applied {
            member,
            selected,
            recorded,
        }) => (
            crate::sema::dispatch::requested_callable(facts, member, selected, recorded, pay)?,
            selected,
        ),
    })
}
