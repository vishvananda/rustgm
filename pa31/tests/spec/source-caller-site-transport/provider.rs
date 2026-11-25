impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}

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

#[lang = "panic_location"]
pub struct Location<'a> {
    filename: NonNull<str>,
    line: u32,
    col: u32,
    _filename: PhantomData<&'a str>,
}

pub mod intrinsics {
    #[rustc_intrinsic_const_stable_indirect]
    #[rustc_nounwind]
    #[rustc_intrinsic]
    pub const fn caller_location() -> &'static crate::Location<'static>;
}

impl<'a> Location<'a> {
    #[must_use]
    #[track_caller]
    #[inline]
    pub const fn caller() -> &'static Location<'static> {
        crate::intrinsics::caller_location()
    }

    pub const fn line(&self) -> u32 { self.line }
    pub const fn column(&self) -> u32 { self.col }
}

#[track_caller]
const fn private_tracked() -> &'static Location<'static> {
    Location::caller()
}

#[track_caller]
pub const fn inner_tracked<T>() -> &'static Location<'static> {
    private_tracked()
}

#[track_caller]
pub const fn outer_tracked<T>() -> &'static Location<'static> {
    inner_tracked::<T>()
}

pub const fn untracked_wrapper<T>() -> &'static Location<'static> {
    inner_tracked::<T>()
}

pub const fn branch_caller(first: bool) -> &'static Location<'static> {
    if first {
        Location::caller()
    } else {
        inner_tracked::<u32>()
    }
}

pub trait Locate {
    fn locate() -> &'static Location<'static>;
}

pub struct TrackedImpl;
pub struct UntrackedImpl;

impl Locate for TrackedImpl {
    #[track_caller]
    fn locate() -> &'static Location<'static> {
        Location::caller()
    }
}

impl Locate for UntrackedImpl {
    fn locate() -> &'static Location<'static> {
        Location::caller()
    }
}

pub fn call_locate<L: Locate>() -> &'static Location<'static> {
    L::locate()
}
