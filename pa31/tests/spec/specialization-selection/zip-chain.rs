#![feature(no_core,lang_items,rustc_attrs,fundamental,min_specialization)]
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
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for u16{fn clone(&self)->Self{*self}}impl Copy for u16{}
pub enum Option<T> { None, Some(T) }
pub trait Iterator { type Item; fn next(&mut self) -> Option<Self::Item>; }
pub struct Range<T> { pub start: T, pub end: T }
impl Iterator for Range<u8> { type Item = u8; fn next(&mut self) -> Option<u8> { loop {} } }
impl Iterator for Range<u16> { type Item = u16; fn next(&mut self) -> Option<u16> { loop {} } }
#[rustc_specialization_trait]
pub unsafe trait TrustedRandomAccessNoCoerce: Sized { fn size(&self) -> usize; }
#[rustc_specialization_trait]
pub unsafe trait TrustedRandomAccess: TrustedRandomAccessNoCoerce {}
unsafe impl TrustedRandomAccessNoCoerce for Range<u8> { fn size(&self) -> usize { loop {} } }
unsafe impl TrustedRandomAccess for Range<u8> {}
unsafe impl TrustedRandomAccessNoCoerce for Range<u16> { fn size(&self) -> usize { loop {} } }
pub struct Zip<A, B> { a: A, b: B }
trait ZipImpl<A, B> {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
    fn nth(&mut self, n: usize) -> Option<Self::Item>;
}
impl<A, B> ZipImpl<A, B> for Zip<A, B> where A: Iterator, B: Iterator {
    type Item = (A::Item, B::Item);
    default fn next(&mut self) -> Option<(A::Item, B::Item)> { loop {} }
    default fn nth(&mut self, n: usize) -> Option<Self::Item> { loop {} }
}
impl<A, B> ZipImpl<A, B> for Zip<A, B>
where A: TrustedRandomAccessNoCoerce + Iterator, B: TrustedRandomAccessNoCoerce + Iterator {
    default fn next(&mut self) -> Option<(A::Item, B::Item)> { loop {} }
    default fn nth(&mut self, n: usize) -> Option<Self::Item> { loop {} }
}
impl<A, B> ZipImpl<A, B> for Zip<A, B>
where A: TrustedRandomAccess + Iterator, B: TrustedRandomAccess + Iterator {
    fn next(&mut self) -> Option<(A::Item, B::Item)> { loop {} }
    fn nth(&mut self, n: usize) -> Option<Self::Item> { loop {} }
}
pub fn three(x: <Zip<Range<u8>, Range<u8>> as ZipImpl<Range<u8>, Range<u8>>>::Item) -> (u8, u8) { x }
pub fn two(x: <Zip<Range<u16>, Range<u16>> as ZipImpl<Range<u16>, Range<u16>>>::Item) -> (u16, u16) { x }
