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

use crate::ops::DerefMut;
use crate::pin::Pin;
use crate::task::{Context, Poll};

/// A trait for dealing with asynchronous iterators.
///
/// This is the main async iterator trait. For more about the concept of async iterators
/// generally, please see the [module-level documentation]. In particular, you
/// may want to know how to [implement `AsyncIterator`][impl].
///
/// [module-level documentation]: index.html
/// [impl]: index.html#implementing-async-iterator
#[unstable(feature = "async_iterator", issue = "79024")]
#[must_use = "async iterators do nothing unless polled"]
#[doc(alias = "Stream")]
#[lang = "async_iterator"]
pub trait AsyncIterator {
    /// The type of items yielded by the async iterator.
    type Item;

    /// Attempts to pull out the next value of this async iterator, registering the
    /// current task for wakeup if the value is not yet available, and returning
    /// `None` if the async iterator is exhausted.
    ///
    /// # Return value
    ///
    /// There are several possible return values, each indicating a distinct
    /// async iterator state:
    ///
    /// - `Poll::Pending` means that this async iterator's next value is not ready
    ///   yet. Implementations will ensure that the current task will be notified
    ///   when the next value may be ready.
    ///
    /// - `Poll::Ready(Some(val))` means that the async iterator has successfully
    ///   produced a value, `val`, and may produce further values on subsequent
    ///   `poll_next` calls.
    ///
    /// - `Poll::Ready(None)` means that the async iterator has terminated, and
    ///   `poll_next` should not be invoked again.
    ///
    /// # Panics
    ///
    /// Once an async iterator has finished (returned `Ready(None)` from `poll_next`), calling its
    /// `poll_next` method again may panic, block forever, or cause other kinds of
    /// problems; the `AsyncIterator` trait places no requirements on the effects of
    /// such a call. However, as the `poll_next` method is not marked `unsafe`,
    /// Rust's usual rules apply: calls must never cause undefined behavior
    /// (memory corruption, incorrect use of `unsafe` functions, or the like),
    /// regardless of the async iterator's state.
    #[lang = "async_iterator_poll_next"]
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>>;

    /// Returns the bounds on the remaining length of the async iterator.
    ///
    /// Specifically, `size_hint()` returns a tuple where the first element
    /// is the lower bound, and the second element is the upper bound.
    ///
    /// The second half of the tuple that is returned is an <code>[Option]<[usize]></code>.
    /// A [`None`] here means that either there is no known upper bound, or the
    /// upper bound is larger than [`usize`].
    ///
    /// # Implementation notes
    ///
    /// It is not enforced that an async iterator implementation yields the declared
    /// number of elements. A buggy async iterator may yield less than the lower bound
    /// or more than the upper bound of elements.
    ///
    /// `size_hint()` is primarily intended to be used for optimizations such as
    /// reserving space for the elements of the async iterator, but must not be
    /// trusted to e.g., omit bounds checks in unsafe code. An incorrect
    /// implementation of `size_hint()` should not lead to memory safety
    /// violations.
    ///
    /// That said, the implementation should provide a correct estimation,
    /// because otherwise it would be a violation of the trait's protocol.
    ///
    /// The default implementation returns <code>(0, [None])</code> which is correct for any
    /// async iterator.
    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, None)
    }
}

#[unstable(feature = "async_iterator", issue = "79024")]
impl<S: ?Sized + AsyncIterator + Unpin> AsyncIterator for &mut S {
    type Item = S::Item;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        S::poll_next(Pin::new(&mut **self), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

#[unstable(feature = "async_iterator", issue = "79024")]
impl<P> AsyncIterator for Pin<P>
where
    P: DerefMut,
    P::Target: AsyncIterator,
{
    type Item = <P::Target as AsyncIterator>::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        <P::Target as AsyncIterator>::poll_next(self.as_deref_mut(), cx)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (**self).size_hint()
    }
}

#[unstable(feature = "async_gen_internals", issue = "none")]
impl<T> Poll<Option<T>> {
    /// A helper function for internal desugaring -- produces `Ready(Some(t))`,
    /// which corresponds to the async iterator yielding a value.
    #[doc(hidden)]
    #[unstable(feature = "async_gen_internals", issue = "none")]
    #[lang = "AsyncGenReady"]
    pub fn async_gen_ready(t: T) -> Self {
        Poll::Ready(Some(t))
    }

    /// A helper constant for internal desugaring -- produces `Pending`,
    /// which corresponds to the async iterator pending on an `.await`.
    #[doc(hidden)]
    #[unstable(feature = "async_gen_internals", issue = "none")]
    #[lang = "AsyncGenPending"]
    // FIXME(gen_blocks): This probably could be deduplicated.
    pub const PENDING: Self = Poll::Pending;

    /// A helper constant for internal desugaring -- produces `Ready(None)`,
    /// which corresponds to the async iterator finishing its iteration.
    #[doc(hidden)]
    #[unstable(feature = "async_gen_internals", issue = "none")]
    #[lang = "AsyncGenFinished"]
    pub const FINISHED: Self = Poll::Ready(None);
}

/// Converts something into an async iterator
#[unstable(feature = "async_iterator", issue = "79024")]
pub trait IntoAsyncIterator {
    /// The type of the item yielded by the iterator
    type Item;
    /// The type of the resulting iterator
    type IntoAsyncIter: AsyncIterator<Item = Self::Item>;

    /// Converts `self` into an async iterator
    #[lang = "into_async_iter_into_iter"]
    fn into_async_iter(self) -> Self::IntoAsyncIter;
}

#[unstable(feature = "async_iterator", issue = "79024")]
impl<I: AsyncIterator> IntoAsyncIterator for I {
    type Item = I::Item;
    type IntoAsyncIter = I;

    fn into_async_iter(self) -> Self::IntoAsyncIter {
        self
    }
}

pub struct Once{alive:bool}
impl AsyncIterator for Once{
 type Item=u8;
 fn poll_next(mut self:Pin<&mut Self>,cx:&mut Context<'_>)->Poll<Option<bool>>{
  if self.alive{self.alive=false;Poll::Ready(Some(true))}else{Poll::Ready(None)}
 }
}
fn fetch<S:AsyncIterator<Item=u8>+?Sized>(s:Pin<&mut S>,cx:&mut Context<'_>)->u8{
 match S::poll_next(s,cx){Poll::Ready(Some(x))=>x,Poll::Ready(None)=>0u8,Poll::Pending=>1u8}
}
fn context()->Context<'static>{Context{token:&()}}
