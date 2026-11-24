#[rustc_builtin_macro(pattern_type)]
#[macro_export]
macro_rules! pattern_type { ($($args:tt)*) => {}; }

#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonNull"]
pub struct NonNull<T: PointeeSized> {
    pointer: crate::pattern_type!(*const T is !null),
}

#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;

// The field types and order follow pinned core/src/panic/location.rs.
#[lang = "panic_location"]
pub struct Location<'a> {
    filename: NonNull<str>,
    line: u32,
    col: u32,
    _filename: PhantomData<&'a str>,
}

impl Clone for u8 {
    fn clone(&self) -> Self { *self }
}
impl Copy for u8 {}

impl Clone for bool {
    fn clone(&self) -> Self { *self }
}
impl Copy for bool {}

#[lang = "add"]
pub trait Add<Rhs = Self> {
    type Output;
    fn add(self, other: Rhs) -> Self::Output;
}

impl Add for u8 {
    type Output = u8;

    fn add(self, other: u8) -> u8 { self + other }
}

#[track_caller]
#[lang = "panic_const_add_overflow"]
pub const fn addition_overflow() -> ! {
    loop {}
}

#[track_caller]
pub fn forwarded(value: u8) -> u8 {
    value
}
