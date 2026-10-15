//! Caller assumptions are indexed once; alternatives use isolated complete-goal equations.
use crate::types::{
    ItemId, Kind, Primitive, Store,
    infer::{self, Constructor, Context, predicates::Predicate as Goal},
    parameters::predicates::{Bound, Predicate},
};
use std::collections::{HashMap, HashSet};
#[cfg(test)]
mod tests;

#[derive(Default)]
pub(in crate::sema) struct Assumptions {
    candidates: HashMap<(ItemId, Head), Vec<Predicate>>,
    seen: HashSet<Predicate>,
}

/// Equations against this actual caller assumption have been committed to inference.
/// Their region equalities still require the ordinary body lifetime proof.
pub(in crate::sema) struct Evidence(Predicate);
/// Applicability may defer unresolved evidence; only Proven authorizes a body goal.
pub(in crate::sema) enum Selection {
    Absent,
    Deferred,
    Proven(Evidence),
}
impl Evidence {
    pub fn assumption(&self) -> Predicate {
        self.0
    }
}

impl Assumptions {
    pub fn insert(&mut self, store: &Store, predicate: Predicate) -> Result<(), infer::Error> {
        let Bound::Trait(id) = predicate.bound else {
            return Err(infer::Error::InvalidIdentity);
        };
        let application = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        if self.seen.len() >= 1_000_000 {
            return Err(infer::Error::Budget);
        }
        if self.seen.insert(predicate) {
            let head = Head::canonical(
                store
                    .kind(predicate.ty)
                    .ok_or(infer::Error::InvalidIdentity)?,
            );
            self.candidates
                .entry((application.item, head))
                .or_default()
                .push(predicate);
        }
        Ok(())
    }

    /// A declaration's own exact bound needs equations only when alternatives exist.
    /// This checks ambiguity, not whether an implementation satisfies a new goal.
    pub fn unambiguous_declaration(
        &self,
        store: &mut Store,
        context: &mut Context,
        predicate: Predicate,
    ) -> Result<bool, infer::Error> {
        context.account_work(1)?;
        if !self.seen.contains(&predicate) {
            return Err(infer::Error::InvalidIdentity);
        }
        let Bound::Trait(id) = predicate.bound else {
            return Err(infer::Error::InvalidIdentity);
        };
        let application = store
            .trait_application(id)
            .ok_or(infer::Error::InvalidIdentity)?;
        let head = Head::canonical(
            store
                .kind(predicate.ty)
                .ok_or(infer::Error::InvalidIdentity)?,
        );
        if self
            .candidates
            .get(&(application.item, head))
            .is_some_and(|candidates| candidates.len() == 1)
        {
            return Ok(true);
        }
        let mut substitution = infer::Instantiation::new(context, store, &[])?;
        let goal = context.instantiate_predicate(store, predicate, &mut substitution)?;
        Ok(self.prove(store, context, &goal)?.is_some())
    }

    /// Unknown Self is not guessed from the available assumptions. Other unresolved
    /// arguments may be inferred once exactly one candidate remains compatible.
    pub fn prove(
        &self,
        store: &Store,
        context: &mut Context,
        goal: &Goal,
    ) -> Result<Option<Evidence>, infer::Error> {
        Ok(match self.select(store, context, goal)? {
            Selection::Proven(evidence) => Some(evidence),
            Selection::Absent | Selection::Deferred => None,
        })
    }

    pub fn select(
        &self,
        store: &Store,
        context: &mut Context,
        goal: &Goal,
    ) -> Result<Selection, infer::Error> {
        context.account_work(1)?;
        let Some((item, _)) = goal.trait_arguments() else {
            return Err(infer::Error::InvalidIdentity);
        };
        let Some(head) = context.head(store, goal.subject())?.map(Head::inferred) else {
            return Ok(Selection::Deferred);
        };
        let mut selected = None;
        for &candidate in self.candidates.get(&(item, head)).into_iter().flatten() {
            let mut probe = Context::default();
            let imported = probe.import_predicate(store, context, goal)?;
            let result = probe.equate_trait_predicate(store, &imported, candidate);
            context.account_work(probe.steps)?;
            match result {
                Ok(()) => {
                    if selected.replace(candidate).is_some() {
                        return Ok(Selection::Deferred);
                    }
                }
                Err(infer::Error::Mismatch | infer::Error::Occurs) => {}
                Err(error) => return Err(error),
            }
        }
        if let Some(assumption) = selected {
            context.equate_trait_predicate(store, goal, assumption)?;
            Ok(Selection::Proven(Evidence(assumption)))
        } else {
            Ok(Selection::Absent)
        }
    }
}

/// Coarse candidate keys exclude lifetimes and children; complete equations still prove equality.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Head {
    Primitive(Primitive),
    Parameter(crate::types::parameters::ParameterId),
    Nominal(ItemId),
    FunctionItem(ItemId),
    Reference(bool),
    Pointer(bool),
    Tuple,
    Array,
    Slice,
    Function,
    Never,
}
impl Head {
    const fn canonical(kind: Kind) -> Self {
        match kind {
            Kind::Primitive(p) => Self::Primitive(p),
            Kind::Parameter(id) => Self::Parameter(id),
            Kind::Adt(item, _) => Self::Nominal(item),
            Kind::FunctionItem { item, .. } => Self::FunctionItem(item),
            Kind::Reference { mutable, .. } => Self::Reference(mutable),
            Kind::Pointer { mutable, .. } => Self::Pointer(mutable),
            Kind::Tuple(_) => Self::Tuple,
            Kind::Array { .. } | Kind::ArrayTerm { .. } => Self::Array,
            Kind::Slice(_) => Self::Slice,
            Kind::Function { .. } => Self::Function,
            Kind::Never => Self::Never,
        }
    }
    const fn inferred(kind: Constructor) -> Self {
        match kind {
            Constructor::Primitive(p) => Self::Primitive(p),
            Constructor::Parameter(id) => Self::Parameter(id),
            Constructor::Adt(item, _) | Constructor::AdtArgs(item, _) => Self::Nominal(item),
            Constructor::FunctionItem(item, _) | Constructor::FunctionItemArgs(item, _) => {
                Self::FunctionItem(item)
            }
            Constructor::Reference { mutable, .. } => Self::Reference(mutable),
            Constructor::Pointer { mutable } => Self::Pointer(mutable),
            Constructor::Tuple => Self::Tuple,
            Constructor::Array(_) | Constructor::ArrayTerm(_) | Constructor::ArrayInfer(_) => {
                Self::Array
            }
            Constructor::Slice => Self::Slice,
            Constructor::Function { .. } => Self::Function,
            Constructor::Never => Self::Never,
        }
    }
}
