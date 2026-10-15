//! Borrow and referent type facts remain distinct from later region/loan proofs.
use super::{Checker, Constructor, Error, Kind, NodeId, Operation, TermId, Work};
use crate::{
    sema::model::{AdjustmentKind, BodyRegionOrigin, ReferenceConversion},
    types::RegionId,
};

pub(super) enum CoercionWork {
    Enter(NodeId, TermId, bool, bool),
    Finish(NodeId, TermId),
}

impl Checker<'_> {
    pub(super) fn finish_reference_coercions(&mut self) -> Result<(), Error> {
        while let Some(work) = self.reference_work.pop() {
            self.reference_steps += 1;
            let node = match work {
                CoercionWork::Enter(node, ..) | CoercionWork::Finish(node, ..) => node,
            };
            if self.reference_steps > 1_000_000 {
                return Err(self.error(node, "reference coercion propagation budget exceeded"));
            }
            match work {
                CoercionWork::Enter(node, expected, identity_reborrow, contextual) => {
                    if contextual {
                        self.reference_expected.insert(node);
                    }
                    self.coerce_one(node, expected, identity_reborrow)?;
                }
                CoercionWork::Finish(node, expected) => {
                    // The result is supplied by the coerced branch/tail, so no
                    // read of an unadjusted mutable reference precedes its borrow.
                    self.save(node, expected);
                    self.adjustments.push(super::PendingAdjustment {
                        node,
                        actual: expected,
                        expected,
                        kind: AdjustmentKind::Equality,
                        reset: true,
                    });
                }
            }
        }
        Ok(())
    }
    fn propagate_reference(
        &mut self,
        node: NodeId,
        expected: TermId,
        identity_reborrow: bool,
    ) -> Result<bool, Error> {
        // A block typed without an expected reference has already consumed its
        // tail. A later join can coerce that result, but cannot undo its move.
        let contextual = self.reference_expected.contains(&node);
        if !contextual
            && !matches!(
                self.kind(node),
                Kind::Group(_) | Kind::Attributed { .. } | Kind::Label { .. }
            )
        {
            return Ok(false);
        }
        if !matches!(
            self.kind(node),
            Kind::Group(_)
                | Kind::Attributed { .. }
                | Kind::Label { .. }
                | Kind::Block { .. }
                | Kind::Expansion { .. }
                | Kind::If { .. }
                | Kind::Match { .. }
                | Kind::Loop { .. }
        ) {
            return Ok(false);
        }
        self.reference_work
            .push(CoercionWork::Finish(node, expected));
        let enter = |node| CoercionWork::Enter(node, expected, identity_reborrow, contextual);
        match self.kind(node) {
            Kind::Group(value) | Kind::Attributed { value, .. } | Kind::Label { value, .. } => {
                self.reference_work.push(enter(value));
            }
            Kind::Block { tail, .. } | Kind::Expansion { tail, .. } => {
                self.reference_work.extend(tail.map(enter));
            }
            Kind::If {
                then, otherwise, ..
            } => {
                self.reference_work.extend(otherwise.map(enter));
                self.reference_work.push(enter(then));
            }
            Kind::Match { .. } => {
                let id = *self
                    .facts
                    .patterns
                    .matches
                    .get(&self.key(node))
                    .ok_or_else(|| self.error(node, "missing reference match selection"))?;
                let decision = &self.facts.patterns.decisions[id as usize];
                for index in (decision.start..decision.end).rev() {
                    self.reference_work
                        .push(enter(self.facts.patterns.arms[index as usize].value.node));
                }
            }
            _ => {}
        }
        let mut site = self.break_roots.get(&node).copied();
        while let Some(index) = site {
            if let Some(value) = self.breaks[index].value {
                self.reference_work.push(enter(value));
            }
            site = self.breaks[index].next;
        }
        Ok(true)
    }
    pub(super) fn reference_coercion(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
        identity_reborrow: bool,
    ) -> Result<bool, Error> {
        let (converted, selection) = self.reference_conversion(node, actual, expected)?;
        let Some(selection) = selection else {
            return Ok(false);
        };
        if !identity_reborrow
            && selection.dereferences == 1
            && !selection.unsize
            && matches!(
                self.head(node, expected)?,
                Some(Constructor::Reference { mutable: true, .. })
            )
        {
            return Ok(false);
        }
        self.callable_shape(node, converted, expected)?;
        if self.propagate_reference(node, expected, identity_reborrow)? {
            return Ok(true);
        }
        let shared = matches!(
            self.head(node, actual)?,
            Some(Constructor::Reference { mutable: false, .. })
        );
        let kind = if shared && selection.dereferences == 1 {
            AdjustmentKind::ArrayToSlice
        } else {
            // An implicit borrow has its own inferred region. The selected
            // coercion separately relates it to the destination; a discarded
            // zero-repeat operand does not acquire that destination's lifetime.
            // The conversion retains the source reference's ancestry for the
            // contract proof. Unlike an explicit place borrow, it does not
            // introduce an unknown local-versus-promoted storage origin.
            let region = self.body_region(node, BodyRegionOrigin::Inference)?;
            let Some(Constructor::Reference { mutable, .. }) = self.head(node, expected)? else {
                return Err(self.error(node, "missing reference coercion target"));
            };
            let referent = self.child(node, expected, 0)?;
            let borrow = self
                .infer
                .construct(Constructor::Reference { mutable, region }, &[referent])
                .map_err(|e| self.error(node, e))?;
            self.reference_adjustments
                .insert(self.adjustments.len(), (selection, borrow));
            AdjustmentKind::Reborrow
        };
        self.record_adjustment(node, expected, kind)?;
        Ok(true)
    }
    pub(super) fn reference_join(
        &mut self,
        node: NodeId,
        a: TermId,
        b: TermId,
    ) -> Result<TermId, Error> {
        let ad = self.reference_depth(node, a)?;
        let bd = self.reference_depth(node, b)?;
        let a_mutable = matches!(
            self.head(node, a)?,
            Some(Constructor::Reference { mutable: true, .. })
        );
        let b_mutable = matches!(
            self.head(node, b)?,
            Some(Constructor::Reference { mutable: true, .. })
        );
        let (actual, expected) = if ad > bd || (ad == bd && a_mutable && !b_mutable) {
            (a, b)
        } else {
            (b, a)
        };
        let (converted, _) = self.reference_conversion(node, actual, expected)?;
        self.callable_shape(node, converted, expected)?;
        self.inferred_regions(node, expected)
    }
    fn reference_conversion(
        &mut self,
        node: NodeId,
        actual: TermId,
        expected: TermId,
    ) -> Result<(TermId, Option<ReferenceConversion>), Error> {
        let (
            Some(Constructor::Reference { mutable: from, .. }),
            Some(Constructor::Reference {
                mutable: to,
                region,
            }),
        ) = (self.head(node, actual)?, self.head(node, expected)?)
        else {
            return Err(self.error(node, "reference conversion requires reference types"));
        };
        if to && !from {
            return Err(self.error(
                node,
                "cannot coerce a shared reference to a mutable reference",
            ));
        }
        let target = self.child(node, expected, 0)?;
        let mut source = self.child(node, actual, 0)?;
        let mut dereferences = 1;
        // An unknown expected referent is inferred from the first candidate.
        // Known nesting chooses built-in dereferences without trial unification.
        if source != target && self.head(node, target)?.is_some() {
            let source_depth = self.reference_depth(node, source)?;
            let target_depth = self.reference_depth(node, target)?;
            for _ in target_depth..source_depth {
                source = self.child(node, source, 0)?;
                dereferences += 1;
            }
        }
        let unsize = matches!(self.head(node, source)?, Some(Constructor::Array(_)))
            && self.head(node, target)? == Some(Constructor::Slice);
        if !from && dereferences == 1 && !unsize {
            return Ok((actual, None));
        }
        let referent = if unsize {
            let element = self.child(node, source, 0)?;
            self.infer
                .construct(Constructor::Slice, &[element])
                .map_err(|e| self.error(node, e))?
        } else {
            source
        };
        let converted = self
            .infer
            .construct(
                Constructor::Reference {
                    mutable: to,
                    region,
                },
                &[referent],
            )
            .map_err(|e| self.error(node, e))?;
        Ok((
            converted,
            Some(ReferenceConversion {
                dereferences,
                unsize,
                two_phase: false,
            }),
        ))
    }
    fn reference_depth(&mut self, node: NodeId, mut term: TermId) -> Result<u32, Error> {
        let mut count = 0;
        while matches!(self.head(node, term)?, Some(Constructor::Reference { .. })) {
            count += 1;
            if count > 1_000_000 {
                return Err(self.error(node, "reference coercion work budget exceeded"));
            }
            term = self.child(node, term, 0)?;
        }
        Ok(count)
    }
    pub(super) fn pattern_input_validity(&self, mut node: NodeId) -> Result<bool, Error> {
        for _ in 0..1_000_000 {
            let key = self.key(node);
            if self.facts.autoderefs.contains_key(&key)
                || self.facts.node(key).and_then(|f| f.operation) == Some(Operation::Dereference)
            {
                return Ok(false);
            }
            match self.kind(node) {
                super::Kind::Group(value)
                | super::Kind::Attributed { value, .. }
                | super::Kind::Field { value, .. }
                | super::Kind::Index { value, .. } => node = value,
                // A value-producing expression initializes its result. Its
                // reads still require the independent move and loan proofs.
                _ => return Ok(true),
            }
        }
        Err(self.error(node, "pattern input place work budget exceeded"))
    }
    pub(super) fn enter_borrow(
        &mut self,
        node: NodeId,
        operand: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let referent = if let Some(expected) = expected
            && matches!(
                self.head(node, expected)?,
                Some(Constructor::Reference { .. })
            ) {
            Some(self.child(node, expected, 0)?)
        } else {
            None
        };
        self.work.push(Work::Enter(operand, referent));
        Ok(())
    }
    pub(super) fn borrow(
        &mut self,
        node: NodeId,
        operand: NodeId,
        mutable: bool,
    ) -> Result<TermId, Error> {
        let referent = self.term(operand)?;
        let region = if self.in_function {
            self.body_region(node, BodyRegionOrigin::ExpressionBorrow)?
        } else {
            // Constant inspection still checks promotion eligibility when demanded.
            RegionId::STATIC
        };
        let term = self
            .infer
            .construct(Constructor::Reference { mutable, region }, &[referent])
            .map_err(|e| self.error(node, e))?;
        self.selected(node, Operation::Borrow)?;
        self.alignment.borrows.push((node, operand));
        Ok(term)
    }
    pub(super) fn dereference(&mut self, node: NodeId, operand: NodeId) -> Result<TermId, Error> {
        let term = self.term(operand)?;
        if !matches!(self.head(node, term)?, Some(Constructor::Reference { .. })) {
            return Err(self.error(
                node,
                "dereferencing requires a known reference; raw pointers require unsafe semantics",
            ));
        }
        self.selected(node, Operation::Dereference)?;
        self.child(node, term, 0)
    }
    pub(super) fn autoderef(&mut self, node: NodeId, term: TermId) -> Result<TermId, Error> {
        let (term, count) = self
            .infer
            .reference_target(&self.facts.types, term)
            .map_err(|e| self.error(node, e))?;
        if count != 0 {
            self.facts.autoderefs.insert(self.key(node), count);
        }
        Ok(term)
    }
}
