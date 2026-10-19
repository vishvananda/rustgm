//! Probe a complete trait goal without separating roots that share inference variables.
use super::{
    Argument, Canonical, CanonicalArgument, Context, Error, Predicate, Requirement, Store,
};
use crate::types::parameters::predicates::Bound;
use std::collections::HashSet;

#[derive(Clone, Copy)]
enum Relation {
    Shape,
    Scoped,
}

impl Context {
    /// Commit a proved predicate's inputs and result to the originating graph.
    /// The caller must already have proved normalization and retain its evidence.
    /// Projection input equations here identify that query; they never infer inputs
    /// from an independently equal result.
    /// # Errors
    /// Rejects changed predicate kinds, malformed projections and failed equations.
    pub fn equate_proved_predicate(
        &mut self,
        store: &mut Store,
        goal: &Predicate,
        proof: Canonical,
    ) -> Result<(), Error> {
        if let Some(bound) = goal.builtin_bound() {
            if bound != proof.bound {
                return Err(Error::Mismatch);
            }
            return self.equate_canonical_input(store, goal.subject(), proof.ty, &[]);
        }
        let Some(result) = goal.projection_result() else {
            return self.equate_trait_predicate(store, goal, proof);
        };
        let Bound::ProjectionEqual(expected) = proof.bound else {
            return Err(Error::Mismatch);
        };
        let Some(crate::types::Kind::Projection(application)) = store.kind(proof.ty) else {
            return Err(Error::InvalidIdentity);
        };
        let original = self.projection_application(store, goal.subject())?;
        self.equate_projection_inputs(store, &original, application)?;
        let expected = self.known(store, expected)?;
        self.equate_scoped_terms(store, result, expected)
    }

    /// Existential header compatibility, with lifetime proof deliberately deferred.
    /// This overapproximation may establish disjointness, never body permissions.
    /// # Errors
    /// Rejects incompatible traits/arguments, malformed data and exhausted inference work.
    pub fn equate_predicate_shapes(
        &mut self,
        store: &Store,
        left: &Predicate,
        right: &Predicate,
    ) -> Result<(), Error> {
        self.relate_predicates(store, left, right, Relation::Shape)
    }

    /// Equate inferred goals with shared type/const variables on either side.
    /// Callable-local regions stay scoped; outer equalities remain proof obligations.
    /// Probe alternatives in isolated contexts before committing a selected header.
    /// # Errors
    /// Rejects mismatches, local lifetime capture, occurs cycles and exhausted work.
    pub fn equate_predicates(
        &mut self,
        store: &Store,
        left: &Predicate,
        right: &Predicate,
    ) -> Result<(), Error> {
        self.relate_predicates(store, left, right, Relation::Scoped)
    }

    fn relate_predicates(
        &mut self,
        store: &Store,
        left: &Predicate,
        right: &Predicate,
        relation: Relation,
    ) -> Result<(), Error> {
        let (Requirement::Trait(a), Requirement::Trait(b)) =
            (&left.requirement, &right.requirement)
        else {
            return Err(Error::InvalidIdentity);
        };
        if a.item != b.item || a.arguments.len() != b.arguments.len() {
            return Err(Error::Mismatch);
        }
        self.account_work(1 + a.arguments.len())?;
        if matches!(relation, Relation::Scoped) {
            let inputs: Vec<_> =
                std::iter::once((Argument::Type(left.subject), Argument::Type(right.subject)))
                    .chain(a.arguments.iter().copied().zip(b.arguments.iter().copied()))
                    .collect();
            return self.equate_input_pairs(store, &inputs);
        }
        // A successful root pair remains equal under compatible later equations.
        // Share evidence only within this relation and its enclosing binder scope.
        let mut proved = (a.arguments.len() > 1).then(HashSet::new);
        let mut relate = |context: &mut Self, left, right| {
            if let Some(proved) = &mut proved {
                let pair = (context.root(left)?, context.root(right)?);
                if !proved.insert(pair) {
                    return Ok(());
                }
            }
            context.relate_predicate_types(store, left, right, relation)
        };
        relate(self, left.subject, right.subject)?;
        for (&a, &b) in a.arguments.iter().zip(&b.arguments) {
            match (a, b) {
                (Argument::Type(a), Argument::Type(b)) => {
                    relate(self, a, b)?;
                }
                (Argument::Const(a), Argument::Const(b)) => self.equate_consts(a, b)?,
                (Argument::Lifetime(a), Argument::Lifetime(b)) => {
                    if matches!(relation, Relation::Scoped) && a != b {
                        self.region_equalities.push((a, b));
                    }
                }
                _ => return Err(Error::Mismatch),
            }
        }
        Ok(())
    }

    fn relate_predicate_types(
        &mut self,
        store: &Store,
        left: super::TermId,
        right: super::TermId,
        relation: Relation,
    ) -> Result<(), Error> {
        match relation {
            Relation::Shape => self.equate_shape(store, left, right),
            Relation::Scoped => self.equate_scoped_terms(store, left, right),
        }
    }
    /// Import Self and every argument with one shared reachable-graph mapping.
    /// Failed candidate equations in this context cannot constrain the source.
    /// # Errors
    /// Rejects invalid identities, compact overflow and exhausted inference resources.
    pub fn import_predicate(
        &mut self,
        store: &Store,
        source: &mut Self,
        predicate: &Predicate,
    ) -> Result<Predicate, Error> {
        let mut mappings = super::super::import::Import::default();
        predicate.map_arguments(|arg| {
            self.step()?;
            mappings.argument(store, source, self, arg)
        })
    }

