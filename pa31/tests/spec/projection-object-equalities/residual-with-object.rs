#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,auto_traits)]
#![unstable(feature="projection_equation_fixture",issue="none")]
#![no_core]
#![allow(internal_features,dead_code,unused_variables)]
#[lang = "pointee_sized"]
#[diagnostic::on_unimplemented(
    message = "values of type `{Self}` may or may not have a size",
    label = "may or may not have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_coinductive]
pub trait PointeeSized {
    // Empty
}
#[lang = "meta_sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known",
    label = "doesn't have a known size"
)]
#[fundamental]
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
// `MetaSized` being coinductive, despite having supertraits, is okay for the same reasons as
// `Sized` above.
#[rustc_coinductive]
pub trait MetaSized: PointeeSized {
    // Empty
}
#[lang = "sized"]
#[diagnostic::on_unimplemented(
    message = "the size for values of type `{Self}` cannot be known at compilation time",
    label = "doesn't have a size known at compile-time"
)]
#[fundamental] // for Default, for example, which requires that `[T]: !Default` be evaluatable
#[rustc_specialization_trait]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
// `Sized` being coinductive, despite having supertraits, is okay as there are no user-written impls,
// and we know that the supertraits are always implemented if the subtrait is just by looking at
// the builtin impls.
#[rustc_coinductive]
pub trait Sized: MetaSized {
    // Empty.
}
#[lang = "legacy_receiver"]
pub trait LegacyReceiver: PointeeSized {}
impl<T: PointeeSized> LegacyReceiver for &T {}
impl<T: PointeeSized> LegacyReceiver for &mut T {}
#[lang = "clone"]
pub trait Clone: Sized { #[lang = "clone_fn"] fn clone(&self) -> Self; }
#[lang = "copy"]
pub trait Copy: Clone {}
#[lang = "freeze"]
unsafe auto trait Freeze {}
#[lang = "destruct"]
const trait Destruct: PointeeSized {}
const trait FromResidual<R = <Self as Try>::Residual> { fn from_residual(residual: R) -> Self; }
const trait Try: [const] FromResidual { type Output; type Residual: Residual<Self::Output>; }
const trait Residual<O>: Sized { type TryType: [const] Try<Output = O, Residual = Self>; }
type ChangeOutputType<T, V> = <<T as Try>::Residual as Residual<V>>::TryType;
trait Object {}
fn object(_: &dyn Object) {}

const fn change<R, const N: usize>(r: R) -> ChangeOutputType<R, [R::Output; N]>
where R: [const] Try<Residual: [const] Residual<[R::Output; N]>> { loop {} }
