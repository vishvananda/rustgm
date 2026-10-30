//! Complete original input plans remain neutral data, never imported body proof.
#[path = "port_plan/recorded_inputs.rs"]
mod coverage;
#[path = "port_plan/applied_debts.rs"]
mod debts;
#[path = "port_plan/goal_mappings.rs"]
mod goals;
#[path = "port_plan/signature_matches.rs"]
mod matching;
#[path = "port_plan/rewritten_nodes.rs"]
mod nodes;
#[path = "port_plan/operation_inputs.rs"]
mod operations;
#[path = "port_plan/evaluation_ports.rs"]
mod ports;
#[path = "port_plan/semantic_requests.rs"]
mod requests;
#[path = "port_plan/rewritten_roots.rs"]
mod roots;
#[path = "port_plan/applied_signatures.rs"]
mod signatures;
#[path = "port_plan/applied_uses.rs"]
mod uses;
#[path = "port_plan/value_requirements.rs"]
mod values;
use super::{Context, DomainIndex, Work, g, s};
use crate::support::{
    constness::Constness,
    crates::owner_catalog::{BodyRef, DeclarationRef},
};

#[derive(Clone, Copy)]
struct OperationSite {
    port: OperationPort,
    site: u32,
}

pub(super) fn build<'rows>(
    domains: &DomainIndex<'_, 'rows>,
    work: &mut Work,
) -> Result<OriginalInputPlan<'rows>, &'static str> {
    requests::Builder::new(domains, work)?.finish(work)
}

/// Every generated request retains its actual operation coordinate as well as
/// its diagnostic site. Equal source sites do not identify distinct Call ports.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OperationPort {
    Declaration {
        body: BodyRef,
        owner: DeclarationRef,
    },
    Formal {
        body: BodyRef,
        ordinal: u32,
    },
    Local {
        body: BodyRef,
        local: u32,
    },
    Parameter {
        body: BodyRef,
        formal: u32,
        local: u32,
    },
    Value {
        body: BodyRef,
        value: u32,
    },
    Statement {
        body: BodyRef,
        statement: u32,
    },
    Terminator {
        body: BodyRef,
        block: u32,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RequiredKind {
    Formation(g::TypeRef),
    Relation {
        actual: g::TypeRef,
        expected: g::TypeRef,
    },
    /// All actual type formals survive, including unused and inherited ones.
    Formal {
        slot: s::SlotRef,
        sizing: Option<s::DeclarationAddress>,
    },
    /// A declaration premise remains a goal on actual substitution. Its exact
    /// owner, qualifier and quantifier remain in the borrowed clause row.
    Declaration {
        clause: u32,
    },
    Variable {
        slot: u32,
        needs_provenance: bool,
    },
    LocalBorrow(s::RegionRef),
    /// The complete actual target declaration inputs must be derived on use;
    /// successful original proof cannot replace them with an empty list.
    Call {
        target: s::FunctionApplicationRef,
        method: Option<s::TraitApplicationRef>,
        effect: Constness,
    },
    /// A query retains the ordinary move fallback; it is not a required Copy bound.
    CopyQuery {
        ty: g::TypeRef,
        declaration: Option<s::DeclarationAddress>,
    },
    DefiningCleanup {
        member: DeclarationRef,
        trait_: s::DeclarationAddress,
    },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RequiredInput {
    port: OperationPort,
    site: s::InputSite,
    kind: RequiredKind,
}
impl RequiredInput {
    #[must_use]
    pub const fn port(self) -> OperationPort {
        self.port
    }
    #[must_use]
    pub const fn site(self) -> s::InputSite {
        self.site
    }
    #[must_use]
    pub const fn kind(self) -> RequiredKind {
        self.kind
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct InputWork {
    pub steps: usize,
    pub expression_nodes: usize,
    pub rewritten_uses: usize,
    pub application_templates: usize,
    pub comparison_pairs: usize,
    pub cache_hits: usize,
    pub required_inputs: usize,
}
/// Immutable original data only. No constructor can mint a Descriptor or body
/// proof, and import must still authenticate headers and re-prove every original.
pub struct OriginalInputPlan<'rows> {
    rows: &'rows s::SourceRows,
    ranges: Vec<g::Range>,
    requests: Vec<RequiredInput>,
    observed: InputWork,
    bytes: usize,
    peak: usize,
}
impl OriginalInputPlan<'_> {
    /// # Errors
    /// Rejects an unknown original and charges all returned request ports.
    pub fn required(
        &self,
        body: BodyRef,
        work: &mut Work,
    ) -> Result<&[RequiredInput], &'static str> {
        work.charge(1)?;
        let range = *self
            .ranges
            .get(body.0 as usize)
            .ok_or("source required-input query has a foreign original body")?;
        let result = range.slice(&self.requests)?;
        work.charge(result.len())?;
        Ok(result)
    }
    /// Complete raw inputs remain distinct from derived operation requests.
    /// In particular equalities never become authenticated universal aliases.
    /// # Errors
    /// Rejects an unknown original body and charges failed queries.
    pub fn inputs(&self, body: BodyRef, work: &mut Work) -> Result<&s::BodyInputs, &'static str> {
        work.charge(1)?;
        self.rows
            .body_inputs
            .get(body.0 as usize)
            .ok_or("source input-plan body has no actual raw inputs")
    }
    /// Complete header, application and qualifier data remain borrowed. A Call
    /// request must rederive all target clauses and unused formal debts on import.
    #[must_use]
    pub const fn rows(&self) -> &s::SourceRows {
        self.rows
    }
    #[must_use]
    pub const fn work(&self) -> InputWork {
        self.observed
    }
    #[must_use]
    pub const fn retained_bytes(&self) -> usize {
        self.bytes
    }
    #[must_use]
    pub const fn peak_bytes(&self) -> usize {
        self.peak
    }
}

#[cfg(test)]
#[path = "../../../../../tests/invariants/ingress_contracts.rs"]
mod tests;
