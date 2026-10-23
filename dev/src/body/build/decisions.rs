//! Source-ordered constructor decisions retain alternative choices until captures commit.
use super::{
    BlockId, Builder, Error, LocalId, LocalKind, NodeId, Place, Rvalue, ScalarId, Terminator,
};
use crate::support::hash::{HashMap, HashSet};
use crate::{
    lex::token::Punctuation as Op,
    sema::{
        model::Scalar,
        patterns::{Capture, Fields, Kind, PatternId, Projection},
    },
    types::{Primitive, TypeId},
};

pub(super) struct Plan {
    pub(super) choices: HashMap<PatternId, LocalId>,
    bindings: Vec<PatternId>,
    borrowed: HashSet<crate::resolve::model::DefId>,
    capture_choices: Option<HashSet<PatternId>>,
}
#[derive(Clone, Copy)]
pub(super) enum Failure {
    Runtime,
    Checked,
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
        failure: Failure,
    ) -> Result<Plan, Error> {
        let plan = self.prepare_decisions(root, false)?;
        let entry = self.current;
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
                    self.switch(
                        self.body
                            .syntax_site(self.body.blocks[block.0 as usize].source)
                            .map_err(|message| self.report.body_error(self.body.source, message))?
                            .node,
                        block,
                    )?;
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
        if matches!(failure, Failure::Checked)
            && let Some(entry) = entry
        {
            let node = self.facts.patterns.nodes[root.0 as usize].source.node;
            self.check_failure(node, entry, no)?;
        }
        Ok(plan)
    }
    /// Keep statically impossible failures in ownership flow without changing execution.
    pub(super) fn check_failure(
        &mut self,
        node: NodeId,
        entry: BlockId,
        no: BlockId,
    ) -> Result<(), Error> {
        if self.scratch.reach[no.0 as usize].incoming {
            return Ok(());
        }
        let terminator = self.body.blocks[entry.0 as usize].terminator;
        let real = match terminator {
            Terminator::Jump(target) => target,
            Terminator::Branch { .. } => {
                let target = self.new_block(node)?;
                let end = self.index(node, self.body.statements.len())?;
                let block = &mut self.body.blocks[target.0 as usize];
                block.start = end;
                block.end = end;
                block.terminator = terminator;
                self.scratch.reach[target.0 as usize] = super::Reach {
                    incoming: true,
                    opened: true,
                };
                target
            }
            _ => return Err(self.error(node, "missing decision entry for checked failure")),
        };
        self.body.blocks[entry.0 as usize].terminator = Terminator::FalseEdge {
            real,
            imaginary: no,
        };
        self.scratch.reach[no.0 as usize].incoming = true;
        Ok(())
    }
    pub(super) fn prepare_decisions(
        &mut self,
        root: PatternId,
        retries: bool,
    ) -> Result<Plan, Error> {
        let mut order = Vec::new();
        let mut work = vec![root];
        let mut bindings = Vec::new();
        let mut borrowed = HashSet::default();
        let mut definitions = HashSet::default();
        while let Some(id) = work.pop() {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            order.push(id);
            match pattern.kind {
                Kind::Binding {
                    definition,
                    capture,
                    subpattern,
                    ..
                } => {
                    if matches!(capture, Capture::Borrow { .. }) {
                        borrowed.insert(definition);
                    }
                    if definitions.insert(definition) {
                        bindings.push(id);
                    }
                    work.extend(subpattern);
                }
                Kind::Dereference { pattern, .. } => work.push(pattern),
                Kind::Product { fields, .. } | Kind::Slice { fields, .. } => work.extend(
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
                Kind::Wildcard | Kind::Range(_) | Kind::Text(_) => {}
                Kind::Pending => {
                    return Err(self.error(pattern.source.node, "incomplete constructor pattern"));
                }
            }
        }
        let captures = self.captured_patterns(&order);
        let mut choices = HashMap::default();
        for id in order {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            if matches!(pattern.kind, Kind::Or(_)) && (retries || captures.contains(&id)) {
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
        let capture_choices = retries.then(|| {
            choices
                .keys()
                .filter(|id| captures.contains(id))
                .copied()
                .collect()
        });
        Ok(Plan {
            choices,
            bindings,
            borrowed,
            capture_choices,
        })
    }
    fn captured_patterns(&self, order: &[PatternId]) -> HashSet<PatternId> {
        let mut captures = HashSet::default();
        for &id in order.iter().rev() {
            let has_capture = match self.facts.patterns.nodes[id.0 as usize].kind {
                Kind::Binding { .. } => true,
                Kind::Dereference { pattern, .. } => captures.contains(&pattern),
                Kind::Product { fields, .. } | Kind::Slice { fields, .. } => self
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
        captures
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
        let kind = pattern.kind;
        if matches!(kind, Kind::Wildcard) {
            self.terminate(node, Terminator::Jump(yes));
            return Ok(());
        }
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
            Kind::Dereference { pattern, .. } => {
                let base = self.dereference_place(node, place)?;
                work.push(Decision::Test(Test {
                    pattern,
                    base,
                    selection: None,
                    yes,
                    no,
                }));
            }
            Kind::Range(_) | Kind::Text(_) => {
                let test = self.pattern_condition(id, place)?;
                self.terminate(
                    node,
                    test.map_or(Terminator::Jump(yes), |condition| Terminator::Branch {
                        condition,
                        yes,
                        no,
                    }),
                );
            }
            Kind::Product { fields, .. } | Kind::Slice { fields, .. } => {
                work.push(Decision::Fields {
                    fields,
                    base: place,
                    node,
                    yes,
                    no,
                });
                if let Some(condition) = self.pattern_condition(id, place)? {
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
        self.selected_captures(plan, root, place, None)
    }
    pub(super) fn selected_captures(
        &mut self,
        plan: &Plan,
        root: PatternId,
        place: Place,
        guard: Option<crate::body::ir::GuardId>,
    ) -> Result<(), Error> {
        if self.current.is_none() || plan.bindings.is_empty() {
            return Ok(());
        }
        let borrowed_views = self.declare_captures(plan, guard)?;
        let mut work = vec![Commit::Pattern(root, place, None)];
        while let Some(task) = work.pop() {
            match task {
                Commit::Switch(block) => {
                    self.switch(
                        self.body
                            .syntax_site(self.body.blocks[block.0 as usize].source)
                            .map_err(|message| self.report.body_error(self.body.source, message))?
                            .node,
                        block,
                    )?;
                }
                _ if self.current.is_none() => {}
                Commit::Jump(node, block) => self.terminate(node, Terminator::Jump(block)),
                Commit::Pattern(id, base, selection) => {
                    self.capture_pattern(plan, id, base, selection, &mut work)?;
                }
                Commit::Binding(id, place) => {
                    self.commit_capture(id, place, guard, &borrowed_views)?;
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
                                operands: crate::body::ir::BinaryOperands::Neither,
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
    fn commit_capture(
        &mut self,
        id: PatternId,
        place: Place,
        guard: Option<crate::body::ir::GuardId>,
        borrowed_views: &HashMap<crate::resolve::model::DefId, LocalId>,
    ) -> Result<(), Error> {
        let pattern = &self.facts.patterns.nodes[id.0 as usize];
        let node = pattern.source.node;
        let Kind::Binding {
            definition,
            capture,
            ..
        } = pattern.kind
        else {
            return Err(self.error(node, "missing constructor capture binding"));
        };
        let target = self.local_place(node, definition)?;
        let ty = self.facts.definitions[definition.0 as usize]
            .ok_or_else(|| self.error(node, "missing capture type"))?;
        if guard.is_some() {
            let Place::Local(view) = target else {
                return Err(self.error(node, "guard binding has no local"));
            };
            let place = if matches!(capture, Capture::Borrow { .. }) {
                let slot = *borrowed_views
                    .get(&definition)
                    .ok_or_else(|| self.error(node, "missing guarded reference storage"))?;
                let value = self.capture_value(id, place, ty)?;
                self.store(node, Place::Local(slot), value)?;
                Place::Local(slot)
            } else {
                place
            };
            self.statement(node, super::Action::BindView { view, place })?;
        } else {
            let value = self.capture_value(id, place, ty)?;
            self.store(node, target, value)?;
        }
        Ok(())
    }
    fn declare_captures(
        &mut self,
        plan: &Plan,
        guard: Option<crate::body::ir::GuardId>,
    ) -> Result<HashMap<crate::resolve::model::DefId, super::LocalId>, Error> {
        let mut borrowed = HashMap::default();
        if guard.is_some() {
            for &id in &plan.bindings {
                let pattern = &self.facts.patterns.nodes[id.0 as usize];
                if let Kind::Binding { definition, .. } = pattern.kind
                    && plan.borrowed.contains(&definition)
                {
                    let node = pattern.source.node;
                    let ty = self.facts.definitions[definition.0 as usize].ok_or_else(|| {
                        self.error(node, "missing guarded reference binding type")
                    })?;
                    let local = self.local(node, ty, LocalKind::Binding, None, false)?;
                    borrowed.insert(definition, local);
                }
            }
        }
        let first_view = self.index(self.source.node, self.body.locals.len())?;
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
            let ty = self.facts.definitions[definition.0 as usize]
                .ok_or_else(|| self.error(node, "missing capture type"))?;
            self.local(
                node,
                ty,
                guard.map_or(LocalKind::Binding, LocalKind::Guard),
                Some(definition),
                mutable,
            )?;
        }
        if let Some(guard) = guard {
            let end = self.index(self.source.node, self.body.locals.len())?;
            self.body.guards[guard.0 as usize].views_start = first_view;
            self.body.guards[guard.0 as usize].views_end = end;
        }
        Ok(borrowed)
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
        if matches!(kind, Kind::Wildcard) {
            return Ok(());
        }
        let place = self.selected_place(id, base, selection)?;
        match kind {
            Kind::Binding { subpattern, .. } => {
                work.push(Commit::Binding(id, place));
                if let Some(child) = subpattern {
                    work.push(Commit::Pattern(child, place, None));
                }
            }
            Kind::Dereference { pattern, .. } => {
                let place = self.dereference_place(node, place)?;
                work.push(Commit::Pattern(pattern, place, None));
            }
            Kind::Product { fields, .. } | Kind::Slice { fields, .. } => {
                for field in self.facts.patterns.fields(fields).iter().rev() {
                    work.push(Commit::Pattern(field.pattern, place, Some(field.selection)));
                }
            }
            Kind::Or(alternatives)
                if plan.choices.contains_key(&id)
                    && plan
                        .capture_choices
                        .as_ref()
                        .is_none_or(|choices| choices.contains(&id)) =>
            {
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
            Kind::Wildcard | Kind::Range(_) | Kind::Text(_) | Kind::Or(_) => {}
            Kind::Pending => return Err(self.error(node, "incomplete capture selection")),
        }
        Ok(())
    }
    pub(super) fn choice_value(
        &mut self,
        node: NodeId,
        value: u32,
    ) -> Result<super::ValueId, Error> {
        let scalar = ScalarId(self.index(node, self.body.scalars.len())?);
        self.body.scalars.push(Scalar::Integer(u128::from(value)));
        self.evaluate(
            node,
            TypeId::primitive(Primitive::U32),
            Rvalue::Scalar(scalar),
        )
    }
}
