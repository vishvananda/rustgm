//! Instantiate typed templates into one body's graph without rechecking syntax.
use super::memo::Memo;
use super::{
    Constructor, Context, Error, Store, TermId, TypeId, Value,
    arguments::{Argument, Slot},
};
use crate::support::hash::HashMap;
use crate::types::{
    arguments::ConstKind,
    parameters::{ParameterId, ParameterKind},
};

/// A simultaneous binding environment for several templates in the same body.
/// Cached results retain inference terms, so later equations refine every use.
/// Construct and use this only with its original inference context and store.
pub struct Instantiation {
    parameters: Memo<ParameterId, Argument>,
    opaques: HashMap<crate::types::opaque::OpaqueId, TermId>,
    completed: Memo<(TermId, u16), TermId>,
    shifted: Memo<(TermId, u16), TermId>,
    shifted_projections: bool,
    /// Traversal storage reused by every template of this environment.
    work: Vec<((TermId, u16), bool)>,
    pub(super) children: Vec<TermId>,
}
/// Storage of finished environments, reused by later ones of the same body
/// owner so that each small instantiation need not allocate its maps.
#[derive(Default)]
pub struct InstantiationPool {
    spare: Vec<Instantiation>,
}
impl Instantiation {
    /// [`Self::new`] using storage recycled into `pool`.
    /// # Errors
    /// Rejects duplicate/mismatched bindings, invalid identities and work limits.
    pub fn pooled(
        context: &mut Context,
        store: &Store,
        bindings: &[(ParameterId, Argument)],
        pool: &mut InstantiationPool,
    ) -> Result<Self, Error> {
        let mut value = pool.spare.pop().unwrap_or_else(Self::empty);
        value.bind(context, store, bindings)?;
        Ok(value)
    }
    /// Return this environment's storage to `pool`; its terms stay valid.
    pub fn recycle(mut self, pool: &mut InstantiationPool) {
        const SPARE: usize = 4;
        if pool.spare.len() < SPARE && self.opaques.is_empty() {
            self.parameters.clear();
            self.completed.clear();
            self.shifted.clear();
            self.shifted_projections = false;
            self.work.clear();
            self.children.clear();
            pool.spare.push(self);
        }
    }
    fn empty() -> Self {
        Self {
            parameters: Memo::default(),
            opaques: HashMap::default(),
            completed: Memo::default(),
            shifted: Memo::default(),
            shifted_projections: false,
            work: Vec::new(),
            children: Vec::new(),
        }
    }
    fn bind(
        &mut self,
        context: &mut Context,
        store: &Store,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<(), Error> {
        context.account_work(bindings.len())?;
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
            if self.parameters.insert(id, argument).is_some() {
                return Err(Error::InvalidIdentity);
            }
        }
        Ok(())
    }
    /// Use fresh hidden-type terms only while checking their defining result.
    /// # Errors
    /// Rejects invalid identities, duplicate declarations and exhausted inference resources.
    pub fn opaque_results(
        context: &mut Context,
        store: &Store,
        ids: &[crate::types::opaque::OpaqueId],
    ) -> Result<Self, Error> {
        let mut value = Self::new(context, store, &[])?;
        for &id in ids {
            if store.opaque_declaration(id).is_none() || value.opaques.contains_key(&id) {
                return Err(Error::InvalidIdentity);
            }
            value
                .opaques
                .insert(id, context.variable(super::Domain::Any)?);
        }
        Ok(value)
    }
    /// Fresh terms to validate and publish after all defining-body return paths agree.
    pub fn opaque_terms(
        &self,
    ) -> impl Iterator<Item = (crate::types::opaque::OpaqueId, TermId)> + use<'_> {
        self.opaques.iter().map(|(&id, &term)| (id, term))
    }
    /// Binder shifts can rebuild projections supplied through a type parameter,
    /// even when its template contains none. Register those new occurrences in
    /// their destination scope; this flag supplies no normalization permission.
    #[must_use]
    pub fn shifted_projections(&self) -> bool {
        self.shifted_projections
    }
    /// No template was instantiated in this environment yet, and no opaque
    /// result is substituted: a compiled signature replays exactly.
    pub(super) fn fresh(&self) -> bool {
        !self.parameters.is_empty() && self.opaques.is_empty() && self.completed.is_empty()
    }
    /// A type binding shifted under `depth` binders, shared like the general path.
    pub(super) fn shifted_binding(
        &mut self,
        context: &mut Context,
        store: &mut Store,
        value: TermId,
        depth: u16,
    ) -> Result<TermId, Error> {
        if let Some(shifted) = self.shifted.get(&(value, depth)) {
            return Ok(shifted);
        }
        let shifted = context.shift_regions(store, value, depth)?;
        self.shifted_projections |= shifted.projections;
        self.shifted.insert((value, depth), shifted.term);
        Ok(shifted.term)
    }
    /// Retrieve an existing actual argument without inventing a new inference variable.
    #[must_use]
    pub(crate) fn binding(&self, id: ParameterId) -> Option<Argument> {
        self.parameters.get(&id)
    }
    /// # Errors
    /// Rejects duplicate/mismatched bindings, invalid identities and work limits.
    pub fn new(
        context: &mut Context,
        store: &Store,
        bindings: &[(ParameterId, Argument)],
    ) -> Result<Self, Error> {
        let mut value = Self {
            parameters: Memo::with_capacity(bindings.len()),
            ..Self::empty()
        };
        value.bind(context, store, bindings)?;
        Ok(value)
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
            return Ok(value);
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
        if substitution.parameters.is_empty() && substitution.opaques.is_empty() {
            // A rigid canonical graph needs no rebuilding when no parameter is
            // replaced. In particular, keep opaque projection inputs canonical
            // instead of materializing and resolving their complete DAG again.
            self.step()?;
            return self.known(store, template);
        }
        let root = self.known(store, template)?;
        let mut work = std::mem::take(&mut substitution.work);
        let mut children = std::mem::take(&mut substitution.children);
        work.clear();
        work.push(((root, 0_u16), false));
        let result = self.instantiate_graph(store, substitution, &mut work, &mut children);
        substitution.work = work;
        substitution.children = children;
        result?;
        substitution
            .completed
            .get(&(root, 0))
            .ok_or(Error::InvalidIdentity)
    }
    fn instantiate_graph(
        &mut self,
        store: &mut Store,
        substitution: &mut Instantiation,
        work: &mut Vec<((TermId, u16), bool)>,
        children: &mut Vec<TermId>,
    ) -> Result<(), Error> {
        while let Some(((term, depth), finish)) = work.pop() {
            self.step()?;
            if substitution.completed.get(&(term, depth)).is_some() {
                continue;
            }
            if let Value::Known(ty) = self.terms[term.0 as usize].value
                && store.contains_parameters(ty) == Some(false)
                && (substitution.opaques.is_empty() || store.contains_opaques(ty) == Some(false))
            {
                substitution.completed.insert((term, depth), term);
                continue;
            }
            let kind = self.materialize(store, term)?;
            if let Constructor::Opaque(id, _) = kind
                && let Some(&value) = substitution.opaques.get(&id)
            {
                substitution.completed.insert((term, depth), value);
                continue;
            }
            if let Constructor::Parameter(id) = kind
                && let Some(Argument::Type(value)) = substitution.parameters.get(&id)
            {
                let value = if let Some(shifted) = substitution.shifted.get(&(value, depth)) {
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
            let child_depth = if matches!(
                kind,
                Constructor::Function { .. } | Constructor::Dynamic { .. }
            ) {
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
                    substitution
                        .completed
                        .get(&(*child, child_depth))
                        .ok_or(Error::InvalidIdentity)?,
                );
            }
            let result =
                self.replace_parameters(store, kind, children, &substitution.parameters)?;
            self.remember_structure(store, result)?;
            substitution.completed.insert((term, depth), result);
        }
        Ok(())
    }
    fn replace_parameters(
        &mut self,
        store: &Store,
        kind: Constructor,
        children: &[TermId],
        bindings: &Memo<ParameterId, Argument>,
    ) -> Result<TermId, Error> {
        if let Constructor::ArrayTerm(id) = kind
            && let Some(ConstKind::Parameter(parameter)) = store.const_kind(id)
            && let Some(Argument::Const(id)) = bindings.get(&parameter)
        {
            return self.construct(Constructor::ArrayInfer(id), children);
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
                        Argument::Const(value)
                    } else {
                        Argument::Const(id)
                    }
                }
            });
        }
        self.rebuild_application(kind, &children[..prefix], &arguments)
    }
}
