// Const addition companion role, usize implementation, and overflow-location
// types required by checked const addition in Iterator::count.
#[rustc_builtin_macro(pattern_type)]
#[macro_export]
macro_rules! pattern_type {
    ($($args:tt)*) => {};
}

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

impl Clone for usize {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for usize {}

#[lang = "add"]
pub const trait Add<Rhs = Self> {
    type Output;
    #[rustc_diagnostic_item = "add"]
    fn add(self, rhs: Rhs) -> Self::Output;
}

const impl Add for usize {
    type Output = usize;

    fn add(self, rhs: usize) -> usize {
        self + rhs
    }
}

impl Add for u8 {
    type Output = u8;

    fn add(self, rhs: u8) -> u8 {
        self + rhs
    }
}

#[lang = "panic_const_add_overflow"]
pub const fn addition_overflow() -> ! {
    loop {}
}
