//! Guard attempts share a body, protected views and source-ordered retry continuations.
use super::{
    Action, BlockId, Builder, Error, Place, Terminator, Work, conditions,
    decisions::{Failure, Plan},
    matches,
};
use crate::{
    body::ir::{Guard, GuardId},
    sema::patterns::{Arm, Kind, PatternId, scalar::Domain},
    types::Kind as Type,
};

pub(super) struct Frame {
    scope_exit: super::ScopeRange,
    plan: Plan,
    pattern: PatternId,
    parent: matches::Frame,
    arm: Arm,
    guard: GuardId,
    success: BlockId,
    rejected: BlockId,
    retry: BlockId,
}
impl Builder<'_> {
    pub(super) fn guard_tested_places(
        &mut self,
        start: u32,
        end: u32,
        input: Place,
    ) -> Result<(u32, u32), Error> {
        let mut work = Vec::new();
        let first = self.index(self.body.source.node, self.body.guard_places.len())?;
        for arm in &self.facts.patterns.arms[start as usize..end as usize] {
            if let Some(pattern) = arm.pattern {
                work.push((pattern, input, None));
            }
        }
        while let Some((id, base, projection)) = work.pop() {
            let place = self.pattern_place(id, base, projection)?;
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            match pattern.kind {
                Kind::Binding { subpattern, .. } => {
                    work.extend(subpattern.map(|id| (id, place, None)));
                }
                Kind::Dereference { pattern: child, .. } => {
                    let referent = self.dereference_projection(pattern.source.node, place)?;
                    self.body.guard_places.push(place);
                    work.push((child, Place::Projection(referent), None));
                }
                Kind::Product { fields, .. } | Kind::Slice { fields, .. } => {
                    let tested = match pattern.kind {
                        Kind::Slice { .. } => true,
                        Kind::Product {
                            variant: Some(v), ..
                        } => {
                            self.facts
                                .members
                                .variants(self.facts.members.variants[v.0 as usize].owner)
                                .len()
                                > 1
                        }
                        _ => false,
                    };
                    if tested {
                        self.body.guard_places.push(place);
                    }
                    work.extend(
                        self.facts
                            .patterns
                            .fields(fields)
                            .iter()
                            .rev()
                            .map(|f| (f.pattern, place, Some(f.selection))),
                    );
                }
                Kind::Or(alternatives) => work.extend(
                    self.facts
                        .patterns
                        .alternatives(alternatives)
                        .iter()
                        .rev()
                        .map(|id| (*id, place, None)),
                ),
                Kind::Range(range) => {
                    let domain = pattern
                        .ty
                        .and_then(|ty| self.facts.types.kind(ty))
                        .and_then(|ty| {
                            if let Type::Primitive(p) = ty {
                                Domain::new(p)
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| {
                            self.error(pattern.source.node, "missing guarded scalar domain")
                        })?;
                    if self.facts.patterns.ranges[range.0 as usize] != domain.full() {
                        self.body.guard_places.push(place);
                    }
                }
                Kind::Text(_) => self.body.guard_places.push(place),
                Kind::Wildcard => {}
                Kind::Pending => {
                    return Err(self.error(pattern.source.node, "incomplete guarded pattern"));
                }
            }
        }
        Ok((
            first,
            self.index(self.body.source.node, self.body.guard_places.len())?,
        ))
    }
    pub(super) fn guarded_arm(
        &mut self,
        index: usize,
        parent: matches::Frame,
        arm: Arm,
        pattern: PatternId,
    ) -> Result<(), Error> {
        let node = arm.source.node;
        let condition = arm
            .guard
            .ok_or_else(|| self.error(node, "missing selected guard"))?
            .node;
        let arm_scope = self.boundary_range(node, super::scopes::Kind::Arm)?;
        let scope_exit = super::ScopeRange {
            inner: self.boundary(condition, super::scopes::Kind::Guard)?,
            outer: arm_scope.outer,
        };
        let plan = self.prepare_decisions(pattern, true)?;
        let attempt_base = self.scratch.live.len();
        let yes = self.new_block(node)?;
        let no = self.new_block(node)?;
        let retry = self.retry_decisions(&plan, pattern, parent.place, yes, no)?;
        self.switch(node, no)?;
        self.cleanup(node, parent.base)?;
        let next = self.new_block(node)?;
        self.terminate(node, Terminator::Jump(next));
        self.scratch.match_frames[index].next += 1;
        self.scratch.match_frames[index].entry = Some(next);
        self.scratch.work.push(Work::MatchNext(index));
        self.switch(node, yes)?;
        let guard = GuardId(self.index(node, self.body.guards.len())?);
        let (tested_start, tested_end) = parent
            .tested
            .ok_or_else(|| self.error(node, "missing tested guard places"))?;
        let views_start = self.index(node, self.body.locals.len())?;
        self.body.guards.push(Guard {
            tested_start,
            tested_end,
            views_start,
            views_end: views_start,
        });
        self.statement(node, Action::GuardStart(guard))?;
        self.selected_captures(&plan, pattern, parent.place, Some(guard))?;
        let success = self.new_block(node)?;
        let rejected = self.new_block(node)?;
        let frame = self.scratch.guard_frames.len();
        self.scratch.guard_frames.push(Some(Frame {
            scope_exit,
            plan,
            pattern,
            parent,
            arm,
            guard,
            success,
            rejected,
            retry,
        }));
        self.scratch.work.push(Work::GuardFinish(frame));
        self.scratch
            .work
            .push(Work::Conditional(conditions::Work::Enter {
                failure_exit: Some(scope_exit),
                failure: Failure::Runtime,
                node: condition,
                yes: success,
                no: rejected,
                base: attempt_base,
            }));
        Ok(())
    }
    pub(super) fn finish_guard(&mut self, index: usize) -> Result<(), Error> {
        let frame = self.scratch.guard_frames[index]
            .take()
            .ok_or_else(|| self.error(self.body.source.node, "missing guard continuation"))?;
        let node = frame.arm.source.node;
        self.switch(node, frame.rejected)?;
        self.terminate(node, Terminator::Jump(frame.retry));
        self.switch(node, frame.success)?;
        if self.current.is_some() {
            // Only successful guards need the final fake uses. A failed or
            // diverging conditional guard must not acquire a new protection.
            self.statement(node, Action::GuardUse(frame.guard))?;
            self.constructor_captures(&frame.plan, frame.pattern, frame.parent.place)?;
        }
        self.scratch.work.push(Work::Arm {
            node: Some(frame.arm.value.node),
            source: node,
            merge: frame.parent.merge,
            target: frame.parent.join,
            base: frame.parent.base,
            scope_exit: Some(frame.scope_exit),
        });
        self.scratch.work.push(Work::Enter(frame.arm.value.node));
        Ok(())
    }
}
