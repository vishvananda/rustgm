//! Constant-backed patterns defer only value/coverage work until typed queries finish.
mod ordering;
use super::{Checker, Error, NodeId, matches::RangeWork};
use crate::{
    resolve::{
        model::{DefId, DefKind, Target},
        package::ModuleId,
        unit::NodeKey,
    },
    sema::patterns::PatternId,
};

#[derive(Default)]
pub(super) struct Pending {
    requests: Vec<(
        NodeKey,
        super::super::constants::PatternValue,
        Option<PatternId>,
    )>,
    scopes: Vec<Scope>,
    ranges: Vec<RangeWork>,
    roots: Vec<(PatternId, bool)>,
    matches: Vec<u32>,
}
impl Pending {
    /// Completed function batches are joined once in declaration order. Active
    /// callers keep their unfinished request suffix inside their suspended state.
    pub(super) fn append(&mut self, mut other: Self) {
        let offset = self.requests.len();
        for scope in &mut other.scopes {
            scope.values.end += offset;
        }
        self.requests.append(&mut other.requests);
        self.scopes.append(&mut other.scopes);
        self.ranges.append(&mut other.ranges);
        self.roots.append(&mut other.roots);
        self.matches.append(&mut other.matches);
    }
}
struct Scope {
    values: super::super::constants::PatternScope,
    module: ModuleId,
    ranges: usize,
    roots: usize,
    matches: usize,
}
impl Checker<'_> {
    pub(super) fn equate_pattern(
        &mut self,
        node: NodeId,
        left: crate::types::infer::TermId,
        right: crate::types::infer::TermId,
    ) -> Result<(), Error> {
        self.state
            .infer
            .equate_pattern(&self.facts.types, left, right)
            .map_err(|e| self.error(node, e))
    }
    pub(super) fn pattern_constant(&self, node: NodeId) -> Option<DefId> {
        let Target::Definition(definition) = self.facts.node(self.key(node))?.reference? else {
            return None;
        };
        (self.report.resolution.definitions[definition.0 as usize].kind == DefKind::Const)
            .then_some(definition)
    }
    pub(super) fn pattern_constant_term(
        &mut self,
        node: NodeId,
        definition: DefId,
    ) -> Result<crate::types::infer::TermId, Error> {
        let ty = self.facts.definitions[definition.0 as usize]
            .ok_or_else(|| self.error(node, "missing constant pattern type"))?;
        if self.facts.associated.owner(definition).is_some() {
            self.associated_constant(node, definition, ty)
        } else {
            self.known(node, ty)
        }
    }
    pub(super) fn request_pattern_constant(
        &mut self,
        node: NodeId,
        definition: DefId,
        pattern: Option<PatternId>,
    ) {
        let key = self.key(node);
        self.state
            .pending_patterns
            .get_or_insert_with(Box::default)
            .requests
            .push((
                key,
                super::super::constants::PatternValue::Definition(definition),
                pattern,
            ));
        self.state.pattern_constants = true;
    }
    pub(super) fn request_pattern_literal(&mut self, node: NodeId, pattern: PatternId) {
        let key = self.key(node);
        self.state
            .pending_patterns
            .get_or_insert_with(Box::default)
            .requests
            .push((
                key,
                super::super::constants::PatternValue::Literal(key),
                Some(pattern),
            ));
        self.state.pattern_constants = true;
    }
    pub(super) fn constant_pattern(
        &mut self,
        node: NodeId,
        term: crate::types::infer::TermId,
        id: PatternId,
    ) -> Result<crate::sema::patterns::Kind, Error> {
        let definition = self
            .pattern_constant(node)
            .ok_or_else(|| self.error(node, "missing constant pattern identity"))?;
        let ty = self.facts.definitions[definition.0 as usize];
        if ty.is_some_and(|ty| {
            matches!(
                self.facts.types.kind(ty),
                Some(crate::types::Kind::Primitive(_))
            )
        }) {
            return self.scalar_pattern(node, term);
        }
        let value = self.pattern_constant_term(node, definition)?;
        self.equate_pattern(node, term, value)?;
        self.save(node, value);
        self.request_pattern_constant(node, definition, Some(id));
        self.state.pattern_refutable = true;
        Ok(crate::sema::patterns::Kind::Pending)
    }
    pub(super) fn defer_pattern_checks(&mut self) -> Result<(), Error> {
        let Some(pending) = &mut self.state.pending_patterns else {
            return Err(self.error(self.state.body.node, "missing constant pattern requests"));
        };
        self.facts.patterns.pending_bodies.insert(self.state.body);
        pending.scopes.push(Scope {
            values: super::super::constants::PatternScope {
                body: self.state.body,
                owner: self.state.environment_owner,
                binders: self.state.body_binders,
                end: pending.requests.len(),
            },
            module: self.state.module,
            ranges: self.state.range_work.len(),
            roots: self.state.pattern_requirements.len(),
            matches: self.state.matches.len(),
        });
        pending.ranges.append(&mut self.state.range_work);
        pending.roots.append(&mut self.state.pattern_requirements);
        pending.matches.append(&mut self.state.matches);
        Ok(())
    }
    /// Finish only this completed definition's patterns before its CFG is demanded.
    /// The waiting caller keeps all inference and unfinished pattern state intact.
    pub(super) fn prepare_defining_patterns(&mut self, pending: Box<Pending>) -> Result<(), Error> {
        let mut scratch = super::Scratch::default();
        scratch.resumable();
        let fresh = Checker::with_scratch(self.report, self.facts, scratch).state;
        let saved = std::mem::replace(&mut self.state, fresh);
        self.state.pending_patterns = Some(pending);
        let result = self.complete_constant_patterns();
        self.state = saved;
        result
    }
    pub(in crate::sema) fn complete_constant_patterns(&mut self) -> Result<(), Error> {
        let Some(pending) = self.state.pending_patterns.take() else {
            return Ok(());
        };
        let mut scopes = pending.into_scopes();
        let order = ordering::schedule(self.facts, &scopes)
            .map_err(|message| self.error(self.state.body.node, message))?;
        for index in order {
            let prepared = scopes[index].take().ok_or_else(|| {
                self.error(
                    self.state.body.node,
                    "constant pattern scope scheduled twice",
                )
            })?;
            self.complete_pattern_scope(prepared)?;
        }
        Ok(())
    }
    fn complete_pattern_scope(&mut self, prepared: PreparedScope) -> Result<(), Error> {
        let PreparedScope {
            scope,
            requests,
            ranges,
            roots,
            matches,
        } = prepared;
        super::super::constants::pattern_values(
            self.report,
            self.facts,
            &requests,
            std::iter::once(scope.values),
            &mut self.pattern_queries,
        )?;
        self.state.body = scope.values.body;
        self.state.file = scope.values.body.file;
        self.state.module = scope.module;
        self.state.range_work.extend(ranges);
        self.state.pattern_requirements.extend(roots);
        self.state.matches.extend(matches);
        self.finish_patterns()?;
        self.facts
            .patterns
            .pending_bodies
            .remove(&scope.values.body);
        Ok(())
    }
}

struct PreparedScope {
    scope: Scope,
    requests: Vec<(
        NodeKey,
        super::super::constants::PatternValue,
        Option<PatternId>,
    )>,
    ranges: Vec<RangeWork>,
    roots: Vec<(PatternId, bool)>,
    matches: Vec<u32>,
}
impl Pending {
    fn into_scopes(self) -> Vec<Option<PreparedScope>> {
        let mut requests = self.requests.into_iter();
        let mut ranges = self.ranges.into_iter();
        let mut roots = self.roots.into_iter();
        let mut matches = self.matches.into_iter();
        let mut start = 0;
        self.scopes
            .into_iter()
            .map(|mut scope| {
                let end = scope.values.end;
                scope.values.end -= start;
                start = end;
                Some(PreparedScope {
                    requests: requests.by_ref().take(scope.values.end).collect(),
                    ranges: ranges.by_ref().take(scope.ranges).collect(),
                    roots: roots.by_ref().take(scope.roots).collect(),
                    matches: matches.by_ref().take(scope.matches).collect(),
                    scope,
                })
            })
            .collect()
    }
}
