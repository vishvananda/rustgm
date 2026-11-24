
#[lang = "pin"]
#[fundamental]
#[repr(transparent)]
pub struct Pinned<P> {
    pub pointer: P,
}

#[lang = "c_void"]
#[repr(u8)]
pub enum Void {
    Hidden1,
    Hidden2,
}

#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;

#[repr(transparent)]
struct Covariant<T>(PhantomData<fn() -> T>);

#[repr(transparent)]
struct Lifetime<'a>(Covariant<&'a ()>);

#[repr(C)]
#[rustc_pass_indirectly_in_non_rustic_abis]
pub struct Cursor {
    gp_offset: i32,
    fp_offset: i32,
    overflow_arg_area: *const Void,
    reg_save_area: *const Void,
}

#[lang = "va_list"]
#[repr(transparent)]
pub struct VaList<'a> {
    inner: Cursor,
    marker: Lifetime<'a>,
}

pub trait Slot {}

pub struct Ordinary<P> {
    pub pointer: P,
}
