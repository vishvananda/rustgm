#![feature(doc_notable_trait,rustdoc_internals)]
#![feature(no_core,lang_items,rustc_attrs,staged_api,auto_traits,const_trait_impl,prelude_import)]
#![no_core]
#![unstable(feature="async_iterator_fixture",issue="none")]
#![allow(internal_features,dead_code,unused_variables)]
#[lang="pointee_sized"]pub trait PointeeSized{}
#[lang="meta_sized"]pub trait MetaSized:PointeeSized{}
#[lang="sized"]pub trait Sized:MetaSized{}
#[lang="legacy_receiver"]pub trait LegacyReceiver:PointeeSized{}
impl<T:PointeeSized>LegacyReceiver for &T{}
impl<T:PointeeSized>LegacyReceiver for &mut T{}
#[lang="freeze"]pub unsafe auto trait Freeze{}
#[rustc_const_unstable(feature="const_destruct",issue="133214")]#[lang="destruct"]#[rustc_deny_explicit_impl]#[rustc_dyn_incompatible_trait]pub const trait Destruct:PointeeSized{}
#[lang="unpin"]pub auto trait Unpin{}
#[lang="clone"]pub trait Clone:Sized{#[lang="clone_fn"]fn clone(&self)->Self;}
#[lang="copy"]pub trait Copy:Clone{}
impl Clone for u8{fn clone(&self)->Self{*self}}impl Copy for u8{}
impl Clone for bool{fn clone(&self)->Self{*self}}impl Copy for bool{}
pub enum Option<T>{None,Some(T)}
pub mod prelude{pub use crate::{Sized,Unpin,Option,Option::{Some,None}};}
#[prelude_import]use crate::prelude::*;
pub mod ops{
#[lang="deref"]pub trait Deref:PointeeSized{#[lang="deref_target"]type Target:PointeeSized;fn deref(&self)->&Self::Target;}
#[lang="deref_mut"]pub trait DerefMut:Deref{fn deref_mut(&mut self)->&mut Self::Target;}
use crate::{PointeeSized,LegacyReceiver};
impl<T:PointeeSized>Deref for &T{type Target=T;fn deref(&self)->&T{*self}}
impl<T:PointeeSized>Deref for &mut T{type Target=T;fn deref(&self)->&T{&**self}}
impl<T:PointeeSized>DerefMut for &mut T{fn deref_mut(&mut self)->&mut T{&mut **self}}
}
pub mod pin{
use crate::{ops::{Deref,DerefMut},PointeeSized,Unpin,LegacyReceiver};
#[lang="pin"]#[repr(transparent)]pub struct Pin<P>{pub pointer:P}
impl<P>Pin<P>{#[lang="new_unchecked"]pub const unsafe fn new_unchecked(pointer:P)->Self{Self{pointer}}}
impl<P:Deref>Pin<P>{pub fn new(pointer:P)->Self where P::Target:Unpin{Self{pointer}}}
impl<P:Deref>Deref for Pin<P>{type Target=P::Target;fn deref(&self)->&Self::Target{self.pointer.deref()}}
impl<P:DerefMut>DerefMut for Pin<P> where P::Target:Unpin{fn deref_mut(&mut self)->&mut Self::Target{self.pointer.deref_mut()}}
impl<P:DerefMut>Pin<P>{pub fn as_deref_mut(self:Pin<&mut Self>)->Pin<&mut P::Target>{unsafe{Pin::new_unchecked(self.pointer.pointer.deref_mut())}}}
impl<P:Deref>LegacyReceiver for Pin<P>{}
}
pub mod task{
#[lang="Poll"]pub enum Poll<T>{#[lang="Ready"]Ready(T),#[lang="Pending"]Pending}
#[lang="Context"]pub struct Context<'a>{pub token:&'a ()}
}

pub mod future {
#![stable(feature = "futures_api", since = "1.36.0")]

use crate::ops;
use crate::pin::Pin;
use crate::task::{Context, Poll};

/// A future represents an asynchronous computation, commonly obtained by use of
/// [`async`].
///
/// A future is a value that might not have finished computing yet. This kind of
/// "asynchronous value" makes it possible for a thread to continue doing useful
/// work while it waits for the value to become available.
///
/// # The `poll` method
///
/// The core method of future, `poll`, *attempts* to resolve the future into a
/// final value. This method does not block if the value is not ready. Instead,
/// the current task is scheduled to be woken up when it's possible to make
/// further progress by `poll`ing again. The `context` passed to the `poll`
/// method can provide a [`Waker`], which is a handle for waking up the current
/// task.
///
/// When using a future, you generally won't call `poll` directly, but instead
/// `.await` the value.
///
/// [`async`]: ../../std/keyword.async.html
/// [`Waker`]: crate::task::Waker
#[doc(notable_trait)]
#[doc(search_unbox)]
#[must_use = "futures do nothing unless you `.await` or poll them"]
#[stable(feature = "futures_api", since = "1.36.0")]
#[lang = "future_trait"]
#[diagnostic::on_unimplemented(
    label = "`{Self}` is not a future",
    message = "`{Self}` is not a future"
)]
pub trait Future {
    /// The type of value produced on completion.
    #[stable(feature = "futures_api", since = "1.36.0")]
    #[lang = "future_output"]
    type Output;

