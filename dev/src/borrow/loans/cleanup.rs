//! Destructor access and storage endpoints use the same ordered points as liveness.
use super::{Depth, Error, Flow, Mode, Point, State, inspection};
use crate::{
    body::{
        ir::{Action, BlockId, Place},
        storage::Location,
    },
    borrow::{
        destruction::{Effect, schedule::Schedule},
        ownership::initialization::Cause,
    },
};

impl Flow<'_> {
    pub(super) fn cleanup_edge<const RECORD: bool>(
        &mut self,
        block: usize,
        target: BlockId,
        state: &mut [State],
        inspect: bool,
    ) -> Result<(), Error> {
        let error = |message| Error {
            node: self.body.source,
            message,
        };
        let Some(plan) = self.destruction else {
            return Ok(());
        };
        let index = self
            .uses
            .ordered
            .as_ref()
            .ok_or_else(|| error("missing cleanup loan timeline"))?;
        let block = BlockId(u32::try_from(block).map_err(|_| error("invalid cleanup loan block"))?);
        if let Some(range) = index.edges.get(&(block, target)).cloned() {
            self.cleanup_events::<RECORD>(plan, range, state, inspect)?;
        }
        Ok(())
    }
    pub(super) fn cleanup_events<const RECORD: bool>(
        &mut self,
        plan: Schedule<'_>,
        range: std::ops::Range<usize>,
        state: &mut [State],
        inspect: bool,
    ) -> Result<(), Error> {
        for index in range {
            let event = &plan.events[index];
            let mut node = self.body.source;
            if let Location::At { block, before } = event.location {
                let cfg = &self.body.blocks[block.0 as usize];
                node.node = if before < cfg.end {
                    self.body.statements[before as usize].source
                } else {
                    cfg.source
                };
            }
            let error = |message| Error { node, message };
            self.charge(1).map_err(error)?;
            let point =
                Point(u32::try_from(index).map_err(|_| error("too many cleanup loan points"))?);
            self.prune(state, point).map_err(error)?;
            match event.effect {
                Effect::Statement => {
                    let Location::At { before, .. } = event.location else {
                        return Err(error("cleanup statement is on an edge"));
                    };
                    let action = self.body.statements[before as usize].action;
                    if let Action::Evaluate(id) = action {
                        self.activation(state, id, inspect).map_err(error)?;
                    }
                    if RECORD && inspect {
                        inspection::record(&mut self.snapshots, point, state).map_err(error)?;
                    }
                    if inspect {
                        self.inspect(state, action).map_err(error)?;
                    }
                    if let Action::Evaluate(id) = action {
                        let (start, end) = self.generated[id.0 as usize];
                        for loan in start..end {
                            self.charge(1).map_err(error)?;
                            state[loan / 64].live |= 1 << (loan % 64);
                            state[loan / 64].active &= !(1 << (loan % 64));
                        }
                    }
                }
                Effect::Cleanup(id) if inspect => {
                    let cleanup = plan
                        .cleanup(id as usize)
                        .ok_or_else(|| error("invalid cleanup loan identity"))?;
                    if cleanup.access {
                        self.access(
                            state,
                            cleanup.candidate.place,
                            Mode::Mutable,
                            Depth::Deep,
                            None,
                        )
                        .map_err(error)?;
                    }
                    if matches!(
                        cleanup.candidate.cause,
                        Cause::Scope(_) | Cause::Storage | Cause::Initializer
                    ) && matches!(
                        cleanup.candidate.place,
                        Place::Local(_) | Place::Temporary(_)
                    ) {
                        self.access(
                            state,
                            cleanup.candidate.place,
                            Mode::Mutable,
                            Depth::Storage,
                            None,
                        )
                        .map_err(error)?;
                    }
                }
                Effect::StorageEnd(end) if inspect => {
                    self.ending(state, end, None).map_err(error)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
