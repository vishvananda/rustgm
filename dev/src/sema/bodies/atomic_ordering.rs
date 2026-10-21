//! The deny lint requires method-call syntax and a direct canonical variant expression.
use super::{Checker, Error, Kind, NodeId, Target};
use crate::support::{
    atomic_ordering::Ordering, intrinsics::Intrinsic, library_types::LibraryType,
};
impl Checker<'_> {
    pub(super) fn validate_atomic_orderings(&self) -> Result<(), Error> {
        let Some(ordering) = self
            .report
            .resolution
            .library_types
            .get(LibraryType::Ordering)
        else {
            return Ok(());
        };
        for (node, _) in &self.state.call_types.methods {
            let key = self.key(*node);
            if !self.facts.safety_policies.denies_atomic_ordering(key) {
                continue;
            }
            let Some(site) = self.report.resolution.method_calls.get(&key) else {
                continue;
            };
            let Some(call) = self.facts.methods.calls.get(&key) else {
                continue;
            };
            let Some(intrinsic) = self.facts.intrinsics.identify(call.method) else {
                continue;
            };
            if intrinsic.is_volatile() {
                continue;
            }
            let Some(&argument) = self
                .report
                .package
                .unit(key.file)
                .configured
                .ast
                .list(site.arguments)
                .last()
            else {
                continue;
            };
            let argument = self.ordering_expression(argument)?;
            if !matches!(self.kind(argument), Kind::Path { .. } | Kind::Name(_)) {
                continue;
            }
            let Some(Target::Definition(definition)) = self
                .facts
                .node(self.key(argument))
                .and_then(|fact| fact.reference)
            else {
                continue;
            };
            let Some(variant) = self.facts.members.constructor(definition) else {
                continue;
            };
            let variant = self.facts.members.variants[variant.0 as usize];
            if variant.owner.0 != ordering.definition.0 {
                continue;
            }
            let order = Ordering::from_tag(variant.ordinal)
                .ok_or_else(|| self.error(argument, "invalid canonical Ordering variant"))?;
            let valid = if intrinsic == Intrinsic::AtomicLoad {
                order.load_valid()
            } else {
                order.store_valid()
            };
            if !valid {
                return Err(self.error(argument, "invalid atomic ordering for this operation"));
            }
        }
        Ok(())
    }
    fn ordering_expression(&self, mut node: NodeId) -> Result<NodeId, Error> {
        for _ in 0..1_000_000 {
            match self.kind(node) {
                Kind::Group(value) | Kind::Attributed { value, .. } => node = value,
                _ => return Ok(node),
            }
        }
        Err(self.error(node, "atomic ordering expression work budget exceeded"))
    }
}
