#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for u16 { fn clone(&self) -> Self { *self } }
impl Copy for u16 {}
impl Clone for u64 { fn clone(&self) -> Self { *self } }
impl Copy for u64 {}
impl Clone for f16 { fn clone(&self) -> Self { *self } }
impl Copy for f16 {}
impl Clone for f128 { fn clone(&self) -> Self { *self } }
impl Copy for f128 {}

#[lang = "neg"]
pub trait Neg {
    type Output;
    fn neg(self) -> Self::Output;
}
impl Neg for f16 {
    type Output = Self;
    fn neg(self) -> Self { -self }
}
impl Neg for f128 {
    type Output = Self;
    fn neg(self) -> Self { -self }
}

#[cfg(not(older_profile))]
#[rustc_intrinsic]
pub const unsafe fn transmute_unchecked<Src, Dst>(src: Src) -> Dst;

pub fn half_low() -> f16 { 1.0009765625f16 }
pub fn half_tiny() -> f16 { 0.000000059604644775390625f16 }
pub fn half_negative_zero() -> f16 { -0.0f16 }

pub fn quad_low() -> f128 { 1.0000000000000000000000000000000001925929944387235853055977942584927318538101648215388195239938795566558837890625f128 }
pub fn quad_pi() -> f128 { 3.14159265358979323846264338327950288419716939937510582097494f128 }
pub fn quad_negative_zero() -> f128 { -0.0f128 }

pub const fn half_generic<T>() -> f16 { 1.5f16 }
pub const fn quad_generic<T>() -> f128 { 1.0000000000000000000000000000000001925929944387235853055977942584927318538101648215388195239938795566558837890625f128 }
