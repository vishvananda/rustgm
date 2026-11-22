//! Actual selected places retain their own field instances across reference conversions.
use super::{Checker, Error, NodeId, TypeId};
use crate::{
    sema::patterns::{Kind as Pattern, PatternId},
    support::hash::{HashMap, HashSet},
    syntax::ast::{Kind, Unary},
    types::{Kind as Type, infer::Solved},
};

struct Inputs {
    expected: HashMap<PatternId, TypeId>,
    pending: Vec<(PatternId, TypeId)>,
}

impl Checker<'_> {
    pub(super) fn projected_storage(&mut self, solved: &Solved) -> Result<Vec<TypeId>, Error> {
        let mut roots = Vec::new();
        if self.facts.source_bindings.is_none() {
            return Ok(roots);
        }
        let mut work = 0;
        let result = (|| {
            // The source of a dereference may have a different lifetime instance
            // from its contextual result. CFG projection reads the former place.
            for &node in &self.state.touched {
                self.storage_visit(&mut work, 1)?;
                match self.kind(node) {
                    Kind::Unary {
                        op: Unary::Dereference,
                        operand,
                    } => {
                        let actual = solved.types[self.term(operand)?.0 as usize];
                        if let Some(referent) = self.storage_referent(node, actual)? {
                            roots.push(referent);
                        }
                    }
                    Kind::Closure { .. } => {
                        self.capture_formation_storage(node, &mut roots, &mut work)?;
                    }
                    _ => {}
                }
            }
            self.pattern_storage(solved, &mut roots, &mut work)
        })();
        self.facts.body_work.canonical_steps =
            self.facts.body_work.canonical_steps.saturating_add(work);
        result?;
        Ok(roots)
    }
    // Closure construction projects selected captures in the parent's lifetime
    // context. The closure environment may store only a reference to that place.
    fn capture_formation_storage(
        &self,
        node: NodeId,
        roots: &mut Vec<TypeId>,
        work: &mut usize,
    ) -> Result<(), Error> {
        let Some(environment) = self
            .facts
            .closures
            .get(self.key(node))
            .and_then(|closure| self.facts.closures.environment(closure.identity))
        else {
            return Ok(());
        };
        let mut seen = HashSet::default();
        for mut path in environment
            .plan
            .captures()
            .iter()
            .map(|capture| capture.path)
            .chain(environment.mentions.iter().copied())
        {
            loop {
                self.storage_visit(work, 1)?;
                if !seen.insert(path) {
                    break;
                }
                let Some((parent, _)) = environment.plan.parent(path) else {
                    break;
                };
                roots.push(environment.formation_types[path.index()]);
                path = parent;
            }
        }
        Ok(())
    }

    /// Local bodies reuse the header producer's input-graph bound. A staged
    /// consumer has no local header producer: its installed source-header owner
    /// pays these queries through the existing selected-work account instead.
    /// Each root, pair and edge pays before traversal, including duplicates and
    /// failed attempts. Neither path creates a ledger or borrows another payer.
    pub(super) fn storage_visit(&self, work: &mut usize, count: usize) -> Result<(), Error> {
        *work = work.saturating_add(count);
        if self.report.package.staged_consumer().is_some() {
            crate::sema::crates::charge_source_header_query(self.facts, count)
        } else {
            crate::sema::header_attributes::charge_source_contract_query(self.facts, count)
        }
    }
    pub(super) fn storage_referent(
        &self,
        node: NodeId,
        ty: TypeId,
    ) -> Result<Option<TypeId>, Error> {
        match self.facts.types.kind(ty) {
            Some(Type::Reference { referent, .. }) => Ok(Some(referent)),
            Some(Type::Pointer { pointee, .. }) => Ok(Some(pointee)),
            _ => crate::sema::compiler_items::boxes::content(
                &self.facts.types,
                &self.facts.aggregates,
                ty,
            )
            .map_err(|error| self.error(node, error)),
        }
    }
    fn pattern_inputs(&self, solved: &Solved, work: &mut usize) -> Result<Inputs, Error> {
        self.storage_visit(work, self.state.pattern_types.len())?;
        let expected = self
            .state
            .pattern_types
            .iter()
            .map(|&(id, term)| (id, solved.types[term.0 as usize]))
            .collect::<HashMap<_, _>>();
        let mut children = HashSet::default();
        for &(id, _) in &self.state.pattern_types {
            self.storage_visit(work, 1)?;
            pattern_children(&self.facts.patterns, id, &mut children, |count| {
                self.storage_visit(work, count)
            })?;
        }
        // Discovery order fixes canonical type publication independently of
        // the temporary lookup tables' bucket order.
        self.storage_visit(work, self.state.pattern_types.len())?;
        let mut pending = self
            .state
            .pattern_types
            .iter()
            .filter(|(id, _)| !children.contains(id))
            .map(|&(id, term)| (id, solved.types[term.0 as usize]))
            .collect::<Vec<_>>();
        // Parameter places use the parameter node's published type, just as
        // CFG entry construction does, rather than its pattern's expected type.
        for &node in &self.state.touched {
            self.storage_visit(work, 1)?;
            if let Kind::Parameter { pattern, .. } = self.kind(node)
                && let Some(&id) = self.facts.patterns.roots.get(&self.key(pattern))
            {
                self.storage_visit(work, 1)?;
                pending.push((id, solved.types[self.term(node)?.0 as usize]));
            }
        }
        for &(id, input) in &self.state.alignment.patterns {
            self.storage_visit(work, 1)?;
            if let Some(input) = input {
                self.storage_visit(work, 1)?;
                pending.push((id, solved.types[self.term(input)?.0 as usize]));
            }
        }
        for &id in &self.state.matches {
            self.storage_visit(work, 1)?;
            let decision = &self.facts.patterns.decisions[id as usize];
            let actual = solved.types[self.term(decision.scrutinee.node)?.0 as usize];
            let arms = &self.facts.patterns.arms[decision.start as usize..decision.end as usize];
            self.storage_visit(work, arms.len())?;
            pending.extend(
                arms.iter()
                    .filter_map(|arm| arm.pattern.map(|pattern| (pattern, actual))),
            );
        }
        Ok(Inputs { expected, pending })
    }
    fn pattern_storage(
        &self,
        solved: &Solved,
        roots: &mut Vec<TypeId>,
        work: &mut usize,
    ) -> Result<(), Error> {
        if self.state.pattern_types.is_empty() {
            return Ok(());
        }
        let Inputs {
            expected,
            mut pending,
        } = self.pattern_inputs(solved, work)?;
        let selected = |id: PatternId| {
            expected
                .get(&id)
                .copied()
                .or_else(|| self.facts.patterns.nodes.get(id.0 as usize)?.ty)
                .ok_or_else(|| {
                    self.error(
                        self.state.body.node,
                        "selected pattern has no solved storage type",
                    )
                })
        };
        let mut seen = HashSet::default();
        // Pairs preserve distinct physical lifetime contexts. Sharing can queue
        // a pair from several edges, but only its first visit traverses children;
        // both duplicate visits and each distinct context remain paid work.
        while let Some((id, actual)) = pending.pop() {
            self.storage_visit(work, 1)?;
            if !seen.insert((id, actual)) {
                continue;
            }
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            match pattern.kind {
                Pattern::Dereference { pattern: child, .. } => {
                    self.storage_visit(work, 1)?;
                    let referent = self
                        .storage_referent(pattern.source.node, actual)?
                        .ok_or_else(|| {
                            self.error(
                                pattern.source.node,
                                "selected pattern storage requires a pointer place",
                            )
                        })?;
                    roots.push(referent);
                    pending.push((child, referent));
                }
                Pattern::Binding { subpattern, .. } => {
                    self.storage_visit(work, usize::from(subpattern.is_some()))?;
                    pending.extend(subpattern.map(|child| (child, actual)));
                }
                Pattern::Or(alternatives) => {
                    self.storage_visit(work, (alternatives.end - alternatives.start) as usize)?;
                    pending.extend(
                        self.facts
                            .patterns
                            .alternatives(alternatives)
                            .iter()
                            .map(|&child| (child, actual)),
                    );
                }
                Pattern::Product { fields, .. } | Pattern::Slice { fields, .. } => {
                    self.storage_visit(work, (fields.end - fields.start) as usize)?;
                    for field in self.facts.patterns.fields(fields) {
                        // A field projection has the selected child's type;
                        // unlike dereference it does not retain its base type.
                        pending.push((field.pattern, selected(field.pattern)?));
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }
}
fn pattern_children(
    patterns: &crate::sema::patterns::Patterns,
    id: PatternId,
    children: &mut HashSet<PatternId>,
    pay: impl FnOnce(usize) -> Result<(), Error>,
) -> Result<(), Error> {
    match patterns.nodes[id.0 as usize].kind {
        Pattern::Dereference { pattern, .. } => {
            pay(1)?;
            children.insert(pattern);
        }
        Pattern::Binding { subpattern, .. } => {
            pay(usize::from(subpattern.is_some()))?;
            children.extend(subpattern);
        }
        Pattern::Product { fields, .. } | Pattern::Slice { fields, .. } => {
            pay((fields.end - fields.start) as usize)?;
            children.extend(patterns.fields(fields).iter().map(|field| field.pattern));
        }
        Pattern::Or(alternatives) => {
            pay((alternatives.end - alternatives.start) as usize)?;
            children.extend(patterns.alternatives(alternatives));
        }
        _ => {}
    }
    Ok(())
}
