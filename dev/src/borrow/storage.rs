//! Definite availability of materialized places. Reference loans need a separate proof.
mod index;
mod state;
use crate::body::{
    ir::{Action, BlockId, Body, NodeKey, Place, Rvalue, ValueId},
    storage::End,
};
use index::Index;
pub(crate) use index::{Event, EventKind};
use state::State;

#[derive(Debug)]
pub struct Error {
    pub node: NodeKey,
    pub message: &'static str,
}
pub struct Analysis {
    pub reachable: Vec<bool>,
    pub steps: usize,
    index: Index,
    incoming: Vec<state::Word>,
    words: usize,
}
impl Analysis {
    pub(crate) fn events(&self, block: usize) -> &[Event] {
        self.index.events(block)
    }
    pub(crate) fn has_edge_ends(&self, block: usize) -> bool {
        !self.index.edges(block).is_empty()
    }
    pub(crate) fn ends(&self, block: usize) -> impl Iterator<Item = (u32, End)> + '_ {
        self.index
            .events(block)
            .iter()
            .filter_map(|e| match e.kind {
                EventKind::End(end) => Some((e.position, end)),
                EventKind::Birth(_) => None,
            })
    }
    pub(crate) fn edge_ends(
        &self,
        block: usize,
        target: BlockId,
    ) -> impl Iterator<Item = End> + '_ {
        self.index
            .edges(block)
            .iter()
            .filter(move |e| e.target == target)
            .map(|e| e.end)
    }
    pub(crate) fn temporary(&self, value: ValueId) -> Option<usize> {
        let bit = self.index.values.get(value.0 as usize).copied().flatten()? as usize;
        (!self.index.permanent[bit]).then_some(bit)
    }
    /// Query a materialized value's definite availability at a reachable block entry.
    #[must_use]
    pub fn live_on_entry(&self, block: BlockId, value: ValueId) -> Option<bool> {
        if !self.reachable.get(block.0 as usize).copied()? {
            return None;
        }
        let bit = self.index.values.get(value.0 as usize).copied().flatten()? as usize;
        let word = self
            .incoming
            .get(block.0 as usize * self.words + bit / 64)?;
        Some(word.must & (1 << (bit % 64)) != 0)
    }
    /// Query whether storage may already be live on any incoming path.
    #[must_use]
    pub fn maybe_live_on_entry(&self, block: BlockId, value: ValueId) -> Option<bool> {
        if !self.reachable.get(block.0 as usize).copied()? {
            return None;
        }
        let bit = self.index.values.get(value.0 as usize).copied().flatten()? as usize;
        let word = self
            .incoming
            .get(block.0 as usize * self.words + bit / 64)?;
        Some(word.may & (1 << (bit % 64)) != 0)
    }
}
/// Verify direct place uses against births and explicit CFG storage endpoints.
///
/// This verifies the storage containing a pointer, not the lifetime of its referent.
/// It neither grants an executable borrow-check token nor discharges region contracts.
///
/// # Errors
/// Reports invalid metadata, unavailable temporary places or bounded-dataflow exhaustion.
pub fn check(body: &Body) -> Result<Analysis, Error> {
    check_ordered(body, None)
}
/// `check`, reusing a reachability walk already computed for this body.
pub(crate) fn check_ordered(
    body: &Body,
    order: Option<&super::order::Order>,
) -> Result<Analysis, Error> {
    let error = |message| Error {
        node: body.source,
        message,
    };
    let mut state = State::new(body.temporaries.len());
    let index = Index::new(body, &mut state).map_err(error)?;
    if body.temporaries.is_empty() {
        return untracked(body, index, state, order).map_err(error);
    }
    let (incoming, reachable) = state::solve(body, &index, &mut state).map_err(error)?;
    let words = state.words.len();
    for (block, reached) in reachable.iter().copied().enumerate() {
        if !reached {
            continue;
        }
        state.charge(words).map_err(error)?;
        state
            .words
            .copy_from_slice(&incoming[block * words..(block + 1) * words]);
        transfer(body, &index, &mut state, block, true).map_err(error)?;
        if matches!(
            body.blocks[block].terminator,
            crate::body::ir::Terminator::Return(_)
        ) {
            state.charge(words).map_err(error)?;
            if state
                .words
                .iter()
                .zip(&index.permanent_masks)
                .any(|(word, permanent)| word.may & !permanent != 0)
            {
                return Err(error("temporary storage remains live at return"));
            }
        }
    }
    Ok(Analysis {
        reachable,
        steps: state.steps,
        index,
        incoming,
        words,
    })
}
/// With no materialized temporary there is no storage bit to solve or end:
/// validated scope exits clear nothing. Only reachability and the inspection
/// pass's materialization checks remain, visited in the same block order.
fn untracked(
    body: &Body,
    index: Index,
    mut state: State,
    order: Option<&super::order::Order>,
) -> Result<Analysis, &'static str> {
    let reachable = if let Some(order) = order {
        state.charge(order.blocks.len())?;
        order.reachable.clone()
    } else {
        state::reachable(body, &mut state)?
    };
    for (block, reached) in reachable.iter().copied().enumerate() {
        if !reached {
            continue;
        }
        let data = &body.blocks[block];
        state.charge((data.end - data.start) as usize + 1)?;
        for point in data.start..data.end {
            inspect_action(
                body,
                &index,
                &mut state,
                body.statements[point as usize].action,
            )?;
        }
    }
    Ok(Analysis {
        reachable,
        steps: state.steps,
        index,
        incoming: Vec::new(),
        words: 0,
    })
}
fn transfer(
    body: &Body,
    index: &Index,
    state: &mut State,
    block: usize,
    inspect: bool,
) -> Result<(), &'static str> {
    let block_data = &body.blocks[block];
    let mut events = index.events(block).iter().peekable();
    for point in block_data.start..=block_data.end {
        state.charge(1)?;
        while let Some(event) = events.peek().filter(|event| event.position == point) {
            match event.kind {
                EventKind::Birth(bit) => {
                    if inspect && !index.permanent[bit as usize] && state.maybe_live(bit) {
                        return Err(
                            "temporary storage is re-entered before its prior lifetime ends",
                        );
                    }
                    state.start(bit)?;
                }
                EventKind::End(end) => end_storage(body, index, state, end)?,
            }
            events.next();
        }
        if inspect && point < block_data.end {
            inspect_action(body, index, state, body.statements[point as usize].action)?;
        }
    }
    Ok(())
}
fn end_storage(
    body: &Body,
    index: &Index,
    state: &mut State,
    end: End,
) -> Result<(), &'static str> {
    match end {
        End::Scopes(range) => {
            let scopes = &body
                .storage
                .as_ref()
                .ok_or("missing storage scopes")?
                .scopes;
            let mut next = Some(range.inner);
            while next != range.outer {
                state.charge(1)?;
                let id = next.ok_or("storage exit is not an ancestor range")?;
                for mask in index.members(id) {
                    state.clear(mask.word, mask.bits)?;
                }
                next = scopes[id.0 as usize].parent;
            }
        }
        End::Temporaries { start, end } => {
            let mut bit = start;
            while bit < end {
                let offset = bit % 64;
                let width = (end - bit).min(64 - offset);
                let mask = (u64::MAX >> (64 - width)) << offset;
                state.clear(bit / 64, mask & !index.permanent_masks[bit as usize / 64])?;
                bit += width;
            }
        }
    }
    Ok(())
}
fn inspect_action(
    body: &Body,
    index: &Index,
    state: &mut State,
    action: Action,
) -> Result<(), &'static str> {
    match action {
        Action::Project(id) | Action::Select { projection: id, .. } => {
            let projection = body
                .projections
                .get(id.0 as usize)
                .ok_or("invalid storage projection")?;
            inspect_place(index, state, projection.base)?;
            if let Action::Select { anchor, .. } = action {
                inspect_place(index, state, anchor)?;
            }
        }
        Action::Store { place, .. } | Action::BindView { place, .. } => {
            inspect_place(index, state, place)?;
        }
        Action::Evaluate(id) => {
            let value = body
                .values
                .get(id.0 as usize)
                .ok_or("invalid storage value")?;
            if let Rvalue::Borrow { place, .. }
            | Rvalue::Read(place)
            | Rvalue::Validate(place)
            | Rvalue::Length(place)
            | Rvalue::TextEquals { place, .. }
            | Rvalue::IsVariant { place, .. } = value.kind
            {
                inspect_place(index, state, place)?;
            }
        }
        _ => {}
    }
    Ok(())
}
fn inspect_place(index: &Index, state: &mut State, place: Place) -> Result<(), &'static str> {
    state.charge(1)?;
    if let Some(bit) = index.root(place)?
        && !state.live(bit)
    {
        return Err("temporary place is outside its storage lifetime");
    }
    Ok(())
}
