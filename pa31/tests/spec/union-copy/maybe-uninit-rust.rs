#![feature(lang_items,rustc_attrs,staged_api,transparent_unions)]#![unstable(feature="union_copy_fixture",issue="none")]
use core::mem::ManuallyDrop;use core::marker::Copy;use core::clone::Clone;
#[stable(feature = "maybe_uninit", since = "1.36.0")]
// Lang item so we can wrap other types in it. This is useful for coroutines.
#[lang = "maybe_uninit"]
#[derive(Copy)]
#[repr(transparent)]
#[rustc_pub_transparent]
pub union MaybeUninit<T> {
    uninit: (),
    value: ManuallyDrop<T>,
}

#[stable(feature = "maybe_uninit", since = "1.36.0")]
impl<T: Copy> Clone for MaybeUninit<T> {
    #[inline(always)]
    fn clone(&self) -> Self {
        // Not calling `T::clone()`, we cannot know if we are initialized enough for that.
        *self
    }
}