    /// Attempts to resolve the future to a final value, registering
    /// the current task for wakeup if the value is not yet available.
    ///
    /// # Return value
    ///
    /// This function returns:
    ///
    /// - [`Poll::Pending`] if the future is not ready yet
    /// - [`Poll::Ready(val)`] with the result `val` of this future if it
    ///   finished successfully.
    ///
    /// Once a future has finished, clients should not `poll` it again.
    ///
    /// When a future is not ready yet, `poll` returns `Poll::Pending` and
    /// stores a clone of the [`Waker`] copied from the current [`Context`].
    /// This [`Waker`] is then woken once the future can make progress.
    /// For example, a future waiting for a socket to become
    /// readable would call `.clone()` on the [`Waker`] and store it.
    /// When a signal arrives elsewhere indicating that the socket is readable,
    /// [`Waker::wake`] is called and the socket future's task is awoken.
    /// Once a task has been woken up, it should attempt to `poll` the future
    /// again, which may or may not produce a final value.
    ///
    /// Note that on multiple calls to `poll`, only the [`Waker`] from the
    /// [`Context`] passed to the most recent call should be scheduled to
    /// receive a wakeup.
    ///
    /// # Runtime characteristics
    ///
    /// Futures alone are *inert*; they must be *actively* `poll`ed for the
    /// underlying computation to make progress, meaning that each time the
    /// current task is woken up, it should actively re-`poll` pending futures
    /// that it still has an interest in.
    ///
    /// Having said that, some Futures may represent a value that is being
    /// computed in a different task. In this case, the future's underlying
    /// computation is simply acting as a conduit for a value being computed
    /// by that other task, which will proceed independently of the Future.
    /// Futures of this kind are typically obtained when spawning a new task into an
    /// async runtime.
    ///
    /// The `poll` function should not be called repeatedly in a tight loop --
    /// instead, it should only be called when the future indicates that it is
    /// ready to make progress (by calling `wake()`). If you're familiar with the
    /// `poll(2)` or `select(2)` syscalls on Unix it's worth noting that futures
    /// typically do *not* suffer the same problems of "all wakeups must poll
    /// all events"; they are more like `epoll(4)`.
    ///
    /// An implementation of `poll` should strive to return quickly, and should
    /// not block. Returning quickly prevents unnecessarily clogging up
    /// threads or event loops. If it is known ahead of time that a call to
    /// `poll` may end up taking a while, the work should be offloaded to a
    /// thread pool (or something similar) to ensure that `poll` can return
    /// quickly.
    ///
    /// # Panics
    ///
    /// Once a future has completed (returned `Ready` from `poll`), calling its
    /// `poll` method again may panic, block forever, or cause other kinds of
    /// problems; the `Future` trait places no requirements on the effects of
    /// such a call. However, as the `poll` method is not marked `unsafe`,
    /// Rust's usual rules apply: calls must never cause undefined behavior
    /// (memory corruption, incorrect use of `unsafe` functions, or the like),
    /// regardless of the future's state.
    ///
    /// [`Poll::Ready(val)`]: Poll::Ready
    /// [`Waker`]: crate::task::Waker
    /// [`Waker::wake`]: crate::task::Waker::wake
    #[lang = "poll"]
    #[stable(feature = "futures_api", since = "1.36.0")]
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}

