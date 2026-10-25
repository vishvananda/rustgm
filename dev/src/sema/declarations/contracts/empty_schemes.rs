//! Complete source zero domains at the real declaration transaction, never at lookup.
use super::{BinderId, Checker, DefId, Error, FunctionRegions, NodeKey, SignatureTypes, Type};
use crate::support::{crates::MAX_BYTES, retained};

#[cfg(test)]
#[path = "../../../../tests/invariants/empty_schemes.rs"]
mod tests;

pub(in crate::sema::declarations) struct CompletedEmpty {
    owner: DefId,
    source: NodeKey,
    scheme: FunctionRegions,
}
pub(super) fn prepare(
    checker: &Checker<'_>,
    source: NodeKey,
    owner: DefId,
    binder: BinderId,
    signatures: SignatureTypes,
    captures: u32,
    bounds: Vec<(crate::types::RegionId, crate::types::RegionId)>,
) -> Result<Option<CompletedEmpty>, Error> {
    if !checker.report.package.uses_source_bindings() {
        return Ok(None);
    }
    let owned = checker.report.resolution.definitions.get(owner.0 as usize);
    let total = checker.regions.count(binder);
    if owned.is_none_or(|row| {
        row.kind != crate::resolve::model::DefKind::Function || row.node != Some(source)
    }) || checker.regions.owner(binder) != Some(owner)
        || total != 0
        || captures != 0
        || !bounds.is_empty()
        || checker.facts.generics.has_parameters(owner)
        || [signatures.normal, signatures.original]
            .into_iter()
            .any(|signature| {
                !matches!(
                    checker.facts.types.kind(signature),
                    Some(Type::Function {
                        bound_regions: 0,
                        ..
                    })
                )
            })
    {
        return Err(checker.error(
            source,
            "source empty scheme lost its actual completed owner or domain",
        ));
    }
    Ok(Some(CompletedEmpty {
        owner,
        source,
        scheme: FunctionRegions {
            total,
            captures,
            early: Vec::new(),
            late: Vec::new(),
            has_declared_late: false,
            bounds,
        },
    }))
}
impl Checker<'_> {
    pub(in crate::sema::declarations) fn retain_empty_scheme(
        &mut self,
        completed: CompletedEmpty,
        steps: &mut usize,
        limit: usize,
    ) -> Result<(), Error> {
        *steps = steps.saturating_add(1);
        let error = |message| self.error(completed.source, message);
        if *steps > limit {
            return Err(error("source empty scheme completion work budget exceeded"));
        }
        if !self.report.package.uses_source_bindings()
            || self
                .report
                .resolution
                .definitions
                .get(completed.owner.0 as usize)
                .is_none_or(|row| row.node != Some(completed.source))
            || self
                .facts
                .lifetimes
                .functions
                .contains_key(&completed.owner)
        {
            return Err(error(
                "source empty scheme completion changed owner or repeated publication",
            ));
        }
        let schemes = &self.facts.lifetimes.functions;
        let needed = schemes.len().saturating_add(1);
        let mut peak = retained::map(schemes);
        if needed > schemes.capacity() {
            *steps = steps.saturating_add(schemes.len());
            peak = peak.saturating_add(retained::new_map::<DefId, FunctionRegions>(needed));
        }
        if *steps > limit || peak > MAX_BYTES {
            return Err(error(
                "source empty scheme completion exceeds work or storage budget",
            ));
        }
        self.facts
            .lifetimes
            .functions
            .insert(completed.owner, completed.scheme);
        Ok(())
    }
}
