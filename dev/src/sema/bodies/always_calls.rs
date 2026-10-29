//! Source `FnItem` policy is checked before legitimate fn-pointer erasure.
use super::{Checker, DefId, Error, NodeId};
use crate::{
    sema::header_attributes::ContractKnowledge,
    support::constness::Constness,
    types::{Kind, TypeId},
};
impl Checker<'_> {
    pub(super) fn check_comptime_callable(
        &mut self,
        node: NodeId,
        mut callable: TypeId,
    ) -> Result<(), Error> {
        if !self.facts.comptime_calls_need_checking() {
            return Ok(());
        }
        let definition = loop {
            self.state
                .infer
                .account_work(1)
                .map_err(|error| self.error(node, error))?;
            match self.facts.types.kind(callable) {
                Some(Kind::Reference { referent, .. }) => callable = referent,
                Some(Kind::FunctionItem { item, .. }) => break DefId(item.0),
                // An actual FnPtr source call retains ordinary runtime semantics,
                // even if later lowering can recover its concrete function body.
                _ => return Ok(()),
            }
        };
        let declaration = self
            .report
            .resolution
            .definitions
            .get(definition.0 as usize)
            .ok_or_else(|| self.error(node, "comptime call lost its actual target"))?;
        match declaration.kind {
            crate::resolve::model::DefKind::Function => {}
            crate::resolve::model::DefKind::Constructor
            | crate::resolve::model::DefKind::Variant => return Ok(()),
            _ => {
                return Err(self.error(
                    node,
                    "comptime policy requires an actual callable declaration",
                ));
            }
        }
        match self
            .facts
            .source_comptime_contract(self.report, definition)?
        {
            ContractKnowledge::KnownAbsent => return Ok(()),
            ContractKnowledge::KnownPresent(_) => {}
            ContractKnowledge::Unknown => {
                return Err(self.error(node, "comptime call lost its actual source contract"));
            }
        }
        if self.callee_use_effect()? != Constness::Const {
            return Err(self.error(
                node,
                "comptime functions can only be called at compile time",
            ));
        }
        Ok(())
    }
}
