//! Match arms share ordinary inference; scalar leaves retain ordered target domains.
use super::{Checker, Constructor, Error, Kind, List, NodeId, TermId, Work};
use crate::{
    sema::patterns::{
        self, Interval, Kind as Pattern, PatternId, RangeId,
        scalar::{Coverage, Domain},
    },
    syntax::ast::Unary,
    types::{Kind as Type, Primitive, TypeId},
};

#[derive(Clone, Copy)]
struct Bound {
    node: NodeId,
    negative: bool,
}
pub(super) struct RangeWork {
    id: RangeId,
    source: NodeId,
    start: Option<Bound>,
    end: Option<Bound>,
    inclusive: bool,
    range: bool,
}
impl Checker<'_> {
    pub(super) fn scalar_pattern(&mut self, node: NodeId, term: TermId) -> Result<Pattern, Error> {
        let (start, end, inclusive, range) = if let Kind::PatternRange {
            start,
            end,
            inclusive,
        } = self.kind(node)
        {
            (start, end, inclusive, true)
        } else {
            (Some(node), Some(node), true, false)
        };
        let start = start.map(|n| self.pattern_bound(n, term)).transpose()?;
        let end = if range {
            end.map(|n| self.pattern_bound(n, term)).transpose()?
        } else {
            start
        };
        let id = RangeId(
            u32::try_from(self.facts.patterns.ranges.len())
                .map_err(|_| self.error(node, "too many scalar patterns"))?,
        );
        self.facts
            .patterns
            .ranges
            .push(Interval { lower: 0, upper: 0 });
        self.state.range_work.push(RangeWork {
            id,
            source: node,
            start,
            end,
            inclusive,
            range,
        });
        Ok(Pattern::Range(id))
    }
    fn pattern_bound(&mut self, node: NodeId, term: TermId) -> Result<Bound, Error> {
        let (literal, negative) = if let Kind::Unary {
            op: Unary::Negate,
            operand,
        } = self.kind(node)
        {
            self.state.negated.insert(operand);
            self.state.checks.push(node);
            (operand, true)
        } else {
            (node, false)
        };
        let value = if let Kind::Literal(index) = self.kind(literal) {
            self.literal(literal, index)?
        } else if !negative && let Some(definition) = self.pattern_constant(literal) {
            let value = self.pattern_constant_term(literal, definition)?;
            self.request_pattern_constant(literal, definition, None);
            value
        } else {
            return Err(self.error(
                node,
                "pattern endpoints require literals or named constants",
            ));
        };
        self.equate(node, term, value)?;
        self.save(literal, value);
        self.save(node, value);
        Ok(Bound {
            node: literal,
            negative,
        })
    }
    pub(super) fn match_arms(
        &mut self,
        node: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let Kind::Match { value, arms } = self.kind(node) else {
            return Err(self.error(node, "missing match syntax"));
        };
        let term = self.term(value)?;
        let start = self.arm_index(node)?;
        let unit = self.report.package.unit(self.state.file);
        for &wrapper in unit.configured.ast.list(arms) {
            if !self.active(wrapper) {
                continue;
            }
            let mut arm = wrapper;
            while let Kind::Attributed { value, .. } = self.kind(arm) {
                arm = value;
            }
            let Kind::Arm { guard, value, .. } = self.kind(arm) else {
                return Err(self.error(arm, "missing match arm"));
            };
            self.facts.patterns.arms.push(patterns::Arm {
                source: self.key(arm),
                pattern: None,
                guard: guard.map(|n| self.key(n)),
                value: self.key(value),
                useful: false,
            });
        }
        let end = self.arm_index(node)?;
        let id = u32::try_from(self.facts.patterns.decisions.len())
            .map_err(|_| self.error(node, "too many matches"))?;
        self.facts.patterns.matches.insert(self.key(node), id);
        self.facts.patterns.decisions.push(patterns::Match {
            source: self.key(node),
            scrutinee: self.key(value),
            start,
            end,
            work: 0,
            uninhabited: false,
        });
        self.state.matches.push(id);
        self.state
            .work
            .extend((start..end).rev().map(|index| Work::Arm {
                index,
                term,
                expected,
            }));
        Ok(())
    }
    fn arm_index(&self, node: NodeId) -> Result<u32, Error> {
        u32::try_from(self.facts.patterns.arms.len())
            .map_err(|_| self.error(node, "too many match arms"))
    }
    pub(super) fn match_arm(
        &mut self,
        index: u32,
        term: TermId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let arm = self.facts.patterns.arms[index as usize];
        let Kind::Arm { pattern, .. } = self.kind(arm.source.node) else {
            return Err(self.error(arm.source.node, "missing arm pattern"));
        };
        let pattern = self.selected_pattern(pattern, term)?;
        self.facts.patterns.arms[index as usize].pattern = Some(pattern);
        self.state.work.push(Work::Exit(arm.source.node));
        self.state.work.push(Work::Enter(arm.value.node, expected));
        if let Some(guard) = arm.guard {
            let boolean = self.known(guard.node, TypeId::primitive(Primitive::Bool))?;
            self.state.work.push(Work::Enter(guard.node, Some(boolean)));
        }
        Ok(())
    }
    pub(super) fn match_result(
        &mut self,
        node: NodeId,
        value: NodeId,
        arms: List,
    ) -> Result<TermId, Error> {
        let mut common = self.known(node, TypeId::NEVER)?;
        let unit = self.report.package.unit(self.state.file);
        for &wrapper in unit.configured.ast.list(arms) {
            if !self.active(wrapper) {
                continue;
            }
            let mut arm = wrapper;
            while let Kind::Attributed { value, .. } = self.kind(arm) {
                arm = value;
            }
            common = self.join_types(node, common, self.term(arm)?)?;
        }
        let id = *self
            .facts
            .patterns
            .matches
            .get(&self.key(node))
            .ok_or_else(|| self.error(node, "missing match facts"))?;
        let decision = &self.facts.patterns.decisions[id as usize];
        let mut reference_hint = false;
        for index in decision.start..decision.end {
            let value = self.facts.patterns.arms[index as usize].value.node;
            let contextual = self.state.reference_expected.contains(&value);
            self.coerce_one(value, common, contextual || reference_hint)?;
            self.finish_reference_coercions()?;
            reference_hint |= matches!(
                self.head(value, self.term(value)?)?,
                Some(Constructor::Reference { .. })
            );
        }
        if self.head(node, self.term(value)?)? == Some(Constructor::Never) {
            self.known(node, TypeId::NEVER)
        } else {
            Ok(common)
        }
    }
    pub(super) fn finish_patterns(&mut self) -> Result<(), Error> {
        if std::mem::take(&mut self.state.pattern_constants) {
            return self.defer_pattern_checks();
        }
        // Solved node facts were published before scalar literal validation.
        for work in std::mem::take(&mut self.state.range_work) {
            let ty = work
                .start
                .or(work.end)
                .and_then(|b| self.facts.node(self.key(b.node)).and_then(|f| f.ty))
                .ok_or_else(|| self.error(work.source, "range pattern requires an endpoint"))?;
            let Some(Type::Primitive(primitive)) = self.facts.types.kind(ty) else {
                return Err(self.error(work.source, "pattern requires a scalar domain"));
            };
            let domain = Domain::new(primitive).ok_or_else(|| {
                self.error(
                    work.source,
                    "pattern domain requires later constructor analysis",
                )
            })?;
            if work.range && primitive == Primitive::Bool {
                return Err(self.error(work.source, "boolean ranges are not valid patterns"));
            }
            let lower = work
                .start
                .map(|b| self.bound_rank(domain, b))
                .transpose()?
                .unwrap_or(0);
            let upper = work
                .end
                .map(|b| self.bound_rank(domain, b))
                .transpose()?
                .unwrap_or_else(|| domain.ordered().upper);
            let upper = if work.end.is_some() && !work.inclusive {
                upper
                    .checked_sub(1)
                    .ok_or_else(|| self.error(work.source, "empty scalar pattern range"))?
            } else {
                upper
            };
            if lower > upper {
                return Err(self.error(work.source, "empty scalar pattern range"));
            }
            self.facts.patterns.ranges[work.id.0 as usize] = Interval { lower, upper };
        }
        for (root, valid) in std::mem::take(&mut self.state.pattern_requirements) {
            self.require_irrefutable(root, valid)?;
        }
        for id in std::mem::take(&mut self.state.matches) {
            self.check_match(id)?;
        }
        Ok(())
    }
    fn bound_rank(&self, domain: Domain, bound: Bound) -> Result<u128, Error> {
        let value = self
            .facts
            .node(self.key(bound.node))
            .and_then(|f| f.scalar)
            .ok_or_else(|| self.error(bound.node, "missing scalar pattern value"))?;
        domain
            .rank(value, bound.negative)
            .map_err(|e| self.error(bound.node, e))
    }
    fn pattern_domain(&self, id: PatternId) -> Option<Domain> {
        let ty = self.facts.patterns.nodes[id.0 as usize].ty?;
        let Type::Primitive(primitive) = self.facts.types.kind(ty)? else {
            return None;
        };
        Domain::new(primitive)
    }
    fn pattern_intervals(
        &self,
        root: PatternId,
        domain: Domain,
        coverage: &mut Coverage,
        intervals: &mut Vec<Interval>,
    ) -> Result<(), Error> {
        let mut work = vec![root];
        while let Some(id) = work.pop() {
            let pattern = &self.facts.patterns.nodes[id.0 as usize];
            coverage
                .charge()
                .map_err(|e| self.error(pattern.source.node, e))?;
            match pattern.kind {
                Pattern::Wildcard
                | Pattern::Binding {
                    subpattern: None, ..
                } => intervals.push(domain.full()),
                Pattern::Binding {
                    subpattern: Some(child),
                    ..
                } => work.push(child),
                Pattern::Range(range) => {
                    intervals.push(self.facts.patterns.ranges[range.0 as usize]);
                }
                Pattern::Or(alternatives) => work.extend(
                    self.facts
                        .patterns
                        .alternatives(alternatives)
                        .iter()
                        .rev()
                        .copied(),
                ),
                _ => {
                    return Err(self.error(
                        pattern.source.node,
                        "pattern coverage requires constructor specialization",
                    ));
                }
            }
        }
        Ok(())
    }
    fn check_match(&mut self, id: u32) -> Result<(), Error> {
        let decision = &self.facts.patterns.decisions[id as usize];
        let (node, start, end) = (decision.source.node, decision.start, decision.end);
        let ty = self
            .facts
            .node(decision.scrutinee)
            .and_then(|f| f.ty)
            .ok_or_else(|| self.error(node, "missing match input type"))?;
        if ty == TypeId::NEVER {
            self.facts.patterns.decisions[id as usize].work = 1;
            self.facts.patterns.decisions[id as usize].uninhabited = true;
            return Ok(());
        }
        let domain = if let Some(Type::Primitive(primitive)) = self.facts.types.kind(ty) {
            Domain::new(primitive)
        } else {
            None
        };
        let Some(domain) = domain else {
            return self.constructor_match(id, ty);
        };
        let mut coverage = Coverage::default();
        let mut intervals = Vec::new();
        for index in start..end {
            let arm = self.facts.patterns.arms[index as usize];
            let root = arm
                .pattern
                .ok_or_else(|| self.error(node, "missing arm selection"))?;
            intervals.clear();
            self.pattern_intervals(root, domain, &mut coverage, &mut intervals)?;
            let mut useful = false;
            for &interval in &intervals {
                useful |= coverage.useful(interval).map_err(|e| self.error(node, e))?;
            }
            self.facts.patterns.arms[index as usize].useful = useful;
            if arm.guard.is_none() {
                for &interval in &intervals {
                    coverage.insert(interval).map_err(|e| self.error(node, e))?;
                }
            }
        }
        self.facts.patterns.decisions[id as usize].work = coverage.work();
        if let Some(missing) = coverage.missing(domain) {
            let value = domain
                .witness(missing.lower)
                .map_err(|e| self.error(node, e))?;
            return Err(self.error(node, format!("non-exhaustive match; missing {value}")));
        }
        Ok(())
    }
    fn require_irrefutable(&mut self, root: PatternId, valid: bool) -> Result<(), Error> {
        if self.pattern_domain(root).is_none() {
            return self.constructor_binding(root, valid);
        }
        let node = self.facts.patterns.nodes[root.0 as usize].source.node;
        let domain = self
            .pattern_domain(root)
            .ok_or_else(|| self.error(node, "missing binding domain"))?;
        let mut coverage = Coverage::default();
        let mut intervals = Vec::new();
        self.pattern_intervals(root, domain, &mut coverage, &mut intervals)?;
        for interval in intervals {
            coverage.insert(interval).map_err(|e| self.error(node, e))?;
        }
        if let Some(missing) = coverage.missing(domain) {
            return Err(self.error(
                node,
                format!(
                    "refutable binding; missing {}",
                    domain
                        .witness(missing.lower)
                        .map_err(|e| self.error(node, e))?
                ),
            ));
        }
        Ok(())
    }
    fn constructor_match(&mut self, id: u32, ty: TypeId) -> Result<(), Error> {
        let decision = &self.facts.patterns.decisions[id as usize];
        let (node, start, end) = (decision.source.node, decision.start, decision.end);
        let valid = self.pattern_input_validity(decision.scrutinee.node)?;
        let (report, source) = (self.report, self.key(node));
        let error =
            |message: &str| crate::sema::model::Declarations::error(report, source, message);
        let mut coverage = patterns::coverage::Coverage::new(
            self.facts,
            report,
            self.state.module,
            ty,
            valid,
            patterns::coverage::Roots::Arms(
                &self.facts.patterns.arms[start as usize..end as usize],
            ),
            &mut self.state.coverage_cache,
        )
        .map_err(error)?;
        let mut usefulness = Vec::new();
        for index in start..end {
            let arm = self.facts.patterns.arms[index as usize];
            let pattern = arm.pattern.ok_or_else(|| error("missing arm pattern"))?;
            usefulness.push(coverage.arm(pattern, arm.guard.is_some()).map_err(error)?);
        }
        let missing = coverage.missing().map_err(error)?;
        let work = coverage.work();
        let uninhabited = coverage.uninhabited();
        if let Some(missing) = missing {
            return Err(self.error(node, format!("non-exhaustive match; missing {missing}")));
        }
        for (index, useful) in (start..end).zip(usefulness) {
            self.facts.patterns.arms[index as usize].useful = useful;
        }
        self.facts.patterns.decisions[id as usize].work = work;
        self.facts.patterns.decisions[id as usize].uninhabited = uninhabited;
        Ok(())
    }
    fn constructor_binding(&mut self, root: PatternId, valid: bool) -> Result<(), Error> {
        let pattern = &self.facts.patterns.nodes[root.0 as usize];
        let node = pattern.source.node;
        let ty = pattern
            .ty
            .ok_or_else(|| self.error(node, "missing pattern input type"))?;
        let (report, source) = (self.report, self.key(node));
        let error =
            |message: &str| crate::sema::model::Declarations::error(report, source, message);
        let mut coverage = patterns::coverage::Coverage::new(
            self.facts,
            report,
            self.state.module,
            ty,
            valid,
            patterns::coverage::Roots::Pattern(root),
            &mut self.state.coverage_cache,
        )
        .map_err(error)?;
        coverage.arm(root, false).map_err(error)?;
        if let Some(missing) = coverage.missing().map_err(error)? {
            return Err(self.error(node, format!("refutable binding; missing {missing}")));
        }
        Ok(())
    }
}
