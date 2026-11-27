extern crate self as core;

#[derive(crate::marker::Copy, crate::clone::Clone)]
#[repr(transparent)]
#[rustc_pub_transparent]
#[lang = "maybe_dangling"]
pub struct MaybeDangling<T: ?Sized>(pub T);

#[derive(crate::marker::Copy, crate::clone::Clone)]
#[repr(transparent)]
#[rustc_pub_transparent]
#[lang = "manually_drop"]
pub struct ManuallyDrop<T: ?Sized> {
    pub value: MaybeDangling<T>,
}
