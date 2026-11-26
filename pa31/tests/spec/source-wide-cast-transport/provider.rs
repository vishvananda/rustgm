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


pub const fn half_to_single(value: f16) -> f32 { value as f32 }
pub const fn half_to_double(value: f16) -> f64 { value as f64 }
pub const fn single_to_half(value: f32) -> f16 { value as f16 }
pub const fn double_to_half(value: f64) -> f16 { value as f16 }
pub const fn quad_to_single(value: f128) -> f32 { value as f32 }
pub const fn quad_to_double(value: f128) -> f64 { value as f64 }
pub const fn double_to_quad(value: f64) -> f128 { value as f128 }
pub const fn half_to_quad(value: f16) -> f128 { value as f128 }
pub const fn quad_to_half(value: f128) -> f16 { value as f16 }
pub const fn quad_to_int(value: f128) -> i32 { value as i32 }
pub const fn half_to_byte(value: f16) -> u8 { value as u8 }
pub const fn integer_to_half(value: u32) -> f16 { value as f16 }
pub const fn integer_to_quad(value: u128) -> f128 { value as f128 }
pub const fn quad_to_integer(value: f128) -> u128 { value as u128 }
pub const fn quad_to_byte(value: f128) -> u8 { value as u8 }
pub const fn integer_to_half_signed(value: i128) -> f16 { value as f16 }
pub const fn quad_to_signed(value: f128) -> i128 { value as i128 }
pub const fn generic_half<T>(value: f16) -> f32 { value as f32 }
