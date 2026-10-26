//! Const API availability never substitutes for semantic const-call eligibility.
use super::{Draft, Inventory, Level, Policy, PolicyId};
use crate::{resolve::model::DefId, syntax::item::ItemKind};

/// Only the existing cold inventory owns this optional direct declaration axis.
pub(super) struct ConstPolicies {
    pub declarations: Vec<Option<PolicyId>>,
}
#[derive(Clone, Copy, Default)]
pub(super) enum Role {
    #[default]
    Other,
    Function(bool),
    Trait(bool),
    Impl(bool),
}
impl Role {
    pub fn of(kind: ItemKind) -> Self {
        match kind {
            ItemKind::Function { flags, .. } => Self::Function(flags.constant),
            ItemKind::Trait { constant, .. } => Self::Trait(constant),
            ItemKind::Impl {
                constant,
                trait_path: Some(_),
                ..
            } => Self::Impl(constant),
            _ => Self::Other,
        }
    }
    pub fn accepts_const(self) -> bool {
        matches!(
            self,
            Self::Function(true) | Self::Trait(true) | Self::Impl(true)
        )
    }
}
#[derive(Clone, Copy, Default)]
pub(super) struct Frame {
    pub regular: Option<PolicyId>,
    pub constant: Option<PolicyId>,
    pub role: Role,
}
impl Frame {
    pub fn inherit(&mut self, draft: &Draft, parent: Self) {
        self.regular = self.regular.or_else(|| draft.inherit(parent.regular));
        if self.constant.is_some() {
            return;
        }
        if matches!(self.role, Role::Function(true)) {
            self.constant = draft.inherit(self.regular);
        }
        if matches!(self.role, Role::Function(_))
            && matches!(parent.role, Role::Trait(true) | Role::Impl(true))
        {
            self.constant = self.constant.or_else(|| draft.inherit(parent.constant));
        }
    }
}
impl Inventory {
    /// Actual local API policy data, independently of const-call/body permission.
    #[must_use]
    pub fn const_declaration(&self, id: DefId) -> Option<&Policy> {
        let id = self
            .consts
            .as_deref()?
            .declarations
            .get(id.0 as usize)
            .copied()
            .flatten()?;
        self.policies.get(id.get() - 1)
    }
    /// Trait identity is selected by the semantic caller, never by source spelling.
    pub(crate) fn const_impl(
        &self,
        owner: DefId,
        trait_definition: DefId,
    ) -> Result<(), &'static str> {
        let actual = self.const_declaration(owner);
        if actual.is_some_and(|policy| matches!(policy.level, Level::Stable(_))) {
            return Err("const trait implementations cannot be declared const-stable");
        }
        if let Some(expected) = self.const_declaration(trait_definition)
            && expected.level.unstable() != actual.is_some_and(|policy| policy.level.unstable())
        {
            return Err("const implementation and its actual trait have different const stability");
        }
        Ok(())
    }
    fn const_safe(&self, definition: DefId) -> bool {
        let Some(policy) = self.const_declaration(definition) else {
            return false;
        };
        matches!(policy.level, Level::Stable(_))
            || self.consts.as_deref().is_some_and(|policies| {
                policies.declarations.get(definition.0 as usize)
                    == self.declarations.get(definition.0 as usize)
            })
    }
    pub(crate) fn checks_const_body(&self, definition: DefId) -> bool {
        self.const_safe(definition)
    }
    /// Local API naming is allowed. Stable and implicitly const-stable-indirect
    /// bodies must recursively avoid exposing explicitly unstable const APIs.
    pub(crate) fn const_body_call(&self, owner: DefId, target: DefId) -> Result<(), &'static str> {
        if self.const_safe(owner) && !self.const_safe(target) {
            return Err("const-stable body calls a function without const-stable API policy");
        }
        Ok(())
    }
}
