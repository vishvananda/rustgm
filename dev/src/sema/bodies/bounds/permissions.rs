//! Body builtin permissions consume declared and implied lifetime assumptions.
use super::{Checker, Error, NodeId, TypeId};

impl Checker<'_> {
    pub(super) fn defer_builtin_identity(
        &mut self,
        node: NodeId,
        ty: TypeId,
        bound: crate::types::parameters::predicates::Bound,
    ) -> Result<bool, Error> {
        let Some(environment) = self
            .state
            .method_environment
            .as_ref()
            .filter(|env| env.has_region_builtins())
        else {
            return Ok(false);
        };
        if self.facts.types.contains_regions(ty) != Some(true) {
            return Ok(false);
        }
        let source = self.key(node);
        let pending = self
            .facts
            .generics
            .builtin_requirements
            .entry(self.state.body)
            .or_insert_with(|| crate::sema::model::BuiltinRequirements {
                environment: Some(std::sync::Arc::clone(environment)),
                predicates: Vec::new(),
                source_copy: None,
            });
        if pending.predicates.len() >= 1_000_000 {
            return Err(self.error(node, "builtin requirement storage budget exceeded"));
        }
        pending.predicates.push((
            source,
            crate::types::parameters::predicates::Predicate { ty, bound },
        ));
        Ok(true)
    }
}
