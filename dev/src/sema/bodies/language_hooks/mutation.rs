//! Mutable place consumers refine ordinary selected index calls in their own body.
use super::{Checker, Error, Hook, NodeId, TermId};
use crate::support::{
    hash::{HashMap, HashSet},
    retained,
};
use crate::syntax::ast::{Kind, Unary};
#[derive(Clone, Copy)]
enum Selected {
    Call {
        method: u32,
        call: u32,
        kind: Selection,
    },
    Field {
        plan: super::super::access_paths::FieldPlanId,
    },
}
#[derive(Clone, Copy, PartialEq, Eq)]
enum Selection {
    Index(Hook),
    Dereference { mutable: bool },
}
const _: [(); 12] = [(); size_of::<Selected>()];
#[derive(Default)]
pub(in crate::sema::bodies) struct Places {
    mutable: HashSet<NodeId>,
    selected: HashMap<NodeId, Selected>,
}
impl Checker<'_> {
    pub(in crate::sema::bodies) fn prepare_language_receiver(
        &mut self,
        mut node: NodeId,
    ) -> Result<(), Error> {
        if self
            .report
            .resolution
            .builtin_traits
            .language(Hook::Index)
            .is_none()
            && !crate::sema::compiler_items::dereferencing::available(self.facts)
            && !crate::sema::compiler_items::index_available(self.facts)
        {
            return Ok(());
        }
        for _ in 0..1_000_000 {
            match self.kind(node) {
                Kind::Index { .. }
                | Kind::Unary {
                    op: Unary::Dereference,
                    ..
                } => {
                    self.state
                        .language_places
                        .get_or_insert_with(Default::default);
                    return Ok(());
                }
                Kind::Group(base)
                | Kind::Attributed { value: base, .. }
                | Kind::Field { value: base, .. } => node = base,
                _ => return Ok(()),
            }
        }
        Err(self.error(node, "language receiver place work budget exceeded"))
    }
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
            && !crate::sema::compiler_items::dereferencing::available(self.facts)
            && !crate::sema::compiler_items::index_available(self.facts)
        {
            return Ok(());
        }
        for _ in 0..1_000_000 {
            match self.kind(node) {
                Kind::Group(base) | Kind::Attributed { value: base, .. } => node = base,
                Kind::Field { value, .. } => {
                    if crate::sema::compiler_items::dereferencing::available(self.facts)
                        && (self.field_plan(node).is_some()
                            || self.state.terms.get(self.key(node)).is_none())
                    {
                        self.retain_mutable_field(node)?;
                    }
                    node = value;
                }
                Kind::Index { value, index } => {
                    let places = self
                        .state
                        .language_places
                        .get_or_insert_with(Default::default);
                    places.mutable.insert(node);
                    let previous = places.selected.get(&node).copied();
                    if previous.is_some_and(|selected| {
                        matches!(
                            selected,
                            Selected::Call {
                                kind: Selection::Index(Hook::Index),
                                ..
                            }
                        )
                    }) {
                        let old = self.term(node)?;
                        let new = if self.facts.source_bindings.is_some() {
                            let subject = self.autoderef(node, self.term(value)?)?;
                            self.source_index_operator(node, value, index, subject, true)?
                        } else {
                            self.language_operator(node, Hook::IndexMut, value, Some(index))?
                        };
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
                    let previous = self
                        .state
                        .language_places
                        .as_ref()
                        .and_then(|p| p.selected.get(&node))
                        .copied();
                    if previous.is_some_and(|selected| {
                        matches!(
                            selected,
                            Selected::Call {
                                kind: Selection::Dereference { mutable: false },
                                ..
                            }
                        )
                    }) {
                        let old = self.term(node)?;
                        let new = self.source_dereference(node, operand, true)?;
                        self.equate(node, old, new)?;
                    }
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
        match self.state.language_places.as_ref()?.selected.get(&node)? {
            Selected::Call { method, .. } => Some(*method as usize),
            Selected::Field { .. } => None,
        }
    }
    pub(super) fn retain_language_index_call(
        &mut self,
        node: NodeId,
        kind: Hook,
        method: usize,
        signature: TermId,
    ) -> Result<(), Error> {
        // Every selected index call keeps the actual row, even without earlier
        // mutable-place preparation; the shared typed helper initializes Places.
        self.retain_place_call(node, Selection::Index(kind), method, signature)
    }

    pub(in crate::sema::bodies) fn retain_dereference_call(
        &mut self,
        node: NodeId,
        mutable: bool,
        method: usize,
        signature: TermId,
    ) -> Result<(), Error> {
        self.retain_place_call(node, Selection::Dereference { mutable }, method, signature)
    }
    fn retain_place_call(
        &mut self,
        node: NodeId,
        kind: Selection,
        method: usize,
        signature: TermId,
    ) -> Result<(), Error> {
        let method = u32::try_from(method)
            .map_err(|_| self.error(node, "language method inventory overflow"))?;
        let previous = self
            .state
            .language_places
            .as_ref()
            .and_then(|places| places.selected.get(&node))
            .copied();
        let call = match previous {
            Some(Selected::Call { call, .. }) => call,
            Some(Selected::Field { .. }) => {
                return Err(self.error(node, "language call overlaps an unfinished field plan"));
            }
            None => u32::try_from(self.state.calls.len())
                .map_err(|_| self.error(node, "language call inventory overflow"))?,
        };
        self.reserve_place_selection(node, usize::from(previous.is_none()))?;
        if previous.is_some() {
            let source = self.key(node);
            let report = self.report;
            let row = self
                .state
                .calls
                .get_mut(call as usize)
                .filter(|(source, _)| *source == node)
                .ok_or_else(|| {
                    super::super::Declarations::error(
                        report,
                        source,
                        "language call changed its current source row",
                    )
                })?;
            *row = (node, signature);
        } else {
            self.state.calls.push((node, signature));
        }
        self.state
            .language_places
            .get_or_insert_with(Default::default)
            .selected
            .insert(node, Selected::Call { method, call, kind });
        Ok(())
    }

    fn reserve_place_selection(&mut self, node: NodeId, additional: usize) -> Result<(), Error> {
        if self.facts.source_bindings.is_none() {
            return Ok(());
        }
        let current = self.state.language_places.as_deref();
        let (length, capacity) = current.map_or((0, 0), |places| {
            (places.selected.len(), places.selected.capacity())
        });
        let growth = retained::Growth::map::<NodeId, Selected>(length, capacity, additional);
        let bytes = growth.bytes.saturating_add(if current.is_none() {
            size_of::<Places>() + size_of::<Option<Box<Places>>>()
        } else {
            0
        });
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            1usize.saturating_add(growth.work),
            bytes,
        )?;
        let report = self.report;
        let source = self.key(node);
        let places = self
            .state
            .language_places
            .get_or_insert_with(Default::default);
        places.selected.try_reserve(additional).map_err(|_| {
            super::super::Declarations::error(report, source, "language place allocation failed")
        })?;
        if growth.bytes != 0 && retained::map(&places.selected) > growth.bytes {
            return Err(self.error(node, "language place exceeded its paid storage"));
        }
        Ok(())
    }

    pub(in crate::sema::bodies) fn field_plan(
        &self,
        node: NodeId,
    ) -> Option<super::super::access_paths::FieldPlanId> {
        match self.state.language_places.as_ref()?.selected.get(&node)? {
            Selected::Field { plan } => Some(*plan),
            Selected::Call { .. } => None,
        }
    }

    pub(in crate::sema::bodies) fn retain_field_plan(
        &mut self,
        node: NodeId,
        plan: super::super::access_paths::FieldPlanId,
    ) -> Result<(), Error> {
        let previous = self
            .state
            .language_places
            .as_ref()
            .and_then(|places| places.selected.get(&node))
            .copied();
        if previous.is_some_and(|previous| {
            !matches!(previous,
            Selected::Field { plan: actual } if actual == plan)
        }) {
            return Err(self.error(node, "field plan changed its actual selected place"));
        }
        self.reserve_place_selection(node, usize::from(previous.is_none()))?;
        self.state
            .language_places
            .get_or_insert_with(Default::default)
            .selected
            .insert(node, Selected::Field { plan });
        Ok(())
    }

    fn retain_mutable_field(&mut self, node: NodeId) -> Result<(), Error> {
        let current = self.state.language_places.as_deref();
        if current.is_some_and(|places| places.mutable.contains(&node)) {
            return Ok(());
        }
        let growth = current.map_or_else(
            || retained::Growth::map::<NodeId, ()>(0, 0, 1),
            |places| {
                retained::Growth::map::<NodeId, ()>(
                    places.mutable.len(),
                    places.mutable.capacity(),
                    1,
                )
            },
        );
        let bytes = growth.bytes.saturating_add(if current.is_none() {
            size_of::<Places>() + size_of::<Option<Box<Places>>>()
        } else {
            0
        });
        crate::sema::header_attributes::charge_source_retention(
            self.facts,
            1usize.saturating_add(growth.work),
            bytes,
        )?;
        let report = self.report;
        let source = self.key(node);
        let places = self
            .state
            .language_places
            .get_or_insert_with(Default::default);
        places.mutable.try_reserve(1).map_err(|_| {
            super::super::Declarations::error(report, source, "mutable field allocation failed")
        })?;
        if growth.bytes != 0
            && places
                .mutable
                .capacity()
                .saturating_mul(size_of::<NodeId>())
                > growth.bytes
        {
            return Err(self.error(node, "mutable field exceeded its paid storage"));
        }
        places.mutable.insert(node);
        Ok(())
    }
}
