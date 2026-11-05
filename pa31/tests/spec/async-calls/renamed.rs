#![feature(prelude_import)]
#![feature(const_trait_impl,const_destruct)]
#![feature(auto_traits)]
#![feature(intrinsics)]
#![feature(unboxed_closures)]
#![feature(no_core,lang_items,rustc_attrs,staged_api)]
#![no_core]
#![unstable(feature="receiver_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
pub mod prelude{pub use crate::Sized;}
#[prelude_import]use crate::prelude::*;
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized> LegacyReceiver for &T{}
impl<T:PointeeSized> LegacyReceiver for &mut T{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u32{fn clone(&self)->Self{*self}}impl Copy for u32{}
impl<T:PointeeSized> Clone for *const T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *const T{}
impl<T:PointeeSized> Clone for *mut T{fn clone(&self)->Self{*self}}impl<T:PointeeSized> Copy for *mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_intrinsic]pub const fn wrapping_add<T:Copy>(a:T,b:T)->T;
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="drop_glue"]#[rustc_const_unstable(feature="const_destruct",issue="none")]pub const unsafe fn drop_glue<T:PointeeSized>(_:&mut T)where T:[const]Destruct{}
pub mod marker{#[lang="tuple_trait"]pub trait Tuple{}}
pub mod future{#[lang="future_trait"]pub trait Future{#[lang="future_output"]type Output;}}
use crate::future::Future;
use crate::marker::Tuple;

/// An async-aware version of the [`Fn`](crate::ops::Fn) trait.
///
/// All `async fn` and functions returning futures implement this trait.
#[stable(feature = "async_closure", since = "1.85.0")]
#[rustc_paren_sugar]
#[must_use = "async closures are lazy and do nothing unless called"]
#[lang = "async_fn"]
pub trait SharedCall<Args: Tuple>: MutableCall<Args> {
    /// Call the [`SharedCall`], returning a future which may borrow from the called closure.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    extern "rust-call" fn async_call(&self, args: Args) -> Self::CallRefFuture<'_>;
}

/// An async-aware version of the [`FnMut`](crate::ops::FnMut) trait.
///
/// All `async fn` and functions returning futures implement this trait.
#[stable(feature = "async_closure", since = "1.85.0")]
#[rustc_paren_sugar]
#[must_use = "async closures are lazy and do nothing unless called"]
#[lang = "async_fn_mut"]
pub trait MutableCall<Args: Tuple>: OwnedCall<Args> {
    /// Future returned by [`MutableCall::async_call_mut`] and [`SharedCall::async_call`].
    #[unstable(feature = "async_fn_traits", issue = "none")]
    #[lang = "call_ref_future"]
    type CallRefFuture<'a>: Future<Output = Self::Output>
    where
        Self: 'a;

    /// Call the [`MutableCall`], returning a future which may borrow from the called closure.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    extern "rust-call" fn async_call_mut(&mut self, args: Args) -> Self::CallRefFuture<'_>;
}

/// An async-aware version of the [`FnOnce`](crate::ops::FnOnce) trait.
///
/// All `async fn` and functions returning futures implement this trait.
#[stable(feature = "async_closure", since = "1.85.0")]
#[rustc_paren_sugar]
#[must_use = "async closures are lazy and do nothing unless called"]
#[lang = "async_fn_once"]
pub trait OwnedCall<Args: Tuple> {
    /// Future returned by [`OwnedCall::async_call_once`].
    #[unstable(feature = "async_fn_traits", issue = "none")]
    #[lang = "call_once_future"]
    type CallOnceFuture: Future<Output = Self::Output>;

    /// Output type of the called closure's future.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    #[lang = "async_fn_once_output"]
    type Output;

    /// Call the [`OwnedCall`], returning a future which may move out of the called closure.
    #[unstable(feature = "async_fn_traits", issue = "none")]
    extern "rust-call" fn async_call_once(self, args: Args) -> Self::CallOnceFuture;
}

mod impls {
    use super::{SharedCall, MutableCall, OwnedCall};
    use crate::marker::Tuple;

    #[stable(feature = "async_closure", since = "1.85.0")]
    impl<A: Tuple, F: ?Sized> SharedCall<A> for &F
    where
        F: SharedCall<A>,
    {
        extern "rust-call" fn async_call(&self, args: A) -> Self::CallRefFuture<'_> {
            F::async_call(*self, args)
        }
    }

    #[stable(feature = "async_closure", since = "1.85.0")]
    impl<A: Tuple, F: ?Sized> MutableCall<A> for &F
    where
        F: SharedCall<A>,
    {
        type CallRefFuture<'a>
            = F::CallRefFuture<'a>
        where
            Self: 'a;

        extern "rust-call" fn async_call_mut(&mut self, args: A) -> Self::CallRefFuture<'_> {
            F::async_call(*self, args)
        }
    }

