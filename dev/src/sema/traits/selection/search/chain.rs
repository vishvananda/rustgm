//! Follow selected dereference suffixes inside their original, still-owned proof.
use super::{Declarations, Predicate, Search, Source};
use crate::{
    support::{
        bindings::LangItem,
        constness::Constness,
        hash::{HashMap, HashSet},
    },
    types::{Kind, TypeId, substitute::Alias},
};

pub(in crate::sema) struct Chain {
    root: Predicate,
    sources: HashMap<Predicate, Source>,
}
impl Search<'_> {
    pub(super) fn selected_chain(
        &self,
        facts: &Declarations,
        root: usize,
    ) -> Result<(Option<Box<Chain>>, usize), String> {
        let node = &self.query.nodes[root];
        let Some((application, Constness::Ordinary)) = node.goal.bound.trait_parts() else {
            return Ok((None, 0));
        };
        let item = facts
            .types
            .trait_application(application)
            .ok_or("selected chain lost its actual trait application")?
            .item;
        if facts
            .source_bindings
            .as_deref()
            .and_then(|c| c.language(LangItem::Deref))
            .is_none_or(|declaration| declaration.definition.0 != item.0)
        {
            return Ok((None, 0));
        }
        if self.query.scopes.is_some()
            || self.query.universal.is_some()
            || self.caller.placeholders.is_some()
            || self.caller.retain_sized
            || !matches!(node.source, Some(Source::Implementation { .. }))
        {
            return Ok((None, 0));
        }
        let mut pending = vec![root];
        let mut visited = HashSet::default();
        let mut sources = HashMap::default();
        let mut work = 0_usize;
        while let Some(index) = pending.pop() {
            work += 1;
            if work > 1_000_000 {
                return Err("selected projection chain work budget exceeded".into());
            }
            if !visited.insert(index) {
                continue;
            }
            let node = &self.query.nodes[index];
            if node.provisional_size || node.cycle_dependent || !node.sized.is_empty() {
                return Ok((None, work));
            }
            pending.extend(node.children.iter().copied());
            if node.goal.bound == self.query.nodes[root].goal.bound
                && let Some(source @ Source::Implementation { .. }) = &node.source
            {
                super::super::refutation::require_positive_source(facts, Some(source))?;
                if sources.insert(node.goal, source.clone()).is_some() {
                    return Ok((None, work));
                }
            }
        }
        if sources.len() < 2 {
            return Ok((None, work));
        }
        Ok((
            Some(Box::new(Chain {
                root: self.query.nodes[root].goal,
                sources,
            })),
            work,
        ))
    }
}
impl Chain {
    /// No selection or lifetime namespace is reopened. The original whole
    /// parent proof already owns every selected child and all of its debts.
    pub(in crate::sema) fn resolve(
        &self,
        facts: &mut Declarations,
        mut alias: Alias,
    ) -> Result<(Option<TypeId>, usize), &'static str> {
        let Some(mut root) = alias.associated() else {
            return Ok((None, 0));
        };
        let Some((application, _)) = self.root.bound.trait_parts() else {
            return Ok((None, 0));
        };
        root.subject = self.root.ty;
        root.trait_application = application;
        alias = Alias::Associated(root);
        let mut seen = HashSet::default();
        let mut result = None;
        let mut work = 0_usize;
        loop {
            work += 1;
            if work > 1_000_000 {
                return Err("selected projection chain work budget exceeded");
            }
            let requirement = crate::sema::opaque::overrides::requirement(facts, alias)?;
            if !seen.insert(requirement) {
                return Ok((None, work));
            }
            let Some(source) = self.sources.get(&requirement) else {
                return Ok((result, work));
            };
            let Some(definition) = crate::sema::projections::Definition::alias(
                facts,
                alias,
                alias.arguments(),
                source.clone(),
            )?
            else {
                return Ok((None, work));
            };
            let Some(ty) = definition.result(facts)? else {
                return Ok((None, work));
            };
            result = Some(ty);
            match facts.types.kind(ty) {
                Some(Kind::Projection(application)) if application.item == root.item => {
                    alias = Alias::Associated(application);
                }
                _ => return Ok((result, work)),
            }
        }
    }
}