    /// Destructively equate a goal with one canonical assumption. Selection must
    /// probe alternatives in isolated contexts before committing these equations.
    /// Region equalities are obligations, not proof that two caller lifetimes agree.
    /// # Errors
    /// Rejects mismatched traits/arguments, invalid identities, occurs cycles and work limits.
    pub fn equate_trait_predicate(
        &mut self,
        store: &mut Store,
        predicate: &Predicate,
        assumption: Canonical,
    ) -> Result<(), Error> {
        self.equate_trait_predicate_inside(store, predicate, assumption, &[])
    }

    /// Commit selected input equations beneath the original enclosing binders.
    /// Ambient slots are rigid identities; only nested callable binders are alpha-renamed.
    /// # Errors
    /// Rejects input mismatch, capture of an ambient lifetime and exhausted work.
    pub fn equate_trait_predicate_inside(
        &mut self,
        store: &mut Store,
        predicate: &Predicate,
        assumption: Canonical,
        ambient: &[u32],
    ) -> Result<(), Error> {
        let (Requirement::Trait(trait_), Bound::Trait(id)) =
            (&predicate.requirement, assumption.bound)
        else {
            return Err(Error::Mismatch);
        };
        let application = store.trait_application(id).ok_or(Error::InvalidIdentity)?;
        if application.item != trait_.item {
            return Err(Error::Mismatch);
        }
        self.equate_canonical_inputs(
            store,
            Some((predicate.subject, assumption.ty)),
            &trait_.arguments,
            application.arguments,
            ambient,
        )
    }

    pub(in crate::types::infer) fn equate_canonical_inputs(
        &mut self,
        store: &mut Store,
        subject: Option<(super::TermId, super::TypeId)>,
        inferred: &[Argument],
        arguments: crate::types::GenericArgs,
        ambient: &[u32],
    ) -> Result<(), Error> {
        let count = store
            .arguments(arguments)
            .ok_or(Error::InvalidIdentity)?
            .len();
        if count != inferred.len() {
            return Err(Error::Mismatch);
        }
        self.account_work(usize::from(subject.is_some()) + count)?;
        let mut proved = (count > 1).then(HashSet::new);
        let mut equate = |context: &mut Self, store: &mut Store, term, ty| {
            if let Some(proved) = &mut proved
                && !proved.insert((context.root(term)?, ty))
            {
                return Ok(());
            }
            context.equate_canonical_input(store, term, ty, ambient)
        };
        if let Some((term, ty)) = subject {
            equate(self, store, term, ty)?;
        }
        for (index, &inferred) in inferred.iter().enumerate() {
            let canonical = store
                .arguments(arguments)
                .and_then(|mut arguments| arguments.nth(index))
                .ok_or(Error::InvalidIdentity)?;
            match (inferred, canonical) {
                (Argument::Type(term), CanonicalArgument::Type(ty)) => {
                    equate(self, store, term, ty)?;
                }
                (Argument::Const(term), CanonicalArgument::Const(id)) => {
                    let known = self.known_const(store, id)?;
                    self.equate_consts(term, known)?;
                }
                (Argument::Lifetime(a), CanonicalArgument::Lifetime(b)) => {
                    self.equate_regions_inside(a, b, ambient)?;
                }
                _ => return Err(Error::Mismatch),
            }
        }
        Ok(())
    }
    // Equal canonical operations are congruent. This never guesses their inputs
    // from result equality, and unchanged opaque inputs do not need a normal form.
    fn equate_canonical_input(
        &mut self,
        store: &mut Store,
        term: super::TermId,
        canonical: super::TypeId,
        ambient: &[u32],
    ) -> Result<(), Error> {
        self.step()?;
        let term = self.root(term)?;
        if self.settled_type(term) == Some(canonical) {
            return Ok(());
        }
        if store
            .contains_projections(canonical)
            .ok_or(Error::InvalidIdentity)?
        {
            match self.resolved(store, term) {
                Ok(original) if original == canonical => return Ok(()),
                Ok(_) | Err(Error::Unresolved) => {}
                Err(error) => return Err(error),
            }
        }
        let known = self.known(store, canonical)?;
        if ambient.is_empty()
            && store
                .contains_projections(canonical)
                .ok_or(Error::InvalidIdentity)?
            && !store
                .contains_callables(canonical)
                .ok_or(Error::InvalidIdentity)?
        {
            // Settle independent holes before comparing opaque operation identities.
            // The blocked result pairs never contribute equations on their inputs.
            let mut blocked = Vec::new();
            self.unify(store, term, known, true, Some(&mut blocked))?;
            for (left, right) in blocked {
                if self.resolved(store, left)? != self.resolved(store, right)? {
                    return Err(Error::Normalization);
                }
            }
            return Ok(());
        }
        self.equate_scoped_inside(store, term, known, ambient)
    }
}
