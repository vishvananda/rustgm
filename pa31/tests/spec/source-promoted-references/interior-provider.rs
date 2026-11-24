#[repr(C)]
pub struct Ops {
    pub message: fn(u8, &mut Arguments<'_>) -> Opt<u8>,
    pub retry: fn(u8) -> bool,
}
impl Ops {
    const DEFAULT: &'static Ops = &Ops {
        message: |_, _| Opt::Some(34),
        retry: |_| false,
    };
}
pub const fn current() -> &'static Ops { Ops::DEFAULT }

#[lang = "sync"]
pub unsafe auto trait Sync {}
impl<T> Clone for *mut T { fn clone(&self) -> Self { *self } }
impl<T> Copy for *mut T {}

#[lang = "unsafe_cell"]
#[repr(transparent)]
pub struct CellSlot<T: ?Sized> { pub item: T }
impl<T: PointeeSized> !Freeze for CellSlot<T> {}
impl<T: PointeeSized> !Sync for CellSlot<T> {}
pub struct Holder<T> { pub slot: CellSlot<T> }
unsafe impl Sync for Holder<*mut Ops> {}

pub static STORED: Holder<*mut Ops> = Holder {
    slot: CellSlot { item: Ops::DEFAULT as *const Ops as *mut Ops },
};
pub static MARKER: u8 = 6;
