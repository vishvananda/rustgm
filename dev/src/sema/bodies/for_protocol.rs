//! Source loops instantiate the two real protocol methods through ordinary calls.
use super::{Checker, Constructor, Error, NodeId, Work};
use crate::{
    sema::{compiler_items::for_roles, model::BodyRegionOrigin},
    syntax::ast::Kind,
};

impl Checker<'_> {
    pub(super) fn source_for_iterator(&mut self, node: NodeId) -> Result<(), Error> {
        let Kind::For {
            pattern,
            iterator,
            body,
        } = self.kind(node)
        else {
            return Err(self.error(node, "invalid source iteration continuation"));
        };
        let protocol = for_roles::source(self.report, self.facts, self.key(node))?;
        let into_iter = self.protocol_call(node, protocol.into_iter)?;
        let input = self.child(node, into_iter.signature, 0)?;
        self.coerce(iterator, input)?;
        let iterator_type = self.child(node, into_iter.signature, 1)?;
        let next = self.protocol_call(node, protocol.next)?;
        let receiver = self.child(node, next.signature, 0)?;
        let region = self.body_region(node, BodyRegionOrigin::ExpressionBorrow)?;
        let reference = self
            .state
            .infer
            .construct(
                Constructor::Reference {
                    mutable: true,
                    region,
                },
                &[iterator_type],
            )
            .map_err(|error| self.error(node, error))?;
        self.shape_equation(node, receiver, reference)?;
        let option = self.child(node, next.signature, 1)?;
        if !matches!(self.head(node, option)?,Some(Constructor::Adt(owner, _) | Constructor::AdtArgs(owner, _)) if owner==protocol.option)
        {
            return Err(self.error(node, "source iteration next must return its actual Option"));
        }
        let item = self.child(node, option, 0)?;
        self.retain_try(super::protocol_control::OpenPlan {
            node,
            kind: super::protocol_control::OpenKind::Iteration {
                pattern,
                iterator,
                body,
                into_iter,
                next,
                iterator_type,
                item,
                option,
                some: protocol.some,
                none: protocol.none,
            },
        })?;
        self.state.work.push(Work::Enter(body, None));
        self.state.work.push(Work::ForPattern(pattern, item));
        Ok(())
    }
}
