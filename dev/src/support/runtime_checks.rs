//! Immutable compilation settings and body-owned decisions precede code generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct RuntimeCheckConfig {
    ub_checks: bool,
}
impl RuntimeCheckConfig {
    /// The course profile disables debug assertions and therefore UB checks by default.
    #[must_use]
    pub const fn with_ub_checks(ub_checks: bool) -> Self {
        Self { ub_checks }
    }
    #[must_use]
    pub const fn ub_checks(self) -> bool {
        self.ub_checks
    }
}

/// A deferred decision is data; executing a call still requires its ordinary proof.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum RuntimeCheckBinding {
    Frozen(bool),
    Deferred,
}
impl RuntimeCheckBinding {
    #[must_use]
    pub const fn for_crate(preserve: bool, producer: RuntimeCheckConfig) -> Self {
        if preserve {
            Self::Deferred
        } else {
            Self::Frozen(producer.ub_checks())
        }
    }
    #[must_use]
    pub const fn resolve(self, generation: RuntimeCheckConfig) -> bool {
        match self {
            Self::Frozen(value) => value,
            Self::Deferred => generation.ub_checks(),
        }
    }
}
