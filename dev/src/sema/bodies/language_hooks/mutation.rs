//! Mutable place consumers refine ordinary selected index calls in their own body.
use super::{Checker, Error, Hook, NodeId, TermId};
use crate::syntax::ast::{Kind, Unary};
use std::collections::{HashMap, HashSet};
#[derive(Clone, Copy)]
struct Selected {
    method: usize,
    call: usize,
    kind: Hook,
}
#[derive(Default)]
pub(in crate::sema::bodies) struct Places {
    mutable: HashSet<NodeId>,
    selected: HashMap<NodeId, Selected>,
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn mutable_language_place(&self, node: NodeId) -> bool {
        self.state
            .language_places
            .as_ref()
            .is_some_and(|places| places.mutable.contains(&node))
    }
    pub(in crate::sema::bodies) fn mark_mutable_language_place(
        &mut self,
        mut node: NodeId,
    ) -> Result<(), Error> {
        if self
            .report
            .resolution
            .builtin_traits
            .language(Hook::Index)
            .is_none()
        {
            return Ok(());
        }
        for _ in 0..1_000_000 {
            match self.kind(node) {
                Kind::Group(base)
                | Kind::Attributed { value: base, .. }
                | Kind::Field { value: base, .. } => node = base,
                Kind::Index { value, index } => {
                    let places = self
                        .state
                        .language_places
                        .get_or_insert_with(Default::default);
                    places.mutable.insert(node);
                    let previous = places.selected.get(&node).copied();
                    if previous.is_some_and(|selected| selected.kind == Hook::Index) {
                        let old = self.term(node)?;
                        let new =
                            self.language_operator(node, Hook::IndexMut, value, Some(index))?;
                        self.equate(node, old, new)?;
                    }
                    node = value;
                }
                Kind::Unary {
                    op: Unary::Dereference,
                    operand,
                } => {
                    self.state
                        .language_places
                        .get_or_insert_with(Default::default)
                        .mutable
                        .insert(node);
                    let reference = if let Some(term) = self.state.terms.get(self.key(operand)) {
                        matches!(
                            self.head(node, term)?,
                            Some(super::Constructor::Reference { mutable: true, .. })
                        )
                    } else {
                        false
                    };
                    if !reference {
                        return Ok(());
                    }
                    node = operand;
                }
                _ => return Ok(()),
            }
        }
        Err(self.error(node, "mutable language place work budget exceeded"))
    }
    pub(in crate::sema::bodies) fn language_index_method(&self, node: NodeId) -> Option<usize> {
        Some(
            self.state
                .language_places
                .as_ref()?
                .selected
                .get(&node)?
                .method,
        )
    }
    pub(super) fn retain_language_index_call(
        &mut self,
        node: NodeId,
        kind: Hook,
        method: usize,
        signature: TermId,
    ) {
        let Some(places) = self.state.language_places.as_mut() else {
            self.state.calls.push((node, signature));
            return;
        };
        let previous = places.selected.get(&node).copied();
        let call = previous.map_or(self.state.calls.len(), |selected| selected.call);
        if previous.is_some() {
            self.state.calls[call] = (node, signature);
        } else {
            self.state.calls.push((node, signature));
        }
        places
            .selected
            .insert(node, Selected { method, call, kind });
    }
}
