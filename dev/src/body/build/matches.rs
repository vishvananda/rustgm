//! Scalar decisions preserve source order, place evaluation and arm storage scopes.
use super::{
    BlockId, Builder, Error, LocalId, NodeId, Place, Rvalue, ScalarId, Terminator, ValueId, Work,
};
use crate::{
    lex::token::Punctuation as Op,
    sema::patterns::{Interval, Kind, PatternId, scalar::Domain},
    types::{Kind as Type, Primitive, TypeId},
};

#[derive(Clone, Copy)]
pub(super) struct Frame {
    pub(super) node: NodeId,
    pub(super) place: Place,
    pub(super) next: u32,
    pub(super) end: u32,
    pub(super) merge: Option<LocalId>,
    pub(super) join: BlockId,
    pub(super) entry: Option<BlockId>,
    pub(super) base: usize,
    pub(super) scalar: bool,
    pub(super) tested: Option<(u32, u32)>,
}
impl Builder<'_> {
    pub(super) fn start_match(&mut self, node: NodeId) -> Result<(), Error> {
        let place = self.pop_place(node)?;
        if self.current.is_none() {
            return Ok(());
        }
        let place = place.ok_or_else(|| self.error(node, "missing match input place"))?;
        let id = *self
            .facts
            .patterns
            .matches
            .get(&super::NodeKey {
                file: self.body.source.file,
                node,
            })
            .ok_or_else(|| self.error(node, "missing checked match decisions"))?;
        let decision = &self.facts.patterns.decisions[id as usize];
        let input = self
            .facts
            .node(decision.scrutinee)
            .and_then(|f| f.ty)
            .ok_or_else(|| self.error(node, "missing match input type"))?;
        let scalar = input == TypeId::NEVER
            || matches!(self.facts.types.kind(input), Some(Type::Primitive(p)) if Domain::new(p).is_some());
        let (start, end) = (decision.start, decision.end);
        let tested = if self.facts.patterns.arms[start as usize..end as usize]
            .iter()
            .any(|a| a.guard.is_some())
        {
            Some(self.guard_tested_places(start, end, place)?)
        } else {
            None
        };
        if start == end && input != TypeId::NEVER {
            self.evaluate(node, TypeId::UNIT, Rvalue::Validate(place))?;
        }
        let merge = self.merge(node)?;
        let join = self.new_block(node)?;
        let frame = self.scratch.match_frames.len();
        self.scratch.match_frames.push(Frame {
            node,
            place,
            next: start,
            end,
            merge,
            join,
            entry: None,
            base: self.scratch.live.len(),
            scalar,
            tested,
        });
        self.scratch.work.push(Work::Join {
            node,
            target: join,
            merge,
            control: false,
        });
        self.scratch.work.push(Work::MatchNext(frame));
        Ok(())
    }
    pub(super) fn next_match_arm(&mut self, index: usize) -> Result<(), Error> {
        let frame = self.scratch.match_frames[index];
        if let Some(entry) = frame.entry {
            self.switch(frame.node, entry)?;
        }
        if frame.next == frame.end {
            self.terminate(frame.node, Terminator::Unreachable);
            return Ok(());
        }
        if self.current.is_none() {
            return Ok(());
        }
        let arm = self.facts.patterns.arms[frame.next as usize];
        let pattern = arm
            .pattern
            .ok_or_else(|| self.error(frame.node, "missing arm binding selection"))?;
        if arm.guard.is_some() {
            return self.guarded_arm(index, frame, arm, pattern);
        }
        let yes = self.new_block(arm.source.node)?;
        let no = self.new_block(arm.source.node)?;
        let checked = frame.next + 1 < frame.end;
        let (plan, next) = if frame.scalar {
            let entry = self.current;
            self.scalar_decision(pattern, frame.place, yes, no)?;
            if checked && let Some(entry) = entry {
                self.check_failure(arm.source.node, entry, no)?;
            }
            (None, no)
        } else {
            let plan = self.constructor_decision(
                pattern,
                frame.place,
                yes,
                no,
                if checked {
                    super::decisions::Failure::Checked
                } else {
                    super::decisions::Failure::Runtime
                },
            )?;
            self.switch(arm.source.node, no)?;
            self.cleanup(arm.source.node, frame.base)?;
            let next = self.new_block(arm.source.node)?;
            self.terminate(arm.source.node, Terminator::Jump(next));
            (Some(plan), next)
        };
        self.scratch.match_frames[index].next += 1;
        self.scratch.match_frames[index].entry = Some(next);
        self.scratch.work.push(Work::MatchNext(index));
        self.scratch.work.push(Work::Arm {
            node: Some(arm.value.node),
            source: arm.source.node,
            merge: frame.merge,
            target: frame.join,
            base: frame.base,
            scope_exit: Some(self.boundary_range(arm.source.node, super::scopes::Kind::Arm)?),
        });
        self.scratch.work.push(Work::Enter(arm.value.node));
        self.switch(arm.source.node, yes)?;
        if self.current.is_some() {
            if let Some(plan) = plan {
                self.constructor_captures(&plan, pattern, frame.place)?;
            } else {
                self.scalar_captures(pattern, frame.place)?;
            }
        }
        Ok(())
    }
    fn scalar_decision(
        &mut self,
        root: PatternId,
        place: Place,
        yes: BlockId,
        no: BlockId,
    ) -> Result<(), Error> {
        self.scratch.match_patterns.clear();
        self.scratch.match_patterns.push(root);
        while let Some(id) = self.scratch.match_patterns.pop() {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            let node = pattern.source.node;
            match pattern.kind {
                Kind::Binding {
                    subpattern: Some(child),
                    ..
                } => self.scratch.match_patterns.push(child),
                Kind::Or(alternatives) => self.scratch.match_patterns.extend(
                    self.facts
                        .patterns
                        .alternatives(alternatives)
                        .iter()
                        .rev()
                        .copied(),
                ),
                Kind::Wildcard
                | Kind::Binding {
                    subpattern: None, ..
                } => {
                    self.terminate(node, Terminator::Jump(yes));
                    break;
                }
                Kind::Range(range) => {
                    let ty = pattern
                        .ty
                        .ok_or_else(|| self.error(node, "missing scalar pattern type"))?;
                    let test = self.interval_test(
                        node,
                        ty,
                        place,
                        self.facts.patterns.ranges[range.0 as usize],
                    )?;
                    let Some(test) = test else {
                        self.terminate(node, Terminator::Jump(yes));
                        break;
                    };
                    let next = if self.scratch.match_patterns.is_empty() {
                        no
                    } else {
                        self.new_block(node)?
                    };
                    self.terminate(
                        node,
                        Terminator::Branch {
                            condition: test,
                            yes,
                            no: next,
                        },
                    );
                    if next != no {
                        self.switch(node, next)?;
                    }
                }
                _ => return Err(self.error(node, "match decision requires constructor lowering")),
            }
        }
        self.scratch.match_patterns.clear();
        Ok(())
    }
    pub(super) fn interval_test(
        &mut self,
        node: NodeId,
        ty: TypeId,
        place: Place,
        interval: Interval,
    ) -> Result<Option<ValueId>, Error> {
        let Some(Type::Primitive(primitive)) = self.facts.types.kind(ty) else {
            return Err(self.error(node, "missing scalar match domain"));
        };
        let domain = Domain::new(primitive)
            .ok_or_else(|| self.error(node, "unsupported scalar match domain"))?;
        if interval == domain.full() {
            return Ok(None);
        }
        let input = self.evaluate(node, ty, Rvalue::Read(place))?;
        let low = self.pattern_value(node, ty, domain, interval.lower)?;
        let boolean = TypeId::primitive(Primitive::Bool);
        if interval.lower == interval.upper {
            return self
                .evaluate(
                    node,
                    boolean,
                    Rvalue::Binary {
                        operands: crate::body::ir::BinaryOperands::Neither,
                        operator: Op::EqualEqual,
                        left: input,
                        right: low,
                    },
                )
                .map(Some);
        }
        let high = self.pattern_value(node, ty, domain, interval.upper)?;
        let lower = self.evaluate(
            node,
            boolean,
            Rvalue::Binary {
                operands: crate::body::ir::BinaryOperands::Neither,
                operator: Op::GreaterEqual,
                left: input,
                right: low,
            },
        )?;
        let upper = self.evaluate(
            node,
            boolean,
            Rvalue::Binary {
                operands: crate::body::ir::BinaryOperands::Neither,
                operator: Op::LessEqual,
                left: input,
                right: high,
            },
        )?;
        self.evaluate(
            node,
            boolean,
            Rvalue::Binary {
                operands: crate::body::ir::BinaryOperands::Neither,
                operator: Op::And,
                left: lower,
                right: upper,
            },
        )
        .map(Some)
    }
    fn pattern_value(
        &mut self,
        node: NodeId,
        ty: TypeId,
        domain: Domain,
        rank: u128,
    ) -> Result<ValueId, Error> {
        let scalar = domain.value(rank).map_err(|e| self.error(node, e))?;
        let id = ScalarId(self.index(node, self.body.scalars.len())?);
        self.body.scalars.push(scalar);
        self.evaluate(node, ty, Rvalue::Scalar(id))
    }
    fn scalar_captures(&mut self, root: PatternId, place: Place) -> Result<(), Error> {
        self.scratch.match_patterns.clear();
        self.scratch.match_patterns.push(root);
        while let Some(id) = self.scratch.match_patterns.pop() {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            match pattern.kind {
                Kind::Binding { subpattern, .. } => {
                    if let Some(child) = subpattern {
                        self.scratch.match_patterns.push(child);
                    }
                    self.pattern_capture(id, Some(place))?;
                }
                Kind::Or(alternatives) => {
                    // Every scalar occurrence captures this same place with the
                    // same solved type and written qualifiers. No projections or
                    // captures choose a different source place here.
                    let first = self
                        .facts
                        .patterns
                        .alternatives(alternatives)
                        .first()
                        .copied()
                        .ok_or_else(|| {
                            self.error(pattern.source.node, "empty scalar alternatives")
                        })?;
                    self.scratch.match_patterns.push(first);
                }
                Kind::Wildcard | Kind::Range(_) => {}
                _ => {
                    return Err(self.error(
                        pattern.source.node,
                        "scalar match capture requires later ownership semantics",
                    ));
                }
            }
        }
        Ok(())
    }
}
