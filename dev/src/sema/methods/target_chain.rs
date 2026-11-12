//! Discover target-chain owners; applicability still consumes the actual receiver.
use super::{
    names::Use,
    selection::{self, Caller, Found},
};
use crate::{
    resolve::{expansion::Report, names::NameId, package::Error, unit::NodeKey},
    sema::{
        compiler_items::arbitrary_self::{self, Level},
        model::Declarations,
        nominals::Proof,
    },
    support::{hash::HashSet, retained},
    types::{
        Kind, TypeId,
        infer::{self, Context, TermId},
    },
};

pub(super) fn collect(
    report: &Report,
    facts: &mut Declarations,
    context: &mut Context,
    (key, term): (NodeKey, TermId),
    (name, site): (NameId, Use),
    caller: Caller<'_>,
    found: &mut Found,
) -> Result<(), Error> {
    let level = arbitrary_self::level(report, key)?;
    // Without the receiver features, rustc still assembles inherent members
    // from each Deref step and matches their declared receiver against the
    // outer one. Only a smart receiver can match there, and its owner follows
    // genuine LegacyReceiver/Deref evidence.
    if level == Level::Legacy
        && (facts.source_bindings.is_none()
            || !facts
                .methods
                .smart_names
                .as_ref()
                .is_some_and(|names| names.contains(&name)))
    {
        return Ok(());
    }
    let error = |message| Declarations::error(report, key, message);
    let current = match context.resolved(&mut facts.types, term) {
        Ok(ty) => ty,
        Err(infer::Error::Unresolved) => return Ok(()),
        Err(e) => return Err(error(selection::inference(e))),
    };
    // A body's receiver keeps the caller's lifetimes bound, which an
    // implementation header would read as its own. Discovery opens them into
    // fresh regions beyond the body frontier; none leaves this probe.
    let mut opening = crate::types::substitute::Bound::open_regions(
        facts.body_regions.probe_frontier().map_err(error)?,
    )
    .map_err(error)?;
    let mut current = opening.apply(&mut facts.types, current).map_err(error)?;
    let first = opening
        .next_region()
        .ok_or_else(|| error("receiver discovery lost its opened regions"))?;
    let mut visited = HashSet::default();
    // Discovery may select actual implementation headers. The unscoped proof
    // constructor only checks caller premises; it cannot traverse such headers.
    // No resulting region evidence authorizes a call or executable adjustment.
    let mut proof = Proof::for_declaration(caller.environment.cloned(), 0, first);
    loop {
        if visited.contains(&current) {
            break;
        }
        if visited.len() >= 1_000_000 {
            return Err(error("receiver method Target chain work budget exceeded"));
        }
        if visited.len() == visited.capacity() {
            let count = visited.len().saturating_mul(2).max(4);
            crate::sema::header_attributes::reserve_source_contracts(
                facts,
                1,
                retained::new_map::<TypeId, ()>(count),
            )?;
            visited
                .try_reserve(count - visited.len())
                .map_err(|_| error("receiver owner allocation failed"))?;
        }
        visited.insert(current);
        if let Some(Kind::Adt(item, _)) = facts.types.kind(current)
            && facts.methods.inherent.contains_key(&(item, name))
        {
            let target = context
                .known(&facts.types, current)
                .map_err(|e| error(selection::inference(e)))?;
            let candidates =
                selection::candidates(report, facts, context, target, (item, name), site)
                    .map_err(error)?;
            found.filtered |= candidates.filtered;
            found.members.extend(candidates.members);
        }
        let Some(next) =
            arbitrary_self::next(report, facts, &mut proof, current, level).map_err(error)?
        else {
            break;
        };
        current = next;
    }
    selection::account_probe(
        facts,
        proof.trait_selection_steps().saturating_add(opening.steps),
    )
    .map_err(error)?;
    found.members.sort_unstable_by_key(|member| member.0);
    found.members.dedup();
    Ok(())
}
