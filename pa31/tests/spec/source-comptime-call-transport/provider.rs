#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for u32 {
    fn clone(&self) -> Self { *self }
}
impl Copy for u32 {}

pub const fn seed(x: u32) -> u32 { x }

const fn generic_seed<const N: u32>() -> u32 { N }

#[rustc_comptime]
pub fn chained() -> u32 { seed(19) }

#[rustc_comptime]
pub fn forwarded() -> u32 { chained() }

#[rustc_comptime]
pub fn generic<const N: u32>() -> u32 { generic_seed::<N>() }

pub struct Wrap(pub u32);

impl Wrap {
    #[rustc_comptime]
    pub fn value(&self) -> u32 { seed(self.0) }
}
