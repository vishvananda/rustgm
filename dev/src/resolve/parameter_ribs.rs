//! rustc resolves an item's generic parameters in a rib of their own, apart
//! from the namespace of the members its body declares: an enum's variants
//! and a trait's or impl's associated items. Here parameters and members
//! share the item's body scope, which is observably the same until one of
//! each takes the same name and namespace, as alloc's
//! `enum ForceResult<Leaf, Internal> { Leaf(Leaf), Internal(Internal) }` does.
//! Only then does the body get a parameter rib between it and its parent, and
//! the parameter moves there: qualified lookup still finds the member in the
//! body, and lexical lookup, which passes over members, finds the parameter
//! in the rib. The rib takes over the body's lookup barrier, so every name is
//! still seen across the same number of barriers.
use super::{
    index::Resolver,
    model::{Binding, DefKind, Definition, Origin, Resolution, ScopeId},
    package::Error,
    table::BindingKey,
    unit::FileId,
};

impl Resolution {
    /// The parameter rib of an item body whose generic parameter shares a
    /// name with one of its members. Every other scope is created after its
    /// parent, so a parent created after its child is such a rib.
    pub(crate) fn parameter_rib(&self, scope: ScopeId) -> Option<ScopeId> {
        self.scopes
            .get(scope.0 as usize)?
            .parent
            .filter(|parent| parent.0 > scope.0)
    }
}

impl Resolver<'_> {
    /// Bind `definition`, whose `key` already binds a different definition of
    /// its scope, when one of the two is a generic parameter and the other a
    /// member: the parameter moves to the scope's rib. Returns whether
    /// `definition` is now bound; any other collision is left to the caller.
    pub(super) fn separates_parameter(
        &mut self,
        definition: &mut Definition,
        binding: Binding,
        key: BindingKey,
    ) -> Result<bool, Error> {
        let Some(existing) = self.facts.bindings.get_key(key).copied() else {
            return Ok(false);
        };
        let previous = &self.facts.definitions[existing.target.0 as usize];
        let associated = self.facts.scopes[definition.scope.0 as usize].associated;
        if existing.origin != Origin::Direct
            || existing.ambiguous
            || previous.scope != definition.scope
        {
            return Ok(false);
        }
        if definition.kind == DefKind::Generic && previous.kind.is_member(associated) {
            let rib = self.parameter_scope(definition.scope)?;
            definition.scope = rib;
            return self
                .facts
                .bindings
                .declare_key((rib, key.1, key.2, key.3), binding, false)
                .map_err(|message| Error::at(FileId(0), 0, message));
        }
        if previous.kind != DefKind::Generic || !definition.kind.is_member(associated) {
            return Ok(false);
        }
        let rib = self.parameter_scope(definition.scope)?;
        self.facts.bindings.remove_key(key);
        if !self
            .facts
            .bindings
            .declare_key((rib, key.1, key.2, key.3), existing, false)
            .map_err(|message| Error::at(FileId(0), 0, message))?
        {
            return Err(Error::at(
                FileId(0),
                0,
                "a parameter rib already binds a moved parameter's name",
            ));
        }
        self.facts.definitions[existing.target.0 as usize].scope = rib;
        self.facts
            .bindings
            .declare_key(key, binding, false)
            .map_err(|message| Error::at(FileId(0), 0, message))
    }
    /// The parameter rib of `scope`, inserted between it and its parent on
    /// first use with the barrier, label barrier and association of `scope`;
    /// `scope` itself keeps its label barrier and gives up its barrier.
    fn parameter_scope(&mut self, scope: ScopeId) -> Result<ScopeId, Error> {
        if let Some(rib) = self.facts.parameter_rib(scope) {
            return Ok(rib);
        }
        let body = &self.facts.scopes[scope.0 as usize];
        let (parent, module, barrier) = (body.parent, body.module, body.barrier);
        let (associated, label_barrier) = (body.associated, body.label_barrier);
        let rib = self.new_scope(parent, module, barrier)?;
        let frame = &mut self.facts.scopes[rib.0 as usize];
        frame.associated = associated;
        frame.label_barrier = label_barrier;
        let body = &mut self.facts.scopes[scope.0 as usize];
        body.parent = Some(rib);
        body.barrier = false;
        Ok(rib)
    }
}