#[stable(feature = "futures_api", since = "1.36.0")]
impl<F: ?Sized + Future + Unpin> Future for &mut F {
    type Output = F::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        F::poll(Pin::new(&mut **self), cx)
    }
}


}
pub mod conversion {
use crate::future::Future;

/// Conversion into a `Future`.
///
/// By implementing `IntoFuture` for a type, you define how it will be
/// converted to a future.
///
/// # `.await` desugaring
///
/// The `.await` keyword desugars into a call to `IntoFuture::into_future`
/// first before polling the future to completion. `IntoFuture` is implemented
/// for all `T: Future` which means the `into_future` method will be available
/// on all futures.
///
/// ```no_run
/// use std::future::IntoFuture;
///
/// # async fn foo() {
/// let v = async { "meow" };
/// let mut fut = v.into_future();
/// assert_eq!("meow", fut.await);
/// # }
/// ```
///
/// # Async builders
///
/// When implementing futures manually there will often be a choice between
/// implementing `Future` or `IntoFuture` for a type. Implementing `Future` is a
/// good choice in most cases. But implementing `IntoFuture` is most useful when
/// implementing "async builder" types, which allow their values to be modified
/// multiple times before being `.await`ed.
///
/// ```rust
/// use std::future::{ready, Ready, IntoFuture};
///
/// /// Eventually multiply two numbers
/// pub struct Multiply {
///     num: u16,
///     factor: u16,
/// }
///
/// impl Multiply {
///     /// Constructs a new instance of `Multiply`.
///     pub fn new(num: u16, factor: u16) -> Self {
///         Self { num, factor }
///     }
///
///     /// Set the number to multiply by the factor.
///     pub fn number(mut self, num: u16) -> Self {
///         self.num = num;
///         self
///     }
///
///     /// Set the factor to multiply the number with.
///     pub fn factor(mut self, factor: u16) -> Self {
///         self.factor = factor;
///         self
///     }
/// }
///
/// impl IntoFuture for Multiply {
///     type Output = u16;
///     type IntoFuture = Ready<Self::Output>;
///
///     fn into_future(self) -> Self::IntoFuture {
///         ready(self.num * self.factor)
///     }
/// }
///
/// // NOTE: Rust does not yet have an `async fn main` function, that functionality
/// // currently only exists in the ecosystem.
/// async fn run() {
///     let num = Multiply::new(0, 0)  // initialize the builder to number: 0, factor: 0
///         .number(2)                 // change the number to 2
///         .factor(2)                 // change the factor to 2
///         .await;                    // convert to future and .await
///
///     assert_eq!(num, 4);
/// }
/// ```
///
/// # Usage in trait bounds
///
/// Using `IntoFuture` in trait bounds allows a function to be generic over both
/// `Future` and `IntoFuture`. This is convenient for users of the function, so
/// when they are using it they don't have to make an extra call to
/// `IntoFuture::into_future` to obtain an instance of `Future`:
///
/// ```rust
/// use std::future::IntoFuture;
///
/// /// Converts the output of a future to a string.
/// async fn fut_to_string<Fut>(fut: Fut) -> String
/// where
///     Fut: IntoFuture,
///     Fut::Output: std::fmt::Debug,
/// {
///     format!("{:?}", fut.await)
/// }
/// ```
#[stable(feature = "into_future", since = "1.64.0")]
#[rustc_diagnostic_item = "IntoFuture"]
#[diagnostic::on_unimplemented(
    label = "`{Self}` is not a future",
    message = "`{Self}` is not a future",
    note = "{Self} must be a future or must implement `IntoFuture` to be awaited"
)]
pub trait IntoFuture {
    /// The output that the future will produce on completion.
    #[stable(feature = "into_future", since = "1.64.0")]
    type Output;

    /// Which kind of future are we turning this into?
    #[stable(feature = "into_future", since = "1.64.0")]
    type IntoFuture: Future<Output = Self::Output>;

    /// Creates a future from a value.
    ///
    /// # Examples
    ///
    /// Basic usage:
    ///
    /// ```no_run
    /// use std::future::IntoFuture;
    ///
    /// # async fn foo() {
    /// let v = async { "meow" };
    /// let mut fut = v.into_future();
    /// assert_eq!("meow", fut.await);
    /// # }
    /// ```
    #[stable(feature = "into_future", since = "1.64.0")]
    #[lang = "into_future"]
    fn into_future(self) -> Self::IntoFuture;
}

#[stable(feature = "into_future", since = "1.64.0")]
impl<F: Future> IntoFuture for F {
    type Output = F::Output;
    type IntoFuture = F;

    fn into_future(self) -> Self::IntoFuture {
        self
    }
}

}
use crate::future::Future;use crate::conversion::IntoFuture;use crate::pin::Pin;use crate::task::{Context,Poll};

trait A{type Out;}trait B:A{}trait F{type Out;fn get()->Self::Out;}struct W<P>(P);impl<P:B<Out:F>>F for W<P>{type Out=<<P as A>::Out as F>::Out;fn get()->Self::Out{loop{}}}fn main(){}