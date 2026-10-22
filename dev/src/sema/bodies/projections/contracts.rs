//! Ready alias contracts freshen private identities before actual arguments enter.
use super::{Checker, Contract, Error, HashMap, NodeId, Obligations, ScopeId};
use crate::types::{
    TypeId,
    arguments::Argument as Canonical,
    infer::arguments::Argument,
    parameters::predicates::{Bound as PredicateBound, Predicate},
    substitute::Bound,
};

impl Checker<'_> {
    pub(in crate::sema::bodies) fn prove_associated_equality(
        &mut self,
        node: NodeId,
        left: TypeId,
        right: TypeId,
    ) -> Result<(), Error> {
        let first = u32::try_from(self.facts.body_regions.len())
            .ok()
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| self.error(node, "associated equality region space exhausted"))?;
        let mut proof = Obligations::new(first);
        let left = self.normalize_body_type(node, &mut proof, ScopeId::ROOT, left)?;
        let right = self.normalize_body_type(node, &mut proof, ScopeId::ROOT, right)?;
        let mut context = crate::types::infer::Context::default();
        context
            .equate_normal_forms(&self.facts.types, left, right)
            .map_err(|e| self.error(node, e))?;
        self.state
            .infer
            .account_work(context.steps)
            .map_err(|e| self.error(node, e))?;
        proof.equalities.append(&mut context.region_equalities);
        proof
            .retain_body(self.facts, self.state.body, self.key(node))
            .map_err(|e| self.error(node, e))
    }

    pub(super) fn complete_contract(
        &mut self,
        node: NodeId,
        ambient: ScopeId,
        contract: Contract,
        proof: &mut Obligations,
    ) -> Result<(), Error> {
        let depth = proof
            .scope_depth(ambient)
            .map_err(|e| self.error(node, e))?;
        let mut expansion = self.instantiate_contract(node, proof.regions.end, depth, contract)?;
        let source = std::mem::replace(&mut expansion.proof, Obligations::new(proof.regions.end));
        let mapping = proof
            .graft(source, ambient)
            .map_err(|e| self.error(node, e))?;
        let roots = proof.type_roots().collect::<Vec<_>>();
        let mut normalized = HashMap::default();
        for (scope, ty) in roots {
            let result = self.normalize_body_type(node, proof, scope, ty)?;
            normalized.insert((scope, ty), result);
        }
        proof.map_type_roots(|scope, ty| normalized.get(&(scope, ty)).copied().unwrap_or(ty));
        for (sized, roots) in [
            (false, expansion.normalizations.as_slice()),
            (true, expansion.sized.as_slice()),
        ] {
            for &(scope, ty) in roots {
                let scope = *mapping
                    .get(scope.0 as usize)
                    .ok_or_else(|| self.error(node, "missing alias proof scope"))?;
                let ty = self.normalize_body_type(node, proof, scope, ty)?;
                if sized {
                    self.prove_projection_size(
                        node,
                        proof.scope_depth(scope).map_err(|e| self.error(node, e))?,
                        ty,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn instantiate_contract(
        &mut self,
        node: NodeId,
        first: u32,
        depth: u16,
        contract: Contract,
    ) -> Result<crate::sema::alias_requirements::Expansion, Error> {
        let mut expansion = contract.expansion;
        let plan = expansion.proof.scope_plan();
        let interval = expansion
            .proof
            .template_interval()
            .map_err(|e| self.error(node, e))?;
        let mut fresh =
            Bound::rebase_free_regions(interval, first).map_err(|e| self.error(node, e))?;
        let rebasing = expansion
            .proof
            .rebase(&mut self.facts.types, first)
            .map_err(|e| self.error(node, e))?;
        let mut arguments = Vec::with_capacity(contract.arguments.len());
        for (id, argument) in contract.arguments {
            let value = match argument {
                Argument::Type(term) => self
                    .state
                    .infer
                    .resolved(&mut self.facts.types, term)
                    .map(Canonical::Type),
                Argument::Const(term) => self
                    .state
                    .infer
                    .resolved_const(&mut self.facts.types, term)
                    .map(Canonical::Const),
                Argument::Lifetime(_) => {
                    return Err(self.error(node, "invalid deferred proof argument"));
                }
            }
            .map_err(|e| self.error(node, e))?;
            arguments.push((id, value));
        }
        let mut parameters =
            Bound::parameters_captured(&self.facts.types, &arguments, contract.regions)
                .map_err(|e| self.error(node, e))?;
        expansion
            .proof
            .substitute(&mut self.facts.types, &mut parameters)
            .map_err(|e| self.error(node, e))?;
        for (scope, ty) in expansion
            .normalizations
            .iter_mut()
            .chain(&mut expansion.sized)
        {
            *ty = fresh
                .apply(&mut self.facts.types, *ty)
                .map_err(|e| self.error(node, e))?;
            let local = plan.depth(*scope).map_err(|e| self.error(node, e))?;
            *ty = parameters
                .apply_inside(&mut self.facts.types, *ty, local)
                .map_err(|e| self.error(node, e))?;
        }
        // Caller universals are interpreted below both the source callable and
        // alias-local binders; local quantified slots must never become body aliases.
        let mut universals = Bound::new(self.state.call_types.universal_arguments());
        expansion
            .proof
            .substitute_inside(&mut self.facts.types, &mut universals, depth)
            .map_err(|e| self.error(node, e))?;
        for (scope, ty) in expansion
            .normalizations
            .iter_mut()
            .chain(&mut expansion.sized)
        {
            let depth = depth
                .checked_add(plan.depth(*scope).map_err(|e| self.error(node, e))?)
                .ok_or_else(|| self.error(node, "alias scope depth exhausted"))?;
            *ty = universals
                .apply_inside(&mut self.facts.types, *ty, depth)
                .map_err(|e| self.error(node, e))?;
        }
        self.state
            .infer
            .account_work(
                rebasing
                    .saturating_add(fresh.steps)
                    .saturating_add(parameters.steps)
                    .saturating_add(universals.steps),
            )
            .map_err(|e| self.error(node, e))?;
        Ok(expansion)
    }

    fn prove_projection_size(&mut self, node: NodeId, depth: u16, ty: TypeId) -> Result<(), Error> {
        let environment = self
            .state
            .method_environment
            .as_ref()
            .map(|env| {
                env.substitute_regions(&mut self.facts.types, &mut Bound::shift(depth))
                    .map(std::sync::Arc::new)
            })
            .transpose()
            .map_err(|e| self.error(node, e))?;
        let mut proof = crate::sema::nominals::Proof::new(environment);
        if !proof
            .prove(
                self.facts,
                Predicate {
                    ty,
                    bound: PredicateBound::Sized,
                },
            )
            .map_err(|e| self.error(node, e))?
        {
            return Err(self.error(node, "alias projection requires a Sized type argument"));
        }
        Ok(())
    }
}
