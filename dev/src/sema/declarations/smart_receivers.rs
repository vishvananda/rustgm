//! Receiver validity consumes actual source traits and retains their lifetime debts.
use super::{Checker, Error, NodeKey, Type, TypeId};
use crate::{
    sema::{compiler_items::arbitrary_self, model::Function},
    support::{hash::HashSet, retained},
};
impl Checker<'_> {
    pub(super) fn validate_smart_receiver(
        &mut self,
        function: &Function,
        key: NodeKey,
        mut current: TypeId,
        target: TypeId,
    ) -> Result<TypeId, Error> {
        let report = self.report;
        let level = arbitrary_self::level(report, key)?;
        let error = |message| crate::sema::model::Declarations::error(report, key, message);
        let environment = self
            .facts
            .generics
            .environments
            .get(&function.definition)
            .cloned();
        let mut proof = self.declaration_proof(function.definition, environment);
        let mut visited = HashSet::default();
        let result = loop {
            if current == target
                || matches!((self.facts.types.kind(current),self.facts.types.kind(target)),(Some(Type::Adt(a,_)),Some(Type::Adt(b,_)))if a==b)
            {
                break current;
            }
            if let Some(Type::Parameter(parameter)) = self.facts.types.kind(current)
                && self
                    .facts
                    .types
                    .parameter(parameter)
                    .is_some_and(|p| p.owner.0 == function.definition.0)
            {
                return Err(error("receiver cannot be the method's own type parameter"));
            }
            if visited.len() >= 1_000_000 || visited.contains(&current) {
                return Err(error(
                    "receiver Target chain is cyclic or exceeds its work budget",
                ));
            }
            if visited.len() == visited.capacity() {
                let count = visited.len().saturating_mul(2).max(4);
                crate::sema::header_attributes::reserve_source_contracts(
                    &mut self.facts,
                    1,
                    retained::new_map::<TypeId, ()>(count),
                )?;
                visited
                    .try_reserve(count - visited.len())
                    .map_err(|_| error("receiver visit allocation failed"))?;
            }
            visited.insert(current);
            current = arbitrary_self::next(report, &mut self.facts, &mut proof, current, level)
                .map_err(error)?
                .ok_or_else(|| {
                    error("receiver requires its actual implementation and Target chain to Self")
                })?;
        };
        self.retain_trait_proof(function.definition, key, &mut proof);
        Ok(result)
    }
}
