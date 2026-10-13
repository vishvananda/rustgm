//! Constant-backed patterns defer only value/coverage work until typed queries finish.
use super::{Checker, Error, NodeId, matches::RangeWork};
use crate::{
    resolve::{
        model::{DefId, DefKind, Target},
        package::ModuleId,
        unit::NodeKey,
    },
    sema::patterns::PatternId,
    types::TypeId,
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
    roots: Vec<PatternId>,
    matches: Vec<u32>,
}
struct Scope {
    body: NodeKey,
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
        self.infer
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
    pub(super) fn pattern_constant_type(
        &self,
        node: NodeId,
        definition: DefId,
    ) -> Result<TypeId, Error> {
        self.facts.definitions[definition.0 as usize]
            .ok_or_else(|| self.error(node, "missing constant pattern type"))
    }
    pub(super) fn request_pattern_constant(
        &mut self,
        node: NodeId,
        definition: DefId,
        pattern: Option<PatternId>,
    ) {
        let key = self.key(node);
        self.pending_patterns
            .get_or_insert_with(Box::default)
            .requests
            .push((
                key,
                super::super::constants::PatternValue::Definition(definition),
                pattern,
            ));
        self.pattern_constants = true;
    }
    pub(super) fn request_pattern_literal(&mut self, node: NodeId, pattern: PatternId) {
        let key = self.key(node);
        self.pending_patterns
            .get_or_insert_with(Box::default)
            .requests
            .push((
                key,
                super::super::constants::PatternValue::Literal(key),
                Some(pattern),
            ));
        self.pattern_constants = true;
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
        let ty = self.pattern_constant_type(node, definition)?;
        if matches!(
            self.facts.types.kind(ty),
            Some(crate::types::Kind::Primitive(_))
        ) {
            return self.scalar_pattern(node, term);
        }
        let value = self.known(node, ty)?;
        self.equate_pattern(node, term, value)?;
        self.request_pattern_constant(node, definition, Some(id));
        self.pattern_refutable = true;
        Ok(crate::sema::patterns::Kind::Pending)
    }
    pub(super) fn defer_pattern_checks(&mut self) -> Result<(), Error> {
        let Some(pending) = &mut self.pending_patterns else {
            return Err(self.error(self.body.node, "missing constant pattern requests"));
        };
        pending.scopes.push(Scope {
            body: self.body,
            module: self.module,
            ranges: self.range_work.len(),
            roots: self.pattern_requirements.len(),
            matches: self.matches.len(),
        });
        pending.ranges.append(&mut self.range_work);
        pending.roots.append(&mut self.pattern_requirements);
        pending.matches.append(&mut self.matches);
        Ok(())
    }
    pub(super) fn complete_constant_patterns(&mut self) -> Result<(), Error> {
        let Some(pending) = self.pending_patterns.take() else {
            return Ok(());
        };
        super::super::constants::pattern_values(self.report, self.facts, &pending.requests)?;
        let mut ranges = pending.ranges.into_iter();
        let mut roots = pending.roots.into_iter();
        let mut matches = pending.matches.into_iter();
        for scope in pending.scopes {
            self.body = scope.body;
            self.file = scope.body.file;
            self.module = scope.module;
            self.range_work.extend(ranges.by_ref().take(scope.ranges));
            self.pattern_requirements
                .extend(roots.by_ref().take(scope.roots));
            self.matches.extend(matches.by_ref().take(scope.matches));
            self.finish_patterns()?;
        }
        Ok(())
    }
}
