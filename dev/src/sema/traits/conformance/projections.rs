//! Checked associated definitions are local to this impl's method conformance.
use super::{Contract, Declarations, DefId, Predicate, TypeId};
use crate::types::{
    GenericArgs, ItemId, Kind,
    lifetime_proof::{Obligations, ScopeId},
    parameters::predicates::Bound,
    substitute::{self, Request},
};
use std::collections::HashMap;

/// Construct only after every associated definition has passed impl conformance.
pub(in crate::sema) struct Definitions {
    parent: Predicate,
    results: HashMap<ItemId, TypeId>,
}
impl Definitions {
    pub(in crate::sema) fn new(
        facts: &Declarations,
        parent: Predicate,
        checked: &[(DefId, DefId)],
    ) -> Result<Self, &'static str> {
        let mut results = HashMap::new();
        for &(declaration, definition) in checked {
            let ty = facts.definitions[definition.0 as usize]
                .ok_or("missing checked associated definition")?;
            // A checked definition is not permission to form any application.
            // Let the normalizer select families and retain their complete input
            // proof, including zero-parameter families with where clauses.
            if !facts.associated.families.contains_key(&declaration)
                && !facts.associated.families.contains_key(&definition)
            {
                results.insert(ItemId(declaration.0), ty);
            }
        }
        Ok(Self { parent, results })
    }

    fn resolve(
        &self,
        facts: &mut Declarations,
        request: Request,
    ) -> Result<Option<TypeId>, &'static str> {
        let Some(application) = request.alias.associated() else {
            return facts.types.intern(request.alias.kind()).map(Some);
        };
        let Some(&result) = self.results.get(&application.item) else {
            return Ok(None);
        };
        let mut shift = substitute::Bound::shift(request.depth);
        let parent = shift.predicate(&mut facts.types, self.parent)?;
        if parent
            != (Predicate {
                ty: application.subject,
                bound: Bound::Trait(application.trait_application),
            })
        {
            return Ok(None);
        }
        if application.arguments != GenericArgs::EMPTY {
            return Err("associated families require advanced generic relationships");
        }
        shift.apply(&mut facts.types, result).map(Some)
    }

    /// Constant types are roots in the impl binder, without a callable's outer binder.
    pub(in crate::sema) fn values(
        &self,
        facts: &mut Declarations,
        environment: Option<std::sync::Arc<crate::types::parameters::environment::Environment>>,
        proof: &mut Obligations,
        types: &mut [TypeId],
    ) -> Result<(), &'static str> {
        if types
            .iter()
            .all(|&ty| facts.types.contains_projections(ty) == Some(false))
        {
            return Ok(());
        }
        let mut session = crate::sema::projections::Session::new(
            environment,
            proof,
            ScopeId::ROOT,
            crate::sema::traits::implementations::Availability::Declarations,
            |facts, request| self.resolve(facts, request),
        )?;
        for ty in types {
            *ty = session.normalize(facts, *ty)?.0;
        }
        Ok(())
    }

    pub(super) fn signature(
        &self,
        facts: &mut Declarations,
        contract: &Contract,
        proof: &mut Obligations,
        ty: TypeId,
        mut implied: Option<&mut super::implied::Formation>,
    ) -> Result<TypeId, &'static str> {
        if facts.types.contains_projections(ty) == Some(false) {
            return Ok(ty);
        }
        let Some(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_,
            convention,
        }) = facts.types.kind(ty)
        else {
            return Err("missing conformance signature");
        };
        let mut parameters = facts
            .types
            .list(parameters)
            .ok_or("missing conformance parameters")?
            .to_vec();
        // The outer signature was already opened by mapping::contract. Only its
        // components start normalization roots; nested callables retain binders.
        let mut session = crate::sema::projections::Session::new(
            Some(std::sync::Arc::clone(&contract.environment)),
            proof,
            ScopeId::ROOT,
            crate::sema::traits::implementations::Availability::Declarations,
            |facts, request| {
                if let Some(implied) = &mut implied {
                    implied.record(facts, request)?;
                }
                self.resolve(facts, request)
            },
        )?;
        for ty in &mut parameters {
            *ty = session.normalize(facts, *ty)?.0;
        }
        let result = session.normalize(facts, result)?.0;
        let parameters = facts.types.intern_list(&parameters)?;
        facts.types.intern(Kind::Function {
            parameters,
            result,
            bound_regions,
            unsafe_,
            convention,
        })
    }
}
