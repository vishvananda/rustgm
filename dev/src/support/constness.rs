//! A qualified trait requirement keeps its effect separate from its identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Constness {
    #[default]
    Ordinary,
    Const,
    /// `[const]`: required only when the enclosing operation is constant.
    Conditional,
}

impl Constness {
    /// A premise with this effect proves a requirement with `required`: every
    /// premise proves an ordinary requirement, and an always-const premise also
    /// proves a `[const]` one, since it holds in every context.
    #[must_use]
    pub const fn proves(self, required: Self) -> bool {
        matches!(
            (self, required),
            (_, Self::Ordinary) | (Self::Const, _) | (Self::Conditional, Self::Conditional)
        )
    }
}
