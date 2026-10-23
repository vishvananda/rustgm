//! Conditional patterns share decisions while retaining success-only binding scopes.
use super::decisions::Failure;
use super::{
    BlockId, Builder, Control, Error, Kind, NodeId, NodeKey, ScopeRange, Terminator,
    Work as BuildWork, scopes,
};
use crate::{lex::token::Punctuation as Op, sema::patterns::Conditional};

#[derive(Clone, Copy)]
pub(super) enum Work {
    Else {
        node: NodeId,
        temporaries: u32,
    },
    Resume {
        node: NodeId,
        join: BlockId,
        base: usize,
        start: usize,
        end: usize,
    },
    Enter {
        failure_exit: Option<ScopeRange>,
        failure: Failure,
        node: NodeId,
        yes: BlockId,
        no: BlockId,
        base: usize,
    },
    Boolean {
        failure_exit: Option<ScopeRange>,
        node: NodeId,
        yes: BlockId,
        no: BlockId,
        base: usize,
    },
    Pattern {
        failure_exit: Option<ScopeRange>,
        failure: Failure,
        node: NodeId,
        yes: BlockId,
        no: BlockId,
        base: usize,
    },
}
impl Builder<'_> {
    pub(super) fn conditional_chain(&self, node: NodeId) -> bool {
        !self.facts.patterns.chains.is_empty()
            && self.facts.patterns.chains.contains(&NodeKey {
                file: self.source.file,
                node,
            })
    }
    fn conditional(&self, node: NodeId) -> Result<Conditional, Error> {
        self.facts
            .patterns
            .conditions
            .get(&NodeKey {
                file: self.source.file,
                node,
            })
            .copied()
            .ok_or_else(|| self.error(node, "missing checked conditional pattern"))
    }
    pub(super) fn conditional_step(&mut self, work: Work) -> Result<(), Error> {
        match work {
            Work::Else { node, temporaries } => self.let_else(node, temporaries),
            Work::Resume {
                node,
                join,
                base,
                start,
                end,
            } => self.resume_let_else(node, join, base, start, end),
            Work::Enter {
                failure_exit,
                failure,
                node,
                yes,
                no,
                base,
            } => {
                if self.current.is_none() {
                    return Ok(());
                }
                if let Kind::Binary {
                    op: Op::AndAnd,
                    left,
                    right,
                } = self.kind(node)
                    && self.conditional_chain(node)
                {
                    let next = self.new_block(node)?;
                    self.scratch.work.push(BuildWork::Conditional(Work::Enter {
                        failure_exit,
                        failure,
                        node: right,
                        yes,
                        no,
                        base,
                    }));
                    self.scratch.work.push(BuildWork::Switch(next));
                    self.scratch.work.push(BuildWork::Conditional(Work::Enter {
                        failure_exit,
                        failure,
                        node: left,
                        yes: next,
                        no,
                        base,
                    }));
                } else if matches!(self.kind(node), Kind::LetCondition { .. }) {
                    let selected = self.conditional(node)?;
                    self.scratch
                        .work
                        .push(BuildWork::Conditional(Work::Pattern {
                            failure_exit,
                            failure,
                            node,
                            yes,
                            no,
                            base,
                        }));
                    self.scratch
                        .work
                        .push(BuildWork::Place(selected.input.node));
                } else {
                    self.scratch
                        .work
                        .push(BuildWork::Conditional(Work::Boolean {
                            failure_exit,
                            node,
                            yes,
                            no,
                            base,
                        }));
                    self.scratch.work.push(BuildWork::Enter(node));
                }
                Ok(())
            }
            Work::Boolean {
                failure_exit,
                node,
                yes,
                no,
                base,
            } => self.conditional_boolean(node, yes, no, base, failure_exit),
            Work::Pattern {
                failure_exit,
                failure,
                node,
                yes,
                no,
                base,
            } => self.conditional_test(node, yes, no, base, failure, failure_exit),
        }
    }
    fn conditional_boolean(
        &mut self,
        node: NodeId,
        yes: BlockId,
        no: BlockId,
        base: usize,
        failure_exit: Option<ScopeRange>,
    ) -> Result<(), Error> {
        if self.current.is_some() {
            let failed = self.new_block(node)?;
            self.terminate(
                node,
                Terminator::Branch {
                    condition: self.value(node)?,
                    yes,
                    no: failed,
                },
            );
            self.switch(node, failed)?;
            if let Some(scopes) = failure_exit {
                self.exit_scopes(node, scopes)?;
            }
            self.cleanup(node, base)?;
            self.terminate(node, Terminator::Jump(no));
        }
        Ok(())
    }
    fn resume_let_else(
        &mut self,
        node: NodeId,
        join: BlockId,
        base: usize,
        start: usize,
        end: usize,
    ) -> Result<(), Error> {
        if self.current.is_some()
            || self.scratch.conditional_live.len() != end
            || self.scratch.live.len() != base
        {
            return Err(self.error(
                node,
                "let-else failure did not close its storage and control flow",
            ));
        }
        self.scratch
            .live
            .extend_from_slice(&self.scratch.conditional_live[start..end]);
        self.scratch.conditional_live.truncate(start);
        self.switch(node, join)?;
        if self.current.is_some() {
            let unit = self.unit(node)?;
            self.save(node, unit)?;
        }
        Ok(())
    }
    fn conditional_test(
        &mut self,
        node: NodeId,
        yes: BlockId,
        no: BlockId,
        base: usize,
        failure: Failure,
        failure_exit: Option<ScopeRange>,
    ) -> Result<(), Error> {
        let input = self.pop_place(node)?;
        if self.current.is_none() {
            return Ok(());
        }
        let place = input.ok_or_else(|| self.error(node, "missing conditional input place"))?;
        let selected = self.conditional(node)?;
        let accepted = self.new_block(node)?;
        let rejected = self.new_block(node)?;
        let plan =
            self.constructor_decision(selected.pattern, place, accepted, rejected, failure)?;
        self.switch(node, rejected)?;
        if let Some(scopes) = failure_exit {
            self.exit_scopes(node, scopes)?;
        }
        self.cleanup(node, base)?;
        self.terminate(node, Terminator::Jump(no));
        self.switch(node, accepted)?;
        self.constructor_captures(&plan, selected.pattern, place)?;
        self.terminate(node, Terminator::Jump(yes));
        Ok(())
    }
    fn let_else(&mut self, node: NodeId, temporaries: u32) -> Result<(), Error> {
        let input = self.pop_place(node)?;
        if self.current.is_none() {
            return Ok(());
        }
        let place = input.ok_or_else(|| self.error(node, "missing let-else input place"))?;
        let selected = self.conditional(node)?;
        let otherwise = selected
            .otherwise
            .ok_or_else(|| self.error(node, "missing let-else failure block"))?
            .node;
        let base = self.scratch.live.len();
        let yes = self.new_block(node)?;
        let no = self.new_block(node)?;
        let failure = self.new_block(otherwise)?;
        let join = self.new_block(node)?;
        let plan = self.constructor_decision(
            selected.pattern,
            place,
            yes,
            no,
            super::decisions::Failure::Checked,
        )?;
        self.switch(node, no)?;
        self.end_initializer(node, temporaries)?;
        self.cleanup(node, base)?;
        self.terminate(node, Terminator::Jump(failure));
        self.switch(node, yes)?;
        self.constructor_captures(&plan, selected.pattern, place)?;
        self.terminate(node, Terminator::Jump(join));
        // The failure block must see only the surrounding scope. Store just the
        // success tail in shared LIFO scratch; never copy all preceding locals.
        let start = self.scratch.conditional_live.len();
        self.scratch
            .conditional_live
            .extend_from_slice(&self.scratch.live[base..]);
        let end = self.scratch.conditional_live.len();
        self.scratch.live.truncate(base);
        self.scratch.work.push(BuildWork::Conditional(Work::Resume {
            node,
            join,
            base,
            start,
            end,
        }));
        self.scratch.work.push(BuildWork::Enter(otherwise));
        self.scratch.work.push(BuildWork::Switch(failure));
        Ok(())
    }
    pub(super) fn conditional_choice(
        &mut self,
        node: NodeId,
        condition: NodeId,
        then: NodeId,
        otherwise: Option<NodeId>,
    ) -> Result<(), Error> {
        let scope_exit = self.boundary_range(node, scopes::Kind::IfThen)?;
        let merge = self.merge(node)?;
        let base = self.scratch.live.len();
        let yes = self.new_block(node)?;
        let no = self.new_block(node)?;
        let join = self.new_block(node)?;
        self.scratch.work.push(BuildWork::Join {
            node,
            target: join,
            merge,
            control: false,
        });
        for (block, value) in [(no, otherwise), (yes, Some(then))] {
            self.scratch.work.push(BuildWork::Arm {
                node: value,
                source: node,
                merge,
                target: join,
                base,
                scope_exit: (block == yes).then_some(scope_exit),
            });
            if let Some(value) = value {
                self.scratch.work.push(BuildWork::Enter(value));
            }
            self.scratch.work.push(BuildWork::Switch(block));
        }
        self.scratch.work.push(BuildWork::Conditional(Work::Enter {
            failure_exit: Some(scope_exit),
            failure: Failure::Checked,
            node: condition,
            yes,
            no,
            base,
        }));
        Ok(())
    }
    pub(super) fn conditional_loop(
        &mut self,
        node: NodeId,
        condition: NodeId,
        body: NodeId,
    ) -> Result<(), Error> {
        let scope_exit = self.boundary_range(node, scopes::Kind::Iteration)?;
        let base = self.scratch.live.len();
        let header = self.new_block(node)?;
        let run = self.new_block(body)?;
        let exit = self.new_block(node)?;
        self.scratch.controls.insert(
            node,
            Control {
                exit,
                repeat: Some(header),
                merge: None,
                live_base: base,
                scope: self.node_scope(node)?,
            },
        );
        self.terminate(node, Terminator::Jump(header));
        self.scratch.work.push(BuildWork::Join {
            node,
            target: exit,
            merge: None,
            control: true,
        });
        self.scratch.work.push(BuildWork::Arm {
            node: None,
            source: node,
            merge: None,
            target: header,
            base,
            scope_exit: Some(scope_exit),
        });
        self.scratch.work.push(BuildWork::Enter(body));
        self.scratch.work.push(BuildWork::Switch(run));
        self.scratch.work.push(BuildWork::Conditional(Work::Enter {
            failure_exit: Some(scope_exit),
            failure: Failure::Checked,
            node: condition,
            yes: run,
            no: exit,
            base,
        }));
        self.scratch.work.push(BuildWork::Switch(header));
        Ok(())
    }
}
