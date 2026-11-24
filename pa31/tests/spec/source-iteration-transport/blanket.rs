// The exact blanket shape follows pinned Rust core/src/iter/traits/collect.rs.
#[lang="destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang="drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T) where T: [const] Destruct {}

const impl<I: [const] Iterator> IntoIterator for I {
    type Item = I::Item;
    type IntoIter = I;

    #[inline]
    fn into_iter(self) -> I {
        self
    }
}
