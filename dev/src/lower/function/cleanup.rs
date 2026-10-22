//! Execute proved cleanup events in body order, splitting only real cleanup edges.
use super::{Lower, ir};
use crate::support::hash::HashMap;
use crate::{
    body::storage::Location,
    borrow::destruction::{Effect, Plan, timeline::Index},
    lowir::{
        builder::Builder,
        program::{BlockId, Terminator},
    },
};

pub(super) struct Pending<'a> {
    pub plan: &'a Plan,
    pub timeline: Index,
    pub state: super::super::cleanup::State,
    edges: HashMap<(ir::BlockId, ir::BlockId), BlockId>,
    lanes: Vec<(BlockId, ir::BlockId, std::ops::Range<usize>)>,
}
impl<'a> Pending<'a> {
    pub fn new(body: &ir::Body, plan: &'a Plan) -> Result<Self, &'static str> {
        Ok(Self {
            plan,
            timeline: Index::new(body, &plan.events)?,
            state: super::super::cleanup::State::default(),
            edges: HashMap::default(),
            lanes: Vec::new(),
        })
    }
    pub fn allocate(
        &mut self,
        builder: &mut Builder,
        body: &ir::Body,
        seen: &[bool],
    ) -> Result<(), &'static str> {
        self.state.allocate(builder, self.plan)?;
        let mut edges: Vec<_> = self
            .timeline
            .edges
            .iter()
            .map(|(&key, range)| (key, range.clone()))
            .collect();
        edges.sort_unstable_by_key(|&((a, b), _)| (a.0, b.0));
        for ((source, target), range) in edges {
            if !seen[source.0 as usize]
                || !real_edge(body.blocks[source.0 as usize].terminator, target)
            {
                continue;
            }
            if !self.plan.events[range.clone()]
                .iter()
                .any(|event| match event.effect {
                    Effect::Cleanup(id) => self.plan.cleanups[id as usize].node.is_some(),
                    Effect::Flags { .. } => true,
                    _ => false,
                })
            {
                continue;
            }
            let block = builder.block()?;
            self.edges.insert((source, target), block);
            self.lanes.push((block, target, range));
        }
        Ok(())
    }
}
fn real_edge(terminator: ir::Terminator, target: ir::BlockId) -> bool {
    match terminator {
        ir::Terminator::Jump(next) | ir::Terminator::FalseEdge { real: next, .. } => next == target,
        ir::Terminator::Branch { yes, no, .. } => yes == target || no == target,
        _ => false,
    }
}
impl Lower<'_, '_, '_> {
    pub(super) fn target(
        &self,
        source: usize,
        target: ir::BlockId,
    ) -> Result<BlockId, &'static str> {
        let source =
            ir::BlockId(u32::try_from(source).map_err(|_| "invalid cleanup branch source")?);
        Ok(self
            .cleanup
            .as_ref()
            .and_then(|p| p.edges.get(&(source, target)))
            .copied()
            .unwrap_or(self.blocks[target.0 as usize]))
    }
    pub(super) fn cleanup_events(
        &mut self,
        range: std::ops::Range<usize>,
    ) -> Result<(), &'static str> {
        let plan = self
            .cleanup
            .as_ref()
            .ok_or("missing cleanup schedule")?
            .plan;
        for event in &plan.events[range] {
            if let Location::At { block, before } = event.location {
                let block = &self.body.blocks[block.0 as usize];
                self.source = if before < block.end {
                    self.body.statements[before as usize].source
                } else {
                    block.source
                };
            }
            match event.effect {
                Effect::Statement => {
                    let Location::At { before, .. } = event.location else {
                        return Err("statement on cleanup edge");
                    };
                    self.statement(self.body.statements[before as usize].action)?;
                }
                Effect::Cleanup(index) => {
                    let cleanup = &plan.cleanups[index as usize];
                    if let Some(node) = cleanup.node {
                        let ty = self.place_type(cleanup.candidate.place)?;
                        let address = self
                            .place(cleanup.candidate.place)?
                            .ok_or("destructor place lacks stable storage")?;
                        let state =
                            &mut self.cleanup.as_mut().ok_or("missing cleanup state")?.state;
                        super::super::cleanup::cleanup(
                            self.context,
                            &mut self.builder,
                            state,
                            plan,
                            node,
                            ty,
                            address,
                        )?;
                    }
                }
                Effect::Flags { range, initialized } => self
                    .cleanup
                    .as_mut()
                    .ok_or("missing cleanup flags")?
                    .state
                    .write(&mut self.builder, range, initialized)?,
                _ => {}
            }
        }
        Ok(())
    }
    pub(super) fn cleanup_edges(&mut self) -> Result<(), &'static str> {
        let count = self.cleanup.as_ref().map_or(0, |p| p.lanes.len());
        for index in 0..count {
            let (block, target, range) =
                self.cleanup.as_ref().ok_or("missing cleanup edges")?.lanes[index].clone();
            self.builder.enter(block)?;
            self.cleanup_events(range)?;
            self.builder
                .terminate(Terminator::Jump(self.blocks[target.0 as usize]))?;
        }
        Ok(())
    }
}
