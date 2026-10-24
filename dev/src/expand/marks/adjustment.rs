//! Container-relative opaque identity, memoized over shared context suffixes.
use super::{ExpansionId, HashMap, ItemContext, Marks};
use crate::expand::ancestry::Ancestry;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Adjusted {
    pub context: ItemContext,
    /// The last removed mark owns definition-site privacy, if any.
    pub definition: Option<ExpansionId>,
}
pub(super) type Cache = HashMap<(ItemContext, ExpansionId), Adjusted>;

impl Marks {
    /// This arena and ancestry belong to the same compilation. Appending later
    /// expansions cannot change the ancestry of either existing identity.
    pub(crate) fn adjust_item(
        &self,
        context: ItemContext,
        container: ExpansionId,
        ancestry: &Ancestry,
    ) -> Result<Adjusted, &'static str> {
        self.adjust_observed(context, container, ancestry, || {})
    }

    fn adjust_observed(
        &self,
        mut context: ItemContext,
        container: ExpansionId,
        ancestry: &Ancestry,
        mut visit: impl FnMut(),
    ) -> Result<Adjusted, &'static str> {
        if !ancestry.ancestor(container, container) {
            return Err("unknown item container expansion");
        }
        if context == ItemContext::ROOT {
            return Ok(Adjusted {
                context,
                definition: None,
            });
        }
        let data = self.0.as_ref().ok_or("unknown item context arena")?;
        let mut cache = data
            .adjusted
            .try_borrow_mut()
            .map_err(|_| "reentrant item context adjustment")?;
        let mut pending = Vec::new();
        let mut answer = loop {
            visit();
            if let Some(answer) = cache.get(&(context, container)) {
                break *answer;
            }
            let Some((parent, expansion)) = self.item_parent(context)? else {
                break Adjusted {
                    context,
                    definition: None,
                };
            };
            if !ancestry.ancestor(expansion, expansion) {
                return Err("unknown item context expansion");
            }
            if ancestry.ancestor(expansion, container) {
                break Adjusted {
                    context,
                    definition: None,
                };
            }
            pending.push((context, expansion));
            context = parent;
        };
        // A rejected walk publishes nothing. Backfilling also shares answers
        // for distinct queries with the same suffix, not only exact repeats.
        if context != ItemContext::ROOT {
            cache.insert((context, container), answer);
        }
        for (context, expansion) in pending.into_iter().rev() {
            answer.definition = answer.definition.or(Some(expansion));
            cache.insert((context, container), answer);
        }
        Ok(answer)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/item_adjustment.rs"]
mod tests;
