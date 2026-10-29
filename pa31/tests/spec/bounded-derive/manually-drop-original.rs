#[stable(feature = "manually_drop", since = "1.20.0")]
#[lang = "manually_drop"]
#[derive(Copy, Clone, Debug, Default)]
#[repr(transparent)]
#[rustc_pub_transparent]
pub struct ManuallyDrop<T: ?Sized> {
    value: MaybeDangling<T>,
}
