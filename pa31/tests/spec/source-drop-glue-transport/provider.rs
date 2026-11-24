#[lang = "drop"]
pub const trait Drop {
    fn drop(&mut self);
}

#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

impl Clone for u8 { fn clone(&self) -> Self { *self } }
impl Copy for u8 {}

// Reduced from the pinned core/src/ptr/mod.rs.
#[lang = "drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
    // Code here does not matter - this is replaced by the
    // real drop glue by the compiler.
}

pub const unsafe fn ordinary<T: PointeeSized>(_: &mut T) {}
