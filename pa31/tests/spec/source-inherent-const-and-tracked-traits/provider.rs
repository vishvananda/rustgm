#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}

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

#[lang = "phantom_data"]
pub struct PhantomData<T: PointeeSized>;

#[macro_export]
#[rustc_builtin_macro(pattern_type)]
macro_rules! pattern_type {
    ($($arg:tt)*) => {};
}

#[repr(transparent)]
#[rustc_nonnull_optimization_guaranteed]
#[rustc_diagnostic_item = "NonNull"]
pub struct NonNull<T: PointeeSized> {
    pointer: crate::pattern_type!(*const T is !null),
}

impl<T: PointeeSized> Clone for *const T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for *const T {}
impl<T: PointeeSized> Clone for *mut T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for *mut T {}
impl<T: PointeeSized> Clone for NonNull<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for NonNull<T> {}
impl<T: PointeeSized> Clone for PhantomData<T> {
    fn clone(&self) -> Self {
        PhantomData
    }
}
impl<T: PointeeSized> Copy for PhantomData<T> {}
impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}
impl<T: PointeeSized> Clone for &T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for &T {}

#[lang = "panic_location"]
pub struct Location<'a> {
    filename: NonNull<str>,
    line: u32,
    col: u32,
    _filename: PhantomData<&'a str>,
}

impl Clone for Location<'_> {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for Location<'_> {}

impl Location<'_> {
    pub const fn line(&self) -> u32 {
        self.line
    }
}

#[rustc_intrinsic]
pub const fn caller_location() -> &'static Location<'static>;

#[rustc_intrinsic]
pub const fn wrapping_add<T>(a: T, b: T) -> T;

pub const trait Step {
    fn next_val(self) -> Self;
}

pub struct Counter(pub u32);

const impl Step for Counter {
    fn next_val(self) -> Self {
        Counter(wrapping_add(self.0, 1))
    }
}

pub struct Acc<T> {
    pub value: T,
}

const impl<T: [const] Step + [const] Destruct> Acc<T> {
    pub fn advance(self) -> Acc<T> {
        Acc {
            value: self.value.next_val(),
        }
    }
}

impl str {
    pub const fn str_tag(&self) -> u32 {
        7
    }
}

impl [u8] {
    pub const fn slice_tag(&self) -> u32 {
        9
    }
}

impl<T, const N: usize> [T; N] {
    pub const fn array_len(&self) -> usize {
        N
    }
}

impl<T: PointeeSized> *const T {
    pub const fn const_ptr_tag(self) -> u32 {
        13
    }
}

impl<T: PointeeSized> *mut T {
    pub const fn mut_ptr_tag(self) -> u32 {
        17
    }
}

pub trait Tracked {
    #[track_caller]
    fn report_line(&self) -> u32;
}

impl Tracked for Counter {
    fn report_line(&self) -> u32 {
        caller_location().line
    }
}

pub trait OverrideTracked {
    fn ping_line(&self) -> u32;
}

impl OverrideTracked for Counter {
    #[track_caller]
    fn ping_line(&self) -> u32 {
        caller_location().line
    }
}
