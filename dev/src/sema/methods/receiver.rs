//! Receiver search probes each dereference level before committing any equations.
use super::selection;
use crate::{
    resolve::{
        expansion::Report,
        model::{Access, DefId, MethodCall},
        package::Error,
        unit::NodeKey,
    },
    sema::model::Declarations,
    types::{
        RegionId,
        infer::{Constructor, Context, TermId},
    },
};

#[derive(Clone, Copy)]
pub(in crate::sema) struct Selected {
    pub method: DefId,
    pub dereferences: u32,
    pub borrow: Option<bool>,
    pub self_term: TermId,
}

pub(in crate::sema) fn choose(
    report: &Report,
    facts: &mut Declarations,
    source: &mut Context,
    key: NodeKey,
    mut term: TermId,
    site: MethodCall,
    caller: selection::Caller<'_>,
) -> Result<Selected, Error> {
    let error = |message: &str| Declarations::error(report, key, message);
    let (base, reference_depth) = source
        .reference_target(&facts.types, term)
        .map_err(|e| error(&e.to_string()))?;
    let Some(Constructor::Adt(nominal, _) | Constructor::AdtArgs(nominal, _)) = source
        .head(&facts.types, base)
        .map_err(|e| error(&e.to_string()))?
    else {
        return Err(error("receiver has no inherent nominal method"));
    };
    let bucket = (nominal, site.name);
    let candidates = facts
        .methods
        .inherent
        .get(&bucket)
        .ok_or_else(|| error("no inherent method with this name"))?;
    let candidates = if candidates.len() == 1 {
        candidates.clone()
    } else {
        selection::candidates(facts, source, base, bucket).map_err(error)?
    };
    let module = report.resolution.scopes[site.scope.0 as usize].module;
    for dereferences in 0..=reference_depth {
        for borrow in [None, Some(false), Some(true)] {
            let mut selected = None;
            for &method in &candidates {
                let Some(&(receiver, references)) = facts.methods.receivers.get(&method) else {
                    continue;
                };
                if references != reference_depth - dereferences + u32::from(borrow.is_some()) {
                    continue;
                }
                if let Access::Within(ancestor) =
                    report.resolution.definitions[method.0 as usize].access
                    && !report.package.is_descendant(module, ancestor)
                {
                    continue;
                }
                let owner = facts
                    .associated
                    .owner(method)
                    .and_then(|id| facts.associated.get(id))
                    .ok_or_else(|| error("missing receiver candidate owner"))?
                    .self_definition;
                let mut probe = Context::default();
                let mut actual = probe
                    .import(&facts.types, source, term)
                    .map_err(|e| error(&e.to_string()))?;
                if let Some(mutable) = borrow {
                    actual = probe
                        .construct(
                            Constructor::Reference {
                                mutable,
                                region: RegionId::STATIC,
                            },
                            &[actual],
                        )
                        .map_err(|e| error(&e.to_string()))?;
                }
                let applicable =
                    selection::applicable(facts, &mut probe, actual, owner, Some(receiver), caller)
                        .map_err(error)?;
                facts.methods.selection_probes += 1;
                facts.methods.selection_steps =
                    facts.methods.selection_steps.saturating_add(probe.steps);
                if facts.methods.selection_steps > 1_000_000 {
                    return Err(error("method selection work budget exceeded"));
                }
                if applicable && selected.replace(method).is_some() {
                    return Err(error("multiple applicable methods for this receiver"));
                }
            }
            if let Some(method) = selected {
                return Ok(Selected {
                    method,
                    dereferences,
                    borrow,
                    self_term: base,
                });
            }
        }
        if !matches!(
            source
                .head(&facts.types, term)
                .map_err(|e| error(&e.to_string()))?,
            Some(Constructor::Reference { .. })
        ) {
            return Err(error("no applicable inherent method for this receiver"));
        }
        term = source
            .child(&facts.types, term, 0)
            .map_err(|e| error(&e.to_string()))?;
    }
    Err(error("receiver dereference work budget exceeded"))
}
