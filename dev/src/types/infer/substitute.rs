//! Instantiate typed templates into one body's graph without rechecking syntax.
use super::{
    Constructor, Context, Error, Store, TermId, TypeId, Value,
    arguments::{Argument, Slot},
};
use crate::types::{
    arguments::ConstKind,
    parameters::{ParameterId, ParameterKind},
};
use std::collections::HashMap;

/// A simultaneous binding environment for several templates in the same body.
/// Cached results retain inference terms, so later equations refine every use.
/// Construct and use this only with its original inference context and store.
pub struct Instantiation {
    parameters: HashMap<ParameterId, Argument>,
    completed: HashMap<(TermId, u16), TermId>,
    shifted: HashMap<(TermId, u16), TermId>,
    shifted_projections: bool,
}
impl Instantiation {
    /// Binder shifts can rebuild projections supplied through a type parameter,
    /// even when its template contains none. Register those new occurrences in
    /// their destination scope; this flag supplies no normalization permission.
    #[must_use]
    pub fn shifted_projections(&self) -> bool {
        self.shifted_projections
    }
    /// Retrieve an existing actual argument without inventing a new inference variable.
    #[must_use]
    pub(crate) fn binding(&self, id: ParameterId) -> Option<Argument> {
        self.parameters.get(&id).copied()
    }
    /// # Errors
    /// Rejects duplicate/mismatched bindings, invalid identities and work limits.
    pub fn new(
        context: &mut Context,
        store: &Store,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<Self, Error> {
        context.account_work(bindings.len())?;
        let mut parameters = HashMap::with_capacity(bindings.len());
        for &(id, argument) in bindings {
            let declaration = store.parameter(id).ok_or(Error::InvalidIdentity)?;
            match (declaration.kind, argument) {
                (ParameterKind::Type { .. }, Argument::Type(term)) => {
                    context.root(term)?;
                }
                (ParameterKind::Const { ty }, Argument::Const(term))
                    if context.constants().ty(term)? == ty => {}
                _ => return Err(Error::Mismatch),
            }
            if parameters.insert(id, argument).is_some() {
                return Err(Error::InvalidIdentity);
            }
        }
        Ok(Self {
            parameters,
            completed: HashMap::new(),
            shifted: HashMap::new(),
            shifted_projections: false,
        })
    }
}

impl Context {
    /// Fresh existential of a declared type or const parameter's kind.
    /// # Errors
    /// Rejects invalid parameter identities and exhausted inference resources.
    pub fn parameter_variable(
        &mut self,
        store: &Store,
        id: ParameterId,
    ) -> Result<Argument, Error> {
        match store.parameter(id).ok_or(Error::InvalidIdentity)?.kind {
            ParameterKind::Type { .. } => self.variable(super::Domain::Any).map(Argument::Type),
            ParameterKind::Const { ty } => self.const_variable(store, ty).map(Argument::Const),
        }
    }
    /// Substitute one constant using the same environment as its enclosing types.
    /// # Errors
    /// Rejects invalid constant identities and exhausted inference resources.
    pub fn instantiate_const(
        &mut self,
        store: &Store,
        constant: crate::types::arguments::ConstId,
        substitution: &Instantiation,
    ) -> Result<super::constants::ConstTermId, Error> {
        self.step()?;
        if let Some(ConstKind::Parameter(id)) = store.const_kind(constant)
            && let Some(Argument::Const(value)) = substitution.parameters.get(&id)
        {
            return Ok(*value);
        }
        self.known_const(store, constant)
    }
    /// Replace rigid parameters simultaneously with type/const inference terms.
    /// Arguments already belong to this body; caller universals must be free.
    /// # Errors
    /// Rejects duplicate/mismatched bindings, invalid identities and work limits.
    pub fn instantiate(
        &mut self,
        store: &mut Store,
        template: TypeId,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<TermId, Error> {
        let mut substitution = Instantiation::new(self, store, bindings)?;
        self.instantiate_with(store, template, &mut substitution)
    }
    /// Reuse a fixed parameter environment across field, signature or bound types.
    /// Lifetime substitution precedes this operation to avoid capturing arguments.
    /// # Errors
    /// Rejects invalid identities and exhausted inference resources.
    pub fn instantiate_with(
        &mut self,
        store: &mut Store,
        template: TypeId,
        substitution: &mut Instantiation,
    ) -> Result<TermId, Error> {
        if substitution.parameters.is_empty() {
            // A rigid canonical graph needs no rebuilding when no parameter is
            // replaced. In particular, keep opaque projection inputs canonical
            // instead of materializing and resolving their complete DAG again.
            self.step()?;
            return self.known(store, template);
        }
        let root = self.known(store, template)?;
        let mut work = vec![((root, 0_u16), false)];
        let mut children = Vec::new();
        while let Some(((term, depth), finish)) = work.pop() {
            self.step()?;
            if substitution.completed.contains_key(&(term, depth)) {
                continue;
            }
            if let Value::Known(ty) = self.terms[term.0 as usize].value
                && store.contains_parameters(ty) == Some(false)
            {
                substitution.completed.insert((term, depth), term);
                continue;
            }
            let kind = self.materialize(store, term)?;
            if let Constructor::Parameter(id) = kind
                && let Some(Argument::Type(value)) = substitution.parameters.get(&id)
            {
                let value = *value;
                let value = if let Some(&shifted) = substitution.shifted.get(&(value, depth)) {
                    shifted
                } else {
                    let shifted = self.shift_regions(store, value, depth)?;
                    substitution.shifted_projections |= shifted.projections;
                    substitution.shifted.insert((value, depth), shifted.term);
                    shifted.term
                };
                substitution.completed.insert((term, depth), value);
                continue;
            }
            let entry = self.terms[term.0 as usize];
            let child_depth = if matches!(kind, Constructor::Function { .. }) {
                depth.checked_add(1).ok_or(Error::Overflow)?
            } else {
                depth
            };
            if !finish {
                work.push(((term, depth), true));
                self.account_work((entry.end - entry.start) as usize)?;
                work.extend(
                    self.edges[entry.start as usize..entry.end as usize]
                        .iter()
                        .rev()
                        .map(|id| ((*id, child_depth), false)),
                );
                continue;
            }
            children.clear();
            for child in &self.edges[entry.start as usize..entry.end as usize] {
                children.push(
                    *substitution
                        .completed
                        .get(&(*child, child_depth))
                        .ok_or(Error::InvalidIdentity)?,
                );
            }
            let result =
                self.replace_parameters(store, kind, &children, &substitution.parameters)?;
            substitution.completed.insert((term, depth), result);
        }
        substitution
            .completed
            .get(&(root, 0))
            .copied()
            .ok_or(Error::InvalidIdentity)
    }
    fn replace_parameters(
        &mut self,
        store: &Store,
        kind: Constructor,
        children: &[TermId],
        bindings: &HashMap<ParameterId, Argument>,
    ) -> Result<TermId, Error> {
        if let Constructor::ArrayTerm(id) = kind
            && let Some(ConstKind::Parameter(parameter)) = store.const_kind(id)
            && let Some(Argument::Const(id)) = bindings.get(&parameter)
        {
            return self.construct(Constructor::ArrayInfer(*id), children);
        }
        let Some((args, prefix)) = kind.application() else {
            return self.construct(kind, children);
        };
        let mut arguments = Vec::new();
        let mut types = children[prefix..].iter();
        let count = self.slots(args)?.len();
        self.account_work(count)?;
        for index in 0..count {
            arguments.push(match self.slots(args)?[index] {
                Slot::Lifetime(region) => Argument::Lifetime(region),
                Slot::Type => Argument::Type(*types.next().ok_or(Error::InvalidIdentity)?),
                Slot::Const(id) => {
                    if let Some(ConstKind::Parameter(parameter)) = self.constants().value(id)?
                        && let Some(Argument::Const(value)) = bindings.get(&parameter)
                    {
                        Argument::Const(*value)
                    } else {
                        Argument::Const(id)
                    }
                }
            });
        }
        self.rebuild_application(kind, &children[..prefix], &arguments)
    }
}
