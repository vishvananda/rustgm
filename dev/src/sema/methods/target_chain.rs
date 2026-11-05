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
    if level == Level::Legacy {
        return Ok(());
    }
    let error = |message| Declarations::error(report, key, message);
    let mut current = match context.resolved(&mut facts.types, term) {
        Ok(ty) => ty,
        Err(infer::Error::Unresolved) => return Ok(()),
        Err(e) => return Err(error(selection::inference(e))),
    };
    let mut visited = HashSet::default();
    // Discovery may select actual implementation headers. The unscoped proof
    // constructor only checks caller premises; it cannot traverse such headers.
    // No resulting region evidence authorizes a call or executable adjustment.
    let mut proof = Proof::for_declaration(caller.environment.cloned(), 0, 1);
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
    selection::account_probe(facts, proof.trait_selection_steps()).map_err(error)?;
    found.members.sort_unstable_by_key(|member| member.0);
    found.members.dedup();
    Ok(())
}
