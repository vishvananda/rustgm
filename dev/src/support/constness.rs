//! A qualified trait requirement keeps its effect separate from its identity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Constness {
    #[default]
    Ordinary,
    Const,
    /// `[const]`: required only when the enclosing operation is constant.
    Conditional,
}
