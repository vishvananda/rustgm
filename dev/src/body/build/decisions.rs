//! Source-ordered constructor decisions retain alternative choices until captures commit.
use super::{
    BlockId, Builder, Error, LocalId, LocalKind, NodeId, Place, Rvalue, ScalarId, Terminator,
};
use crate::{
    lex::token::Punctuation as Op,
    sema::{
        model::Scalar,
        patterns::{Capture, Fields, Kind, PatternId, Projection},
    },
    types::{Primitive, TypeId},
};
use std::collections::{HashMap, HashSet};

pub(super) struct Plan {
    choices: HashMap<PatternId, LocalId>,
    bindings: Vec<PatternId>,
}
#[derive(Clone, Copy)]
struct Test {
    pattern: PatternId,
    base: Place,
    selection: Option<Projection>,
    yes: BlockId,
    no: BlockId,
}
enum Decision {
    Test(Test),
    Fields {
        fields: Fields,
        base: Place,
        node: NodeId,
        yes: BlockId,
        no: BlockId,
    },
    Alternatives {
        pattern: PatternId,
        base: Place,
        next: u32,
        end: u32,
        yes: BlockId,
        no: BlockId,
    },
    Selected {
        pattern: PatternId,
        alternative: u32,
        target: BlockId,
    },
    Switch(BlockId),
}
enum Commit {
    Pattern(PatternId, Place, Option<Projection>),
    Binding(PatternId, Place),
    Alternatives {
        pattern: PatternId,
        base: Place,
        next: u32,
        end: u32,
        join: BlockId,
    },
    Jump(NodeId, BlockId),
    Switch(BlockId),
}
impl Builder<'_> {
    #[expect(
        clippy::too_many_lines,
        reason = "flat decision worklist keeps ordered tests, choices and continuations nonrecursive"
    )]
    pub(super) fn constructor_decision(
        &mut self,
        root: PatternId,
        place: Place,
        yes: BlockId,
        no: BlockId,
    ) -> Result<Plan, Error> {
        let plan = self.prepare_decisions(root)?;
        let mut work = vec![Decision::Test(Test {
            pattern: root,
            base: place,
            selection: None,
            yes,
            no,
        })];
        while let Some(task) = work.pop() {
            match task {
                Decision::Switch(block) => {
                    self.switch(self.body.blocks[block.0 as usize].source, block)?;
                }
                _ if self.current.is_none() => {}
                Decision::Test(test) => self.constructor_test(test, &mut work)?,
                Decision::Fields {
                    fields,
                    base,
                    node,
                    yes,
                    no,
                } => {
                    if fields.start == fields.end {
                        self.terminate(node, Terminator::Jump(yes));
                        continue;
                    }
                    let field = self.facts.patterns.fields[fields.start as usize];
                    let next = if fields.start + 1 == fields.end {
                        yes
                    } else {
                        self.new_block(node)?
                    };
                    if next != yes {
                        work.push(Decision::Fields {
                            fields: Fields {
                                start: fields.start + 1,
                                end: fields.end,
                            },
                            base,
                            node,
                            yes,
                            no,
                        });
                        work.push(Decision::Switch(next));
                    }
                    work.push(Decision::Test(Test {
                        pattern: field.pattern,
                        base,
                        selection: Some(field.selection),
                        yes: next,
                        no,
                    }));
                }
                Decision::Alternatives {
                    pattern,
                    base,
                    next,
                    end,
                    yes,
                    no,
                } => {
                    let node = self.facts.patterns.nodes[pattern.0 as usize].source.node;
                    let fail = if next + 1 == end {
                        no
                    } else {
                        self.new_block(node)?
                    };
                    let success = if plan.choices.contains_key(&pattern) {
                        self.new_block(node)?
                    } else {
                        yes
                    };
                    if fail != no {
                        work.push(Decision::Alternatives {
                            pattern,
                            base,
                            next: next + 1,
                            end,
                            yes,
                            no,
                        });
                        work.push(Decision::Switch(fail));
                    }
                    if success != yes {
                        work.push(Decision::Selected {
                            pattern,
                            alternative: next,
                            target: yes,
                        });
                        work.push(Decision::Switch(success));
                    }
                    work.push(Decision::Test(Test {
                        pattern: self.facts.patterns.alternatives[next as usize],
                        base,
                        selection: None,
                        yes: success,
                        no: fail,
                    }));
                }
                Decision::Selected {
                    pattern,
                    alternative,
                    target,
                } => {
                    let node = self.facts.patterns.nodes[pattern.0 as usize].source.node;
                    let local = plan.choices[&pattern];
                    let value = self.choice_value(node, alternative)?;
                    self.store(node, Place::Local(local), value)?;
                    self.terminate(node, Terminator::Jump(target));
                }
            }
        }
        Ok(plan)
    }
    fn prepare_decisions(&mut self, root: PatternId) -> Result<Plan, Error> {
        let mut order = Vec::new();
        let mut work = vec![root];
        let mut bindings = Vec::new();
        let mut definitions = HashSet::new();
        while let Some(id) = work.pop() {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            order.push(id);
            match pattern.kind {
                Kind::Binding {
                    definition,
                    capture: Capture::Value,
                    subpattern,
                    ..
                } => {
                    if definitions.insert(definition) {
                        bindings.push(id);
                    }
                    work.extend(subpattern);
                }
                Kind::Product { fields, .. } => work.extend(
                    self.facts
                        .patterns
                        .fields(fields)
                        .iter()
                        .rev()
                        .map(|f| f.pattern),
                ),
                Kind::Or(alternatives) => work.extend(
                    self.facts
                        .patterns
                        .alternatives(alternatives)
                        .iter()
                        .rev()
                        .copied(),
                ),
                Kind::Wildcard | Kind::Range(_) => {}
                _ => {
                    return Err(self.error(
                        pattern.source.node,
                        "constructor execution requires owned selected patterns",
                    ));
                }
            }
        }
        let mut captures = HashSet::new();
        for &id in order.iter().rev() {
            let has_capture = match self.facts.patterns.nodes[id.0 as usize].kind {
                Kind::Binding { .. } => true,
                Kind::Product { fields, .. } => self
                    .facts
                    .patterns
                    .fields(fields)
                    .iter()
                    .any(|f| captures.contains(&f.pattern)),
                Kind::Or(alternatives) => self
                    .facts
                    .patterns
                    .alternatives(alternatives)
                    .iter()
                    .any(|a| captures.contains(a)),
                _ => false,
            };
            if has_capture {
                captures.insert(id);
            }
        }
        let mut choices = HashMap::new();
        for id in order {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            if matches!(pattern.kind, Kind::Or(_)) && captures.contains(&id) {
                let node = pattern.source.node;
                let local = self.local(
                    node,
                    TypeId::primitive(Primitive::U32),
                    LocalKind::Merge,
                    None,
                    true,
                )?;
                // A nested choice may be skipped by its parent. Initialize it once
                // so ownership proof does not need value-sensitive path correlation.
                let zero = self.choice_value(node, 0)?;
                self.store(node, Place::Local(local), zero)?;
                choices.insert(id, local);
            }
        }
        Ok(Plan { choices, bindings })
    }
    fn constructor_test(&mut self, test: Test, work: &mut Vec<Decision>) -> Result<(), Error> {
        let Test {
            pattern: id,
            base,
            selection,
            yes,
            no,
        } = test;
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let ty = pattern
            .ty
            .ok_or_else(|| self.error(node, "missing constructor input type"))?;
        let kind = pattern.kind;
        let place = self.selected_place(id, base, selection)?;
        match kind {
            Kind::Wildcard
            | Kind::Binding {
                subpattern: None, ..
            } => self.terminate(node, Terminator::Jump(yes)),
            Kind::Binding {
                subpattern: Some(pattern),
                ..
            } => work.push(Decision::Test(Test {
                pattern,
                base: place,
                selection: None,
                yes,
                no,
            })),
            Kind::Range(range) => {
                let test = self.interval_test(
                    node,
                    ty,
                    place,
                    self.facts.patterns.ranges[range.0 as usize],
                )?;
                self.terminate(
                    node,
                    test.map_or(Terminator::Jump(yes), |condition| Terminator::Branch {
                        condition,
                        yes,
                        no,
                    }),
                );
            }
            Kind::Product { variant, fields } => {
                work.push(Decision::Fields {
                    fields,
                    base: place,
                    node,
                    yes,
                    no,
                });
                if let Some(variant) = variant {
                    let selected = &self.facts.members.variants[variant.0 as usize];
                    if self.facts.members.variants(selected.owner).len() > 1 {
                        let condition = self.evaluate(
                            node,
                            TypeId::primitive(Primitive::Bool),
                            Rvalue::IsVariant { place, variant },
                        )?;
                        let fields = self.new_block(node)?;
                        self.terminate(
                            node,
                            Terminator::Branch {
                                condition,
                                yes: fields,
                                no,
                            },
                        );
                        work.push(Decision::Switch(fields));
                    }
                }
            }
            Kind::Or(alternatives) => work.push(Decision::Alternatives {
                pattern: id,
                base: place,
                next: alternatives.start,
                end: alternatives.end,
                yes,
                no,
            }),
            _ => return Err(self.error(node, "incomplete owned constructor selection")),
        }
        Ok(())
    }
    pub(super) fn constructor_captures(
        &mut self,
        plan: &Plan,
        root: PatternId,
        place: Place,
    ) -> Result<(), Error> {
        if self.current.is_none() || plan.bindings.is_empty() {
            return Ok(());
        }
        for &id in &plan.bindings {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            let node = pattern.source.node;
            let Kind::Binding {
                definition,
                mutable,
                ..
            } = pattern.kind
            else {
                return Err(self.error(node, "invalid constructor capture plan"));
            };
            let ty = pattern
                .ty
                .ok_or_else(|| self.error(node, "missing capture type"))?;
            self.local(node, ty, LocalKind::Binding, Some(definition), mutable)?;
        }
        let mut work = vec![Commit::Pattern(root, place, None)];
        while let Some(task) = work.pop() {
            match task {
                Commit::Switch(block) => {
                    self.switch(self.body.blocks[block.0 as usize].source, block)?;
                }
                _ if self.current.is_none() => {}
                Commit::Jump(node, block) => self.terminate(node, Terminator::Jump(block)),
                Commit::Pattern(id, base, selection) => {
                    self.capture_pattern(plan, id, base, selection, &mut work)?;
                }
                Commit::Binding(id, place) => {
                    let pattern = &self.facts.patterns.nodes[id.0 as usize];
                    let node = pattern.source.node;
                    let Kind::Binding { definition, .. } = pattern.kind else {
                        return Err(self.error(node, "missing constructor capture binding"));
                    };
                    let target = self.local_place(node, definition)?;
                    let ty = pattern
                        .ty
                        .ok_or_else(|| self.error(node, "missing capture type"))?;
                    let value = self.evaluate(node, ty, Rvalue::Read(place))?;
                    self.store(node, target, value)?;
                }
                Commit::Alternatives {
                    pattern,
                    base,
                    next,
                    end,
                    join,
                } => {
                    let node = self.facts.patterns.nodes[pattern.0 as usize].source.node;
                    if next + 1 < end {
                        let yes = self.new_block(node)?;
                        let no = self.new_block(node)?;
                        let ty = TypeId::primitive(Primitive::U32);
                        let input = self.evaluate(
                            node,
                            ty,
                            Rvalue::Read(Place::Local(plan.choices[&pattern])),
                        )?;
                        let value = self.choice_value(node, next)?;
                        let condition = self.evaluate(
                            node,
                            TypeId::primitive(Primitive::Bool),
                            Rvalue::Binary {
                                operator: Op::EqualEqual,
                                left: input,
                                right: value,
                            },
                        )?;
                        self.terminate(node, Terminator::Branch { condition, yes, no });
                        work.push(Commit::Alternatives {
                            pattern,
                            base,
                            next: next + 1,
                            end,
                            join,
                        });
                        work.push(Commit::Switch(no));
                        work.push(Commit::Jump(node, join));
                        work.push(Commit::Pattern(
                            self.facts.patterns.alternatives[next as usize],
                            base,
                            None,
                        ));
                        work.push(Commit::Switch(yes));
                    } else {
                        work.push(Commit::Jump(node, join));
                        work.push(Commit::Pattern(
                            self.facts.patterns.alternatives[next as usize],
                            base,
                            None,
                        ));
                    }
                }
            }
        }
        Ok(())
    }
    fn capture_pattern(
        &mut self,
        plan: &Plan,
        id: PatternId,
        base: Place,
        selection: Option<Projection>,
        work: &mut Vec<Commit>,
    ) -> Result<(), Error> {
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let kind = pattern.kind;
        let place = self.selected_place(id, base, selection)?;
        match kind {
            Kind::Binding { subpattern, .. } => {
                work.push(Commit::Binding(id, place));
                if let Some(child) = subpattern {
                    work.push(Commit::Pattern(child, place, None));
                }
            }
            Kind::Product { fields, .. } => {
                for field in self.facts.patterns.fields(fields).iter().rev() {
                    work.push(Commit::Pattern(field.pattern, place, Some(field.selection)));
                }
            }
            Kind::Or(alternatives) if plan.choices.contains_key(&id) => {
                let join = self.new_block(node)?;
                work.push(Commit::Switch(join));
                work.push(Commit::Alternatives {
                    pattern: id,
                    base: place,
                    next: alternatives.start,
                    end: alternatives.end,
                    join,
                });
            }
            Kind::Wildcard | Kind::Range(_) | Kind::Or(_) => {}
            _ => return Err(self.error(node, "invalid owned capture selection")),
        }
        Ok(())
    }
    fn choice_value(&mut self, node: NodeId, value: u32) -> Result<super::ValueId, Error> {
        let scalar = ScalarId(self.index(node, self.body.scalars.len())?);
        self.body.scalars.push(Scalar::Integer(u128::from(value)));
        self.evaluate(
            node,
            TypeId::primitive(Primitive::U32),
            Rvalue::Scalar(scalar),
        )
    }
}
