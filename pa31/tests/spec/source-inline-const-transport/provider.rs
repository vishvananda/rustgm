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

pub const fn ordinary() -> u32 {
    const { 42 }
}

pub const fn generic<const N: u32>() -> u32 {
    const { N }
}

pub fn in_closure<const N: u32>() -> u32 {
    let value = || const { N };
    value()
}

pub const fn nested() -> u32 {
    const { const { 9 } }
}

pub const VALUE: u32 = const { 23 };

#[cfg(not(older_profile))]
#[rustc_nounwind]
#[rustc_intrinsic_const_stable_indirect]
#[rustc_intrinsic]
pub const fn size_of<T>() -> usize;

#[cfg(not(older_profile))]
pub const fn for_type<T>() -> u32 {
    const { size_of::<T>() as u32 }
}
