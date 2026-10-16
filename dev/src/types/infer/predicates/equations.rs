//! Probe a complete trait goal without separating roots that share inference variables.
use super::{
    Argument, Canonical, CanonicalArgument, Context, Error, Predicate, Requirement, Store, Trait,
};
use crate::types::parameters::predicates::Bound;

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
        if original.item != application.item
            || !original.arguments.is_empty()
            || application.arguments != crate::types::GenericArgs::EMPTY
        {
            return Err(Error::InvalidIdentity);
        }
        self.equate_trait_predicate(
            store,
            &original.predicate,
            Canonical {
                ty: application.subject,
                bound: Bound::Trait(application.trait_application),
            },
        )?;
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
        self.relate_predicate_types(store, left.subject, right.subject, relation)?;
        for (&a, &b) in a.arguments.iter().zip(&b.arguments) {
            match (a, b) {
                (Argument::Type(a), Argument::Type(b)) => {
                    self.relate_predicate_types(store, a, b, relation)?;
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
        let subject = self.import_with(store, source, predicate.subject, &mut mappings)?;
        let requirement = match &predicate.requirement {
            Requirement::Builtin(bound) => Requirement::Builtin(*bound),
            Requirement::ProjectionEqual(result) => Requirement::ProjectionEqual(
                self.import_with(store, source, *result, &mut mappings)?,
            ),
            Requirement::Trait(trait_) => {
                self.account_work(1 + trait_.arguments.len())?;
                let mut arguments = Vec::with_capacity(trait_.arguments.len());
                for &argument in &trait_.arguments {
                    arguments.push(match argument {
                        Argument::Lifetime(region) => Argument::Lifetime(region),
                        Argument::Type(term) => {
                            Argument::Type(self.import_with(store, source, term, &mut mappings)?)
                        }
                        Argument::Const(term) => {
                            Argument::Const(mappings.constant(store, source, self, term)?)
                        }
                    });
                }
                Requirement::Trait(Box::new(Trait {
                    item: trait_.item,
                    arguments,
                }))
            }
        };
        Ok(Predicate {
            subject,
            requirement,
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
        let arguments = store
            .arguments(application.arguments)
            .ok_or(Error::InvalidIdentity)?;
        if application.item != trait_.item || arguments.len() != trait_.arguments.len() {
            return Err(Error::Mismatch);
        }
        self.account_work(1 + arguments.len())?;
        self.equate_canonical_input(store, predicate.subject, assumption.ty, ambient)?;
        for (index, &inferred) in trait_.arguments.iter().enumerate() {
            let canonical = store
                .arguments(application.arguments)
                .and_then(|mut arguments| arguments.nth(index))
                .ok_or(Error::InvalidIdentity)?;
            match (inferred, canonical) {
                (Argument::Type(term), CanonicalArgument::Type(ty)) => {
                    self.equate_canonical_input(store, term, ty, ambient)?;
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
