pub trait Try {
    type Residual;
}
pub trait Residual<V> {
    type TryType;
}
impl Try for u8 {
    type Residual = u16;
}
impl Residual<u8> for u16 {
    type TryType = u32;
}
// Reduced from the two projections in the pinned Rust 1.98.0 core/src/ops/try_trait.rs.
#[allow(type_alias_bounds)]
pub(crate) type ChangeOutputType<T: Try<Residual: Residual<V>>, V> =
    <T::Residual as Residual<V>>::TryType;
#[allow(type_alias_bounds)]
pub type Exported<T: Try<Residual: Residual<V>>, V> = <T::Residual as Residual<V>>::TryType;

pub fn provider_value() -> ChangeOutputType<u8, u8> {
    34
}
