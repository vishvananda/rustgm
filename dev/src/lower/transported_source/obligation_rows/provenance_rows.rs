//! Export the same raw CFG provenance requests consumed by the original proof.
use super::{Body, Context, Declarations, Stage, locations::Sites, s};
use crate::{body::regions::Obligation, borrow::regions::PlaceBoundsSink};

struct Rows<'a> {
    stage: &'a mut Stage,
    facts: &'a Declarations,
    context: Context,
    sites: &'a mut Sites,
    output: &'a mut Vec<s::Obligation>,
    scratch: usize,
}
impl PlaceBoundsSink for Rows<'_> {
    fn charge(&mut self, count: usize) -> Result<(), &'static str> {
        self.stage.charge(count)
    }
    fn scratch(&mut self, growth: usize) -> Result<(), &'static str> {
        // Capacity replacements may coexist with all earlier live provenance
        // buffers. Retain conservative attempted growth, including failures.
        self.scratch = self.scratch.saturating_add(growth);
        self.stage.scratch(self.scratch)
    }
    fn obligation(&mut self, value: Obligation) -> Result<(), &'static str> {
        self.stage.charge(1)?;
        self.stage.reserve(4 * size_of::<s::Obligation>())?;
        let site = self.sites.input(self.stage, value.source)?;
        let kind = super::obligation_kind(self.stage, self.facts, value.kind, self.context)?;
        self.output.push(s::Obligation { site, kind });
        Ok(())
    }
}
pub(super) fn append(
    stage: &mut Stage,
    facts: &Declarations,
    body: &Body,
    context: Context,
    sites: &mut Sites,
    output: &mut Vec<s::Obligation>,
) -> Result<(), &'static str> {
    // The caller has authenticated the checked original and its immutable raw
    // Contract. This runs once before that original's complete rows are sealed;
    // later recipe demands only borrow those rows and never revisit this CFG.
    crate::borrow::regions::visit_place_bounds(
        &facts.types,
        body,
        &mut Rows {
            stage,
            facts,
            context,
            sites,
            output,
            scratch: 0,
        },
    )
}
