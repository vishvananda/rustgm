// Authored reduction of pinned core str/pattern.rs:733,956 (88d9e12ae);
// language declarations reused from the qualified closure-input-regions packet.
// Rust MIT/Apache-2.0; see root NOTICE and third_party/sources.json.
#![feature(no_core,lang_items,rustc_attrs,fundamental,staged_api,const_trait_impl,const_destruct,unboxed_closures,tuple_trait,fn_traits,auto_traits)]
#![unstable(feature="source_fn_sugar_fixture",issue="none")]
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
#[unstable(feature="const_destruct",issue="133214")]
#[rustc_const_unstable(feature="const_destruct",issue="133214")]
#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]
pub const trait Destruct:PointeeSized{}
impl Clone for usize{fn clone(&self)->Self{*self}}impl Copy for usize{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for i32{fn clone(&self)->Self{*self}}impl Copy for i32{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}

#[unstable(feature = "tuple_trait", issue = "157987")]
#[lang = "tuple_trait"]
#[diagnostic::on_unimplemented(message = "`{Self}` is not a tuple")]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Tuple {}

#[lang = "fn"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait Fn<Args: Tuple>: [const] FnMut<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call(&self, args: Args) -> Self::Output;
}

#[lang = "fn_mut"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnMut<Args: Tuple>: FnOnce<Args> {
    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_mut(&mut self, args: Args) -> Self::Output;
}

#[lang = "fn_once"]
#[stable(feature = "rust1", since = "1.0.0")]
#[rustc_paren_sugar]
#[rustc_on_unimplemented(
    on(
        Args = "()",
        note = "wrap the `{Self}` in a closure with no arguments: `|| {{ /* code */ }}`"
    ),
    on(
        Self = "unsafe fn",
        note = "unsafe function cannot be called generically without an unsafe block",
        // SAFETY: tidy is not smart enough to tell that the below unsafe block is a string
        label = "call the function in a closure: `|| unsafe {{ /* code */ }}`"
    ),
    message = "expected an `{This:resolved}` closure, found `{Self}`",
    label = "expected an `{This:resolved}` closure, found `{Self}`"
)]
#[fundamental] // so that regex can rely that `&str: !FnMut`
#[must_use = "closures are lazy and do nothing unless called"]
#[rustc_const_unstable(feature = "const_trait_impl", issue = "143874")]
pub const trait FnOnce<Args: Tuple> {
    /// The returned type after the call operator is used.
    #[lang = "fn_once_output"]
    #[stable(feature = "fn_once_output", since = "1.12.0")]
    type Output;

    /// Performs the call operation.
    #[unstable(feature = "fn_traits", issue = "29625")]
    extern "rust-call" fn call_once(self, args: Args) -> Self::Output;
}

#[lang = "freeze"]
#[unstable(feature = "freeze", issue = "121675")]
pub unsafe auto trait Freeze {}
impl<T:PointeeSized> Clone for &T {fn clone(&self)->Self{*self}}
impl<T:PointeeSized> Copy for &T {}
macro_rules! pattern_methods {
    ($life:lifetime,$searcher:ty,$pmap:expr,$smap:expr) => {
        type Searcher<$life> = $searcher;
        fn into_searcher<$life>(self,haystack:&$life u8)->$searcher {
            ($smap)(($pmap)(self).into_searcher(haystack))
        }
        fn is_contained_in<$life>(self,haystack:&$life u8)->bool {
            ($pmap)(self).is_contained_in(haystack)
        }
    };
}
pub struct ByteSearcher<'a,'b>{pub haystack:&'a u8,pub needle:&'b u8}
pub trait Pattern {
    type Searcher<'a>;
    fn into_searcher<'a>(self,haystack:&'a u8)->Self::Searcher<'a>;
    fn is_contained_in<'a>(self,haystack:&'a u8)->bool;
}
unsafe extern "C" {fn equal_u8(haystack:&u8,needle:&u8)->bool;}
impl<'b> Pattern for &'b u8 {
    type Searcher<'a> = ByteSearcher<'a,'b>;
    fn into_searcher<'a>(self,haystack:&'a u8)->Self::Searcher<'a>{ByteSearcher{haystack,needle:self}}
    fn is_contained_in<'a>(self,haystack:&'a u8)->bool{unsafe{equal_u8(haystack,self)}}
}
impl<'b,'c> Pattern for &'c &'b u8 {
    pattern_methods!('a,ByteSearcher<'a,'b>,|&s|s,|s|s);
}
#[unsafe(no_mangle)]pub extern "C" fn search_needle<'b,'c>(needle:&'c &'b u8,haystack:&u8)->u8{*needle.into_searcher(haystack).needle}
#[unsafe(no_mangle)]pub extern "C" fn search_haystack<'b,'c>(needle:&'c &'b u8,haystack:&u8)->u8{*needle.into_searcher(haystack).haystack}
#[unsafe(no_mangle)]pub extern "C" fn contains<'b,'c>(needle:&'c &'b u8,haystack:&u8)->bool{needle.is_contained_in(haystack)}
