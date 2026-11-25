#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[lang = "unsafe_cell"]
#[repr(transparent)]
pub struct UnsafeCell<T: PointeeSized> {
    pub value: T,
}

#[lang = "unsize"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Unsize<T: PointeeSized>: PointeeSized {}

#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T: PointeeSized>: Sized {}

impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

#[rustc_intrinsic]
pub const fn wrapping_add<T: Copy>(a: T, b: T) -> T;

impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}

pub const TABLE: [u8; 4] = [3, 7, 11, 19];
pub const TAG: &str = "rustgm";
const LUT: &[u8; 4] = &[2, 4, 6, 8];

pub struct Flag(UnsafeCell<u8>);

pub const FLAG_INIT: Flag = Flag(UnsafeCell { value: 5 });

impl Flag {
    pub const fn into_inner(self) -> u8 {
        self.0.value
    }
}

pub struct Pair {
    pub a: u32,
    pub b: u32,
}

pub const PAIR: Pair = Pair { a: 10, b: 20 };

impl Pair {
    pub const LABEL: &str = "pair";
    pub const WEIGHTS: [u8; 3] = [1, 2, 3];
}

pub const trait ConstSeed {
    fn seed() -> u32;
}

pub struct Wrap<T>(pub T);

const impl<T: const ConstSeed> Wrap<T> {
    pub const ONE: u32 = T::seed();
}

pub const fn add4(a: u8, b: u8, c: u8, d: u8) -> u8 {
    wrapping_add(wrapping_add(a, b), wrapping_add(c, d))
}

pub const fn table_copy() -> [u8; 4] {
    TABLE
}

pub const fn lut_ref() -> &'static [u8; 4] {
    LUT
}

pub fn local_table() -> [u8; 4] {
    const LOCAL: [u8; 4] = [9, 8, 7, 6];
    LOCAL
}

pub const fn tag_ref() -> &'static str {
    TAG
}

pub const fn label_ref() -> &'static str {
    Pair::LABEL
}
