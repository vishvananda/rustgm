#[lang="freeze"] pub unsafe auto trait Freeze {}
#[lang="future_trait"]
pub trait Future {
    #[lang="future_output"]
    type Output;
}
#[lang="Option"] pub enum Option<T> {
    #[lang="None"] None,
    #[lang="Some"] Some(T),
}
use Option::{None, Some};
#[lang="destruct"]
#[rustc_const_unstable(feature="const_destruct", issue="133214")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
// Original pinned Poll enum.
// Original pinned async generator helpers.
