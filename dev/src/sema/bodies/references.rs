//! Borrow and referent type facts remain distinct from later region/loan proofs.
use super::{Checker, Constructor, Error, NodeId, Operation, TermId, Work};
use crate::{sema::model::BodyRegionOrigin, types::RegionId};

impl Checker<'_> {
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
                Some(Constructor::Reference { mutable: false, .. })
            ) {
            Some(self.child(node, expected, 0)?)
        } else {
            None
        };
        self.work.push(Work::Enter(operand, referent));
        Ok(())
    }
    pub(super) fn borrow(&mut self, node: NodeId, operand: NodeId) -> Result<TermId, Error> {
        let referent = self.term(operand)?;
        let region = if self.in_function {
            self.body_region(node, BodyRegionOrigin::ExpressionBorrow)?
        } else {
            // Constant inspection still checks promotion eligibility when demanded.
            RegionId::STATIC
        };
        let term = self
            .infer
            .construct(
                Constructor::Reference {
                    mutable: false,
                    region,
                },
                &[referent],
            )
            .map_err(|e| self.error(node, e))?;
        self.selected(node, Operation::Borrow)?;
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
    pub(super) fn autoderef(&mut self, node: NodeId, mut term: TermId) -> Result<TermId, Error> {
        let mut count = 0;
        while matches!(self.head(node, term)?, Some(Constructor::Reference { .. })) {
            count += 1;
            if count > 1_000_000 {
                return Err(self.error(node, "automatic dereference work budget exceeded"));
            }
            term = self.child(node, term, 0)?;
        }
        if count != 0 {
            self.facts.autoderefs.insert(self.key(node), count);
        }
        Ok(term)
    }
}
