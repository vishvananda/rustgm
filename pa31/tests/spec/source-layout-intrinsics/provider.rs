pub mod intrinsics {
    #[rustc_nounwind]
    #[rustc_intrinsic_const_stable_indirect]
    #[rustc_intrinsic]
    pub const fn size_of<T>() -> usize;

    #[rustc_nounwind]
    #[rustc_intrinsic_const_stable_indirect]
    #[rustc_intrinsic]
    pub const fn align_of<T>() -> usize;
}

pub trait SizedTypeProperties: Sized {
    #[lang = "mem_size_const"]
    const SIZE: usize = intrinsics::size_of::<Self>();

    #[lang = "mem_align_const"]
    const ALIGN: usize = intrinsics::align_of::<Self>();
}
impl<T> SizedTypeProperties for T {}
