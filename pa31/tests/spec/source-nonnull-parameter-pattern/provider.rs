#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "unsize"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Unsize<T: PointeeSized>: PointeeSized {}
#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T: PointeeSized>: Sized {}
#[lang = "dispatch_from_dyn"]
pub trait DispatchFromDyn<T>: Sized {}
impl<T: PointeeSized> Clone for *const T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for *const T {}
impl<T: PointeeSized> Clone for &T {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for &T {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*const U> for *const T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<*const U> for *const T {}

#[rustc_builtin_macro(pattern_type)]
macro_rules! pattern_type {
    ($($arg:tt)*) => {};
}

impl<T: PointeeSized, U: PointeeSized> CoerceUnsized<pattern_type!(*const U is !null)>
    for pattern_type!(*const T is !null)
where
    T: Unsize<U>,
{
}

impl<T: DispatchFromDyn<U>, U> DispatchFromDyn<pattern_type!(U is !null)>
    for pattern_type!(T is !null)
{
}

#[rustc_diagnostic_item = "NonNull"]
#[repr(transparent)]
pub struct NonNull<T: PointeeSized> {
    pointer: pattern_type!(*const T is !null),
}

impl<T: PointeeSized> Clone for NonNull<T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T: PointeeSized> Copy for NonNull<T> {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<NonNull<U>> for NonNull<T> {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<NonNull<U>> for NonNull<T> {}

pub trait Meter {
    fn read(&self) -> u8;
}

pub fn check_pattern_dispatch<T, U>() -> u8
where
    pattern_type!(T is !null): DispatchFromDyn<pattern_type!(U is !null)>,
{
    17
}

pub fn require_dispatch<A: DispatchFromDyn<B>, B>() -> u8 {
    17
}
