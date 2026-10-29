#[lang = "unsafe_pinned"]
#[repr(transparent)]
#[unstable(feature = "unsafe_pinned", issue = "125735")]
pub struct UnsafePinned<T: ?Sized> {
    value: UnsafeCell<T>,
}
