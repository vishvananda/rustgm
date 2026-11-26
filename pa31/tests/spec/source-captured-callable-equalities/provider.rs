#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
}

impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}

pub trait Visit {
    fn visit(&self, f: impl FnMut(u32)) -> &Self;
}
impl Visit for u32 {
    fn visit(&self, mut f: impl FnMut(u32)) -> &Self { f(17); self }
}
pub fn direct<F: FnMut(u32)>(x: &u32, mut f: F) -> &u32 {
    f(17);
    x
}
pub fn free_control(x: &u32) -> (u32, u32) {
    let mut total = 0;
    let r = direct(x, |digit| { total = digit; });
    (*r, total)
}
pub fn nested(slot: &mut &u32) -> u32 {
    let mut total = 0;
    let source = *slot;
    *slot = source.visit(|digit| { total = digit; });
    total
}
pub fn choose<'a>(slot: &mut &'a u32, other: &'a u32, keep: bool) -> (u32, u32) {
    let mut total = 0;
    let source = if keep { *slot } else { other };
    *slot = source.visit(|digit| { total = digit; });
    (total, **slot)
}
