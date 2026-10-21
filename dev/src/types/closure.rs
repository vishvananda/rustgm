//! Closure identity is nominal; callable contracts and stored captures are separate edges.
use super::{
    GenericArgs, Store, TypeId,
    properties::{Properties, Property},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ClosureId(pub u32);

/// One concrete receiver adapter identity, separate from its original body.
/// Identity validation alone grants no execution or source-proof permission.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Adapter {
    instance: super::instance::Instance,
    receiver: TypeId,
    requested: crate::support::call_traits::CallTrait,
}
impl Adapter {
    pub(crate) fn new(
        types: &Store,
        instance: super::instance::Instance,
        requested: crate::support::call_traits::CallTrait,
    ) -> Result<Self, &'static str> {
        Self::for_receiver(types, instance, instance.ty(), requested)
    }
    pub(crate) fn for_receiver(
        types: &Store,
        instance: super::instance::Instance,
        receiver: TypeId,
        requested: crate::support::call_traits::CallTrait,
    ) -> Result<Self, &'static str> {
        let Some(super::Kind::Closure {
            identity,
            environment,
            ..
        }) = types.kind(instance.ty())
        else {
            return Err("receiver adapter requires a complete closure instance");
        };
        let strongest = types
            .closure_kind(identity)
            .ok_or("receiver adapter requires complete call permission")?;
        let permission = if receiver == instance.ty() {
            requested
        } else {
            let Some(super::Kind::Reference {
                referent, mutable, ..
            }) = types.kind(receiver)
            else {
                return Err("receiver adapter Self is not its closure or a supported reference");
            };
            if referent != instance.ty() {
                return Err("nested reference receiver adapters are not implemented");
            }
            requested
                .referent(mutable)
                .ok_or("mutable references do not implement shared call permission")?
        };
        let required = CallKind::from_trait(permission);
        if environment == TypeId::UNIT
            || types.contains_parameters(instance.ty()) != Some(false)
            || types.contains_parameters(receiver) != Some(false)
            || (receiver == instance.ty() && strongest == required)
            || !strongest.implements(required)
        {
            return Err("receiver adapter requires a compatible concrete captured closure");
        }
        Ok(Self {
            instance,
            receiver,
            requested,
        })
    }
    pub(crate) fn validate(self, types: &Store) -> Result<(), &'static str> {
        if Self::for_receiver(types, self.instance, self.receiver, self.requested)? != self {
            return Err("receiver adapter differs from its canonical identity");
        }
        Ok(())
    }
    #[must_use]
    pub const fn receiver(self) -> TypeId {
        self.receiver
    }
    #[must_use]
    pub const fn instance(self) -> super::instance::Instance {
        self.instance
    }
    #[must_use]
    pub const fn requested(self) -> crate::support::call_traits::CallTrait {
        self.requested
    }
}

/// The strongest receiver permission required by the body, independent of `move` capture.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum CallKind {
    Shared,
    Mutable,
    Once,
}

/// A reserved source body cannot authorize a call before its capture uses finish.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Declaration {
    Pending,
    Complete(CallKind),
}
impl CallKind {
    pub(crate) const fn from_trait(kind: crate::support::call_traits::CallTrait) -> Self {
        match kind {
            crate::support::call_traits::CallTrait::Fn => Self::Shared,
            crate::support::call_traits::CallTrait::FnMut => Self::Mutable,
            crate::support::call_traits::CallTrait::FnOnce => Self::Once,
        }
    }
    #[must_use]
    pub const fn implements(self, required: Self) -> bool {
        self as u8 <= required as u8
    }
}

impl Store {
    /// Publish immutable body call permissions before interning its canonical type.
    /// Source identity and capture places remain owned by semantic analysis.
    /// # Errors
    /// Rejects compact identity overflow.
    pub fn declare_closure(&mut self, kind: CallKind) -> Result<ClosureId, &'static str> {
        let id = self.reserve_closure()?;
        self.complete_closure(id, kind)?;
        Ok(id)
    }

    /// Reserve identity while the body's signature and capture terms are inferred.
    /// Pending identities cannot be interned as canonical closure types.
    /// # Errors
    /// Rejects compact identity overflow.
    pub fn reserve_closure(&mut self) -> Result<ClosureId, &'static str> {
        let declarations = self.closures.get_or_insert_with(Vec::new);
        let id = u32::try_from(declarations.len()).map_err(|_| "too many closure identities")?;
        declarations.push(Declaration::Pending);
        Ok(ClosureId(id))
    }

    /// Seal one body's call permission before canonical type properties can exist.
    /// # Errors
    /// Rejects foreign identities and any attempt to revise a completed declaration.
    pub fn complete_closure(&mut self, id: ClosureId, kind: CallKind) -> Result<(), &'static str> {
        let declaration = self
            .closures
            .as_mut()
            .and_then(|declarations| declarations.get_mut(id.0 as usize))
            .ok_or("missing closure declaration")?;
        if *declaration != Declaration::Pending {
            return Err("closure call permission is already complete");
        }
        *declaration = Declaration::Complete(kind);
        Ok(())
    }

    #[must_use]
    pub fn closure_declaration(&self, id: ClosureId) -> Option<Declaration> {
        self.closures.as_ref()?.get(id.0 as usize).copied()
    }

    #[must_use]
    pub fn closure_kind(&self, id: ClosureId) -> Option<CallKind> {
        match self.closure_declaration(id)? {
            Declaration::Pending => None,
            Declaration::Complete(kind) => Some(kind),
        }
    }

    pub(super) fn closure_properties(
        &self,
        arguments: GenericArgs,
        signature: TypeId,
        environment: TypeId,
    ) -> Result<Properties, &'static str> {
        let mut result = *self
            .properties
            .get(environment.0 as usize)
            .ok_or("invalid closure environment")?;
        let signature = *self
            .properties
            .get(signature.0 as usize)
            .ok_or("invalid closure signature")?;
        result.include(signature);
        self.argument_properties(arguments, &mut result)?;
        // A closure always owns sized fields. Invalid captures cannot become a DST.
        result.set(
            Property::WellFormed,
            result.has(Property::WellFormed) && result.has(Property::Sized),
        );
        result.set(Property::Sized, true);
        Ok(result)
    }
}
