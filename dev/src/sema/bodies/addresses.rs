//! Raw address syntax selects a place without introducing a reference lifetime.
use super::{Checker, Constructor, Error, Kind, NodeId, Operation, TermId};
use crate::syntax::ast::Unary;

impl Checker<'_> {
    fn address_operand(&self, mut node: NodeId) -> Result<NodeId, Error> {
        for _ in 0..1_000_000 {
            match self.kind(node) {
                Kind::Group(value) | Kind::Attributed { value, .. } => node = value,
                _ => return Ok(node),
            }
        }
        Err(self.error(node, "raw address operand work budget exceeded"))
    }
    pub(super) fn enter_address(
        &mut self,
        node: NodeId,
        operand: NodeId,
        expected: Option<TermId>,
    ) -> Result<(), Error> {
        let direct = self.address_operand(operand)?;
        self.state.raw_address_places.insert(direct);
        if matches!(
            self.kind(direct),
            Kind::Unary {
                op: Unary::Dereference,
                ..
            }
        ) {
            self.state.address_dereferences.insert(direct);
        }
        self.enter_borrow(node, operand, expected)
    }
    pub(super) fn address(
        &mut self,
        node: NodeId,
        operand: NodeId,
        mutable: bool,
    ) -> Result<TermId, Error> {
        let direct = self.address_operand(operand)?;
        let operation = if self.facts.node(self.key(direct)).and_then(|f| f.operation)
            == Some(Operation::AddressDereference)
        {
            let Kind::Unary {
                operand: pointer, ..
            } = self.kind(direct)
            else {
                return Err(self.error(node, "missing raw address dereference"));
            };
            if mutable
                && self.head(node, self.term(pointer)?)?
                    != Some(Constructor::Pointer { mutable: true })
            {
                return Err(self.error(node, "raw address cannot strengthen pointer mutability"));
            }
            Operation::PointerAddress { operand: pointer }
        } else {
            Operation::Address
        };
        let referent = self.term(operand)?;
        let ty = self
            .state
            .infer
            .construct(Constructor::Pointer { mutable }, &[referent])
            .map_err(|e| self.error(node, e))?;
        self.selected(node, operation)?;
        Ok(ty)
    }
}
