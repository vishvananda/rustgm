#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}
#[lang = "drop_glue"]
pub const unsafe fn drop_glue<T: PointeeSized>(_: &mut T)
where
    T: [const] Destruct,
{
}
#[lang = "unsize"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Unsize<T: PointeeSized>: PointeeSized {}
#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T: PointeeSized>: Sized {}
#[lang = "dispatch_from_dyn"]
pub trait DispatchFromDyn<T>: Sized {}
impl<'a, T: PointeeSized + Unsize<U>, U: PointeeSized> DispatchFromDyn<&'a U> for &'a T {}
impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*const U> for *const T {}
impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<*mut U> for *mut T {}

impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for i8 { fn clone(&self) -> Self { *self } }
impl Copy for i8 {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}
impl<T: PointeeSized> Clone for *const T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for *const T {}

#[lang = "add"]
pub const trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
const impl Add for u32 {
    type Output = u32;
    fn add(self, rhs: u32) -> u32 { self + rhs }
}

#[lang = "eq"]
pub const trait PartialEq<Rhs: PointeeSized = Self>: PointeeSized {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool { if self.eq(other) { false } else { true } }
}
const impl PartialEq for u8 { fn eq(&self, other: &u8) -> bool { *self == *other } }
const impl PartialEq for u32 { fn eq(&self, other: &u32) -> bool { *self == *other } }
impl PartialEq for [u8; 2] {
    fn eq(&self, other: &[u8; 2]) -> bool {
        let &[a0, a1] = self;
        let &[b0, b1] = other;
        a0 == b0 && a1 == b1
    }
}
impl PartialEq for &[u8; 2] {
    fn eq(&self, other: &&[u8; 2]) -> bool {
        <[u8; 2] as PartialEq>::eq(*self, *other)
    }
}

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

#[lang = "panic_const_add_overflow"]
pub const fn panic_const_add_overflow() -> ! { loop {} }

#[lang = "CStr"]
#[repr(transparent)]
pub struct CStr {
    inner: [i8],
}

#[repr(transparent)]
pub struct Wrap<T: PointeeSized>(pub *const T);

impl<T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<Wrap<U>> for Wrap<T> {}

pub struct Counter(pub u32);
impl Counter {
    pub fn bump(&mut self, delta: u32) -> u32 {
        self.0 = self.0 + delta;
        self.0
    }
}
fn make_counter(start: u32) -> Counter {
    Counter(start)
}
fn is_high(v: u32) -> bool {
    match v {
        0 | 1 | 2 | 3 | 4 | 5 => false,
        _ => true,
    }
}

pub fn temp_guard_and_mut(x: u32) -> u32 {
    let guarded = match make_counter(x) {
        Counter(v) if is_high(v) => v + 1,
        Counter(v) => v,
    };
    let bumped = make_counter(guarded).bump(3);
    let r = &mut make_counter(bumped);
    r.0 = r.0 + 2;
    r.0
}

pub const fn byte_literal_sum() -> u32 {
    let raw: &'static [u8; 3] = b"\xff\x02\x80";
    let &[a, b, _] = raw;
    (a as u32) + (b as u32)
}

pub fn match_raw_bytes(b: &[u8; 2]) -> u32 {
    match b {
        b"\xff\x02" => 11,
        _ => 22,
    }
}

pub fn cstr_first_and_len() -> u32 {
    let s: &'static CStr = c"hi";
    match &s.inner {
        [first, _, _] => (*first as u8 as u32) + 3,
        _ => 0,
    }
}

pub fn unsize_slice_sum(a: u32, b: u32, c: u32) -> u32 {
    let arr = [a, b, c];
    let slice: &[u32] = &arr;
    match slice {
        [x, y, z] => *x + *y + *z,
        _ => 0,
    }
}

pub trait Val {
    fn val(&self) -> u32;
}
pub trait SubVal: Val {
    fn extra(&self) -> u32;
}

pub fn unsize_object<T: Val>(x: &T) -> u32 {
    let d: &dyn Val = x;
    d.val()
}

pub fn unsize_upcast(d: &dyn SubVal) -> u32 {
    let base: &dyn Val = d;
    base.val() + d.extra()
}

pub fn unsize_bound<'a, T: Unsize<U>, U: PointeeSized>(x: &'a T) -> &'a U {
    x
}

pub fn unsize_nominal<T: Val>(x: &T) -> u32 {
    let w: Wrap<T> = Wrap(x as *const T);
    let d: Wrap<dyn Val> = w;
    unsafe { (*d.0).val() }
}
