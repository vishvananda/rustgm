#[doc(hidden)]
#[unstable(feature = "sized_type_properties", issue = "none")]
pub trait SizedTypeProperties: Sized {
    #[doc(hidden)]
    #[unstable(feature = "sized_type_properties", issue = "none")]
    #[lang = "mem_size_const"]
    const SIZE: usize = intrinsics::size_of::<Self>();

    #[doc(hidden)]
    #[unstable(feature = "sized_type_properties", issue = "none")]
    #[lang = "mem_align_const"]
    const ALIGN: usize = intrinsics::align_of::<Self>();


