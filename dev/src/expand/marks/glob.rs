//! A glob republishes only identities that can be mapped back to its source.
use super::{
    ExpansionId, HashMap, ItemContext, Marks, SyntaxContextId, Transparency, adjustment::Adjusted,
};
use crate::expand::ancestry::Ancestry;

pub(super) type Cache = HashMap<(ItemContext, ExpansionId, ItemContext), Option<Adjusted>>;

impl Marks {
    /// Reverse the synchronized glob/use-site adjustment. Names sharing a
    /// context share the transformed suffix; no per-name ancestry is retained.
    pub(crate) fn reverse_glob(
        &mut self,
        context: ItemContext,
        container: ExpansionId,
        glob: ItemContext,
        ancestry: &Ancestry,
    ) -> Result<Option<Adjusted>, &'static str> {
        self.reverse_observed(context, container, glob, ancestry, || {})
    }

    fn reverse_observed(
        &mut self,
        context: ItemContext,
        container: ExpansionId,
        glob: ItemContext,
        ancestry: &Ancestry,
        mut visit: impl FnMut(),
    ) -> Result<Option<Adjusted>, &'static str> {
        visit();
        if context == ItemContext::ROOT && glob == ItemContext::ROOT {
            self.adjust_item(context, container, ancestry)?;
            return Ok(Some(Adjusted {
                context,
                definition: None,
            }));
        }
        let key = (context, container, glob);
        if let Some(answer) = self
            .0
            .as_ref()
            .and_then(|data| data.reversed_globs.get(&key))
        {
            return Ok(*answer);
        }
        let answer = if self
            .adjust_item(context, container, ancestry)?
            .definition
            .is_some()
        {
            None
        } else {
            let adjusted = self.adjust_item(glob, container, ancestry)?;
            let mut remaining = glob;
            let mut suffix = Vec::new();
            while remaining != adjusted.context {
                visit();
                let (parent, expansion) = self
                    .item_parent(remaining)?
                    .ok_or("missing glob context suffix")?;
                suffix.push(expansion);
                remaining = parent;
            }
            let mut marked = context.0;
            for expansion in suffix.into_iter().rev() {
                visit();
                marked = self.apply(
                    marked,
                    expansion,
                    Transparency::Opaque,
                    SyntaxContextId::ROOT,
                )?;
            }
            Some(Adjusted {
                context: self.item(marked)?,
                definition: adjusted.definition,
            })
        };
        self.0
            .get_or_insert_with(Box::default)
            .reversed_globs
            .insert(key, answer);
        Ok(answer)
    }
}

#[cfg(test)]
#[path = "../../../tests/invariants/glob_adjustment.rs"]
mod tests;