    #[stable(feature = "async_closure", since = "1.85.0")]
    impl<'a, A: Tuple, F: ?Sized> OwnedCall<A> for &'a F
    where
        F: SharedCall<A>,
    {
        type Output = F::Output;
        type CallOnceFuture = F::CallRefFuture<'a>;

        extern "rust-call" fn async_call_once(self, args: A) -> Self::CallOnceFuture {
            F::async_call(self, args)
        }
    }

    #[stable(feature = "async_closure", since = "1.85.0")]
    impl<A: Tuple, F: ?Sized> MutableCall<A> for &mut F
    where
        F: MutableCall<A>,
    {
        type CallRefFuture<'a>
            = F::CallRefFuture<'a>
        where
            Self: 'a;

        extern "rust-call" fn async_call_mut(&mut self, args: A) -> Self::CallRefFuture<'_> {
            F::async_call_mut(*self, args)
        }
    }

    #[stable(feature = "async_closure", since = "1.85.0")]
    impl<'a, A: Tuple, F: ?Sized> OwnedCall<A> for &'a mut F
    where
        F: MutableCall<A>,
    {
        type Output = F::Output;
        type CallOnceFuture = F::CallRefFuture<'a>;

        extern "rust-call" fn async_call_once(self, args: A) -> Self::CallOnceFuture {
            F::async_call_mut(self, args)
        }
    }
}

mod internal_implementation_detail {
    /// A helper trait that is used to enforce that the `ClosureKind` of a goal
    /// is within the capabilities of a `CoroutineClosure`, and which allows us
    /// to delay the projection of the tupled upvar types until after upvar
    /// analysis is complete.
    ///
    /// The `Self` type is expected to be the `kind_ty` of the coroutine-closure,
    /// and thus either `?0` or `i8`/`i16`/`i32` (see docs for `ClosureKind`
    /// for an explanation of that). The `GoalKind` is also the same type, but
    /// representing the kind of the trait that the closure is being called with.
    #[lang = "async_fn_kind_helper"]
    trait SharedCallKindHelper<GoalKind> {
        // Projects a set of closure inputs (arguments), a region, and a set of upvars
        // (by move and by ref) to the upvars that we expect the coroutine to have
        // according to the `GoalKind` parameter above.
        //
        // The `Upvars` parameter should be the upvars of the parent coroutine-closure,
        // and the `BorrowedUpvarsAsFnPtr` will be a function pointer that has the shape
        // `for<'env> fn() -> (&'env T, ...)`. This allows us to represent the binder
        // of the closure's self-capture, and these upvar types will be instantiated with
        // the `'closure_env` region provided to the associated type.
        #[lang = "async_fn_kind_upvars"]
        type Upvars<'closure_env, Inputs, Upvars, BorrowedUpvarsAsFnPtr>;
    }
}
pub struct Ready{pub value:u32}impl Future for Ready{type Output=u32;}
pub struct Task{value:u32}
impl OwnedCall<(u32,)> for Task{type Output=u32;type CallOnceFuture=Ready;extern "rust-call" fn async_call_once(self,args:(u32,))->Ready{Ready{value:wrapping_add(self.value,args.0)}}}
impl MutableCall<(u32,)> for Task{type CallRefFuture<'a>=Ready where Self:'a;extern "rust-call" fn async_call_mut(&mut self,args:(u32,))->Ready{Ready{value:wrapping_add(self.value,args.0)}}}
impl SharedCall<(u32,)> for Task{extern "rust-call" fn async_call(&self,args:(u32,))->Ready{Ready{value:wrapping_add(self.value,args.0)}}}
#[unsafe(no_mangle)]pub extern "C" fn once()->u32{OwnedCall::async_call_once(Task{value:10},(13,)).value}
#[unsafe(no_mangle)]pub extern "C" fn shared()->u32{SharedCall::async_call(&Task{value:10},(7,)).value}
#[unsafe(no_mangle)]pub extern "C" fn mutable()->u32{MutableCall::async_call_mut(&mut Task{value:10},(1,)).value}
#[unsafe(no_mangle)]pub extern "C" fn forwarded()->u32{<&Task as OwnedCall<(u32,)>>::async_call_once(&Task{value:10},(19,)).value}
#[unsafe(no_mangle)]pub extern "C" fn unique()->u32{<&mut Task as OwnedCall<(u32,)>>::async_call_once(&mut Task{value:10},(21,)).value}
