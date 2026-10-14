//! Callable identities retain the complete canonical argument and signature context.
use super::{Kind, Store, TypeId};

/// A selected function item, including all lifetime, type and const arguments.
/// Identity is scoped to its canonical type store; lifetime erasure is separate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Instance(TypeId);
impl Instance {
    /// # Errors
    /// Rejects non-item types and invalid callable signatures.
    pub fn new(store: &Store, ty: TypeId) -> Result<Self, &'static str> {
        let Some(Kind::FunctionItem { signature, .. }) = store.kind(ty) else {
            return Err("instance requires a selected function item");
        };
        if !matches!(store.kind(signature), Some(Kind::Function { .. })) {
            return Err("instance requires a callable signature");
        }
        Ok(Self(ty))
    }
    #[must_use]
    pub const fn ty(self) -> TypeId {
        self.0
    }
}
