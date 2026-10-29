#![feature(lang_items,rustc_attrs,staged_api,maybe_dangling)]#![unstable(feature="bounded_derive",issue="none")]
use core::mem::MaybeDangling;
#[stable(feature = "manually_drop", since = "1.20.0")]
#[lang = "manually_drop"]
#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
#[rustc_pub_transparent]
pub struct ManuallyDrop<T: ?Sized> {
    value: MaybeDangling<T>,
}
