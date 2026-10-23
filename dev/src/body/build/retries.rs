//! Each subtree has first-match and resume entries; products never expand combinations.
use super::{Action, BlockId, Builder, Error, NodeId, Place, Rvalue, Terminator, decisions::Plan};
use crate::{
    lex::token::Punctuation as Op,
    sema::patterns::{Fields, Kind, PatternId},
    types::{Primitive, TypeId},
};

#[derive(Clone, Copy)]
struct Test {
    pattern: PatternId,
    place: Place,
    entry: BlockId,
    retry: BlockId,
    yes: BlockId,
    first_no: BlockId,
    next_no: BlockId,
}
enum Work {
    Test(Test),
    Fields {
        fields: Fields,
        parent: Test,
        entry: BlockId,
        previous: BlockId,
    },
    Or(Alternative),
}
#[derive(Clone, Copy)]
struct Alternative {
    parent: Test,
    next: u32,
    end: u32,
    entry: BlockId,
    dispatch: BlockId,
    exhausted: BlockId,
}
impl Builder<'_> {
    pub(super) fn retry_decisions(
        &mut self,
        plan: &Plan,
        pattern: PatternId,
        anchor: Place,
        yes: BlockId,
        no: BlockId,
    ) -> Result<BlockId, Error> {
        let node = self.facts.patterns.nodes[pattern.0 as usize].source.node;
        let entry = self.new_block(node)?;
        let retry = self.new_block(node)?;
        // Ownership must also see the path on which this guard never ran.
        self.terminate(
            node,
            Terminator::FalseEdge {
                real: entry,
                imaginary: no,
            },
        );
        let mut work = vec![Work::Test(Test {
            pattern,
            place: anchor,
            entry,
            retry,
            yes,
            first_no: no,
            next_no: no,
        })];
        while let Some(task) = work.pop() {
            match task {
                Work::Test(test) => self.retry_test(plan, anchor, test, &mut work)?,
                Work::Fields {
                    fields,
                    parent,
                    entry,
                    previous,
                } => {
                    let field = self.facts.patterns.fields[fields.start as usize];
                    let node = self.facts.patterns.nodes[field.pattern.0 as usize]
                        .source
                        .node;
                    let place =
                        self.pattern_place(field.pattern, parent.place, Some(field.selection))?;
                    let retry = self.new_block(node)?;
                    let last = fields.start + 1 == fields.end;
                    let yes = if last {
                        parent.yes
                    } else {
                        self.new_block(node)?
                    };
                    if last {
                        self.open_retry(parent.retry)?;
                        self.terminate(node, Terminator::Jump(retry));
                    } else {
                        work.push(Work::Fields {
                            fields: Fields {
                                start: fields.start + 1,
                                end: fields.end,
                            },
                            parent,
                            entry: yes,
                            previous: retry,
                        });
                    }
                    // A field with no first match makes the entire product fail.
                    // Only exhaustion after a match resumes a preceding field.
                    work.push(Work::Test(Test {
                        pattern: field.pattern,
                        place,
                        entry,
                        retry,
                        yes,
                        first_no: parent.first_no,
                        next_no: previous,
                    }));
                }
                Work::Or(alternative) => self.retry_alternative(plan, alternative, &mut work)?,
            }
        }
        Ok(retry)
    }
    fn retry_alternative(
        &mut self,
        plan: &Plan,
        alternative: Alternative,
        work: &mut Vec<Work>,
    ) -> Result<(), Error> {
        let Alternative {
            parent,
            next,
            end,
            entry,
            dispatch,
            exhausted,
        } = alternative;

        let node = self.facts.patterns.nodes[parent.pattern.0 as usize]
            .source
            .node;
        let retry = self.new_block(node)?;
        let selected = self.new_block(node)?;
        let last = next + 1 == end;
        let no = if last {
            exhausted
        } else {
            self.new_block(node)?
        };
        self.open_retry(dispatch)?;
        if last {
            self.terminate(node, Terminator::Jump(retry));
        } else {
            let following = self.new_block(node)?;
            let condition = self.choice_test(node, plan, parent.pattern, next)?;
            self.terminate(
                node,
                Terminator::Branch {
                    condition,
                    yes: retry,
                    no: following,
                },
            );
            work.push(Work::Or(Alternative {
                parent,
                next: next + 1,
                end,
                entry: no,
                dispatch: following,
                exhausted,
            }));
        }
        self.open_retry(selected)?;
        let choice = self.choice_value(node, next)?;
        self.store(node, Place::Local(plan.choices[&parent.pattern]), choice)?;
        self.terminate(
            node,
            if last {
                Terminator::Jump(parent.yes)
            } else {
                Terminator::FalseEdge {
                    real: parent.yes,
                    imaginary: no,
                }
            },
        );
        work.push(Work::Test(Test {
            pattern: self.facts.patterns.alternatives[next as usize],
            place: parent.place,
            entry,
            retry,
            yes: selected,
            first_no: no,
            next_no: no,
        }));
        Ok(())
    }
    fn open_retry(&mut self, block: BlockId) -> Result<(), Error> {
        // Emit each continuation once even when its backedge will be built later.
        // Executable and ownership reachability are recomputed from real IR edges.
        self.scratch.reach[block.0 as usize].incoming = true;
        self.switch(self.body.blocks[block.0 as usize].source, block)
    }
    fn choice_test(
        &mut self,
        node: NodeId,
        plan: &Plan,
        pattern: PatternId,
        next: u32,
    ) -> Result<super::ValueId, Error> {
        let input = self.evaluate(
            node,
            TypeId::primitive(Primitive::U32),
            Rvalue::Read(Place::Local(plan.choices[&pattern])),
        )?;
        let value = self.choice_value(node, next)?;
        self.evaluate(
            node,
            TypeId::primitive(Primitive::Bool),
            Rvalue::Binary {
                operands: crate::body::ir::BinaryOperands::Neither,
                operator: Op::EqualEqual,
                left: input,
                right: value,
            },
        )
    }
    fn retry_test(
        &mut self,
        plan: &Plan,
        anchor: Place,
        test: Test,
        work: &mut Vec<Work>,
    ) -> Result<(), Error> {
        let pattern = &self.facts.patterns.nodes[test.pattern.0 as usize];
        let node = pattern.source.node;
        let kind = pattern.kind;
        self.open_retry(test.entry)?;
        match kind {
            Kind::Wildcard
            | Kind::Binding {
                subpattern: None, ..
            }
            | Kind::Range(_)
            | Kind::Text(_) => {
                let value = if matches!(kind, Kind::Range(_) | Kind::Text(_)) {
                    self.select_pattern_place(node, test.place, anchor)?;
                    self.pattern_condition(test.pattern, test.place)?
                } else {
                    None
                };
                self.terminate(
                    node,
                    value.map_or(Terminator::Jump(test.yes), |condition| Terminator::Branch {
                        condition,
                        yes: test.yes,
                        no: test.first_no,
                    }),
                );
                self.open_retry(test.retry)?;
                self.terminate(node, Terminator::Jump(test.next_no));
            }
            Kind::Binding {
                subpattern: Some(pattern),
                ..
            }
            | Kind::Dereference { pattern, .. } => {
                let place = if matches!(kind, Kind::Dereference { .. }) {
                    Place::Projection(self.dereference_projection(node, test.place)?)
                } else {
                    test.place
                };
                let entry = self.new_block(node)?;
                let retry = self.new_block(node)?;
                self.terminate(node, Terminator::Jump(entry));
                self.open_retry(test.retry)?;
                self.terminate(node, Terminator::Jump(retry));
                work.push(Work::Test(Test {
                    pattern,
                    place,
                    entry,
                    retry,
                    ..test
                }));
            }
            Kind::Product { fields, .. } | Kind::Slice { fields, .. } => {
                self.retry_product(anchor, test, fields, work)?;
            }
            Kind::Or(alternatives) => {
                let sentinel = self.choice_value(node, alternatives.end)?;
                self.store(node, Place::Local(plan.choices[&test.pattern]), sentinel)?;
                let entry = self.new_block(node)?;
                let exhausted = self.new_block(node)?;
                self.terminate(node, Terminator::Jump(entry));
                self.open_retry(exhausted)?;
                if test.first_no == test.next_no {
                    self.terminate(node, Terminator::Jump(test.first_no));
                } else {
                    let initial = self.choice_test(node, plan, test.pattern, alternatives.end)?;
                    self.terminate(
                        node,
                        Terminator::Branch {
                            condition: initial,
                            yes: test.first_no,
                            no: test.next_no,
                        },
                    );
                }
                work.push(Work::Or(Alternative {
                    parent: test,
                    next: alternatives.start,
                    end: alternatives.end,
                    entry,
                    dispatch: test.retry,
                    exhausted,
                }));
            }
            _ => return Err(self.error(node, "retry decisions require owned selected patterns")),
        }
        Ok(())
    }
    fn retry_product(
        &mut self,
        anchor: Place,
        test: Test,
        fields: Fields,
        work: &mut Vec<Work>,
    ) -> Result<(), Error> {
        let node = self.facts.patterns.nodes[test.pattern.0 as usize]
            .source
            .node;

        let field_entry = if fields.start == fields.end {
            test.yes
        } else {
            self.new_block(node)?
        };
        let kind = self.facts.patterns.nodes[test.pattern.0 as usize].kind;
        let tested = match kind {
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
            self.select_pattern_place(node, test.place, anchor)?;
            let condition = self
                .pattern_condition(test.pattern, test.place)?
                .ok_or_else(|| self.error(node, "missing selected constructor test"))?;
            self.terminate(
                node,
                Terminator::Branch {
                    condition,
                    yes: field_entry,
                    no: test.first_no,
                },
            );
        } else {
            self.terminate(node, Terminator::Jump(field_entry));
        }
        if fields.start == fields.end {
            self.open_retry(test.retry)?;
            self.terminate(node, Terminator::Jump(test.next_no));
        } else {
            work.push(Work::Fields {
                fields,
                parent: test,
                entry: field_entry,
                previous: test.next_no,
            });
        }

        Ok(())
    }
    fn select_pattern_place(
        &mut self,
        node: NodeId,
        place: Place,
        anchor: Place,
    ) -> Result<(), Error> {
        if place != anchor
            && let Place::Projection(projection) = place
        {
            self.statement(node, Action::Select { projection, anchor })?;
        }
        Ok(())
    }
}
