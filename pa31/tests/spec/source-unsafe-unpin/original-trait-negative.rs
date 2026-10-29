#[lang = "unsafe_unpin"]
#[unstable(feature = "unsafe_unpin", issue = "125735")]
pub unsafe auto trait UnsafeUnpin {}

#[unstable(feature = "unsafe_unpin", issue = "125735")]
impl<T: PointeeSized> !UnsafeUnpin for UnsafePinned<T> {}
