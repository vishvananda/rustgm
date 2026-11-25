impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for u16 { fn clone(&self) -> Self { *self } }
impl Copy for u16 {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl Clone for i32 { fn clone(&self) -> Self { *self } }
impl Copy for i32 {}
impl Clone for usize { fn clone(&self) -> Self { *self } }
impl Copy for usize {}
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}
impl<T: PointeeSized> Clone for *const T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for *const T {}
impl<T: PointeeSized> Clone for *mut T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for *mut T {}
impl<T: Copy, const N: usize> Clone for [T; N] { fn clone(&self) -> Self { *self } }
impl<T: Copy, const N: usize> Copy for [T; N] {}

#[lang = "add"]
pub const trait Add<Rhs = Self> { type Output; fn add(self, rhs: Rhs) -> Self::Output; }
const impl Add for u32 { type Output = u32; fn add(self, rhs: u32) -> u32 { self + rhs } }
#[lang = "rem"]
pub const trait Rem<Rhs = Self> { type Output; fn rem(self, rhs: Rhs) -> Self::Output; }
const impl Rem for u32 { type Output = u32; fn rem(self, rhs: u32) -> u32 { self % rhs } }
#[lang = "shl"]
pub const trait Shl<Rhs = Self> { type Output; fn shl(self, rhs: Rhs) -> Self::Output; }
const impl Shl<u8> for u32 { type Output = u32; fn shl(self, rhs: u8) -> u32 { self << rhs } }
#[lang = "shr"]
pub const trait Shr<Rhs = Self> { type Output; fn shr(self, rhs: Rhs) -> Self::Output; }
const impl Shr<u16> for u32 { type Output = u32; fn shr(self, rhs: u16) -> u32 { self >> rhs } }
#[lang = "neg"]
pub const trait Neg { type Output; fn neg(self) -> Self::Output; }
const impl Neg for i32 { type Output = i32; fn neg(self) -> i32 { -self } }
#[lang = "not"]
pub const trait Not { type Output; fn not(self) -> Self::Output; }
const impl Not for u32 { type Output = u32; fn not(self) -> u32 { !self } }
#[lang = "freeze"]
pub unsafe auto trait Freeze {}
#[lang = "eq"]
pub const trait PartialEq<Rhs: PointeeSized = Self>: PointeeSized {
    fn eq(&self, other: &Rhs) -> bool;
    fn ne(&self, other: &Rhs) -> bool { if self.eq(other) { false } else { true } }
}
const impl PartialEq for u8 { fn eq(&self, other: &u8) -> bool { *self == *other } }
const impl PartialEq for u32 { fn eq(&self, other: &u32) -> bool { *self == *other } }
impl PartialEq for str {
    fn eq(&self, other: &str) -> bool {
        let a = unsafe { &*(self as *const str as *const [u8]) };
        let b = unsafe { &*(other as *const str as *const [u8]) };
        match (a, b) {
            ([x0, x1], [y0, y1]) => *x0 == *y0 && *x1 == *y1,
            _ => false,
        }
    }
}
impl PartialEq for &str {
    fn eq(&self, other: &&str) -> bool {
        <str as PartialEq>::eq(*self, *other)
    }
}
#[lang = "index"]
pub const trait Index<Idx: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}
const impl<T> Index<usize> for [T] {
    type Output = T;
    fn index(&self, index: usize) -> &T { &self[index] }
}
const impl<T, const N: usize> Index<usize> for [T; N] {
    type Output = T;
    fn index(&self, index: usize) -> &T { &self[index] }
}

#[lang = "unsize"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub trait Unsize<T: PointeeSized>: PointeeSized {}
#[lang = "coerce_unsized"]
pub trait CoerceUnsized<T: PointeeSized>: Sized {}
impl<'a, 'b: 'a, T: PointeeSized + Unsize<U>, U: PointeeSized> CoerceUnsized<&'a U> for &'b T {}

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

#[lang = "panic_bounds_check"]
pub fn panic_bounds_check(_index: usize, _len: usize) -> ! { loop {} }
#[lang = "panic_misaligned_pointer_dereference"]
#[rustc_nounwind]
pub fn panic_misaligned_pointer_dereference(_required: usize, _found: usize) -> ! { loop {} }
#[lang = "panic_null_pointer_dereference"]
#[rustc_nounwind]
pub fn panic_null_pointer_dereference() -> ! { loop {} }
#[lang = "panic_const_add_overflow"]
pub const fn panic_const_add_overflow() -> ! { loop {} }
#[lang = "panic_const_neg_overflow"]
pub const fn panic_const_neg_overflow() -> ! { loop {} }
#[lang = "panic_const_rem_overflow"]
pub const fn panic_const_rem_overflow() -> ! { loop {} }
#[lang = "panic_const_rem_by_zero"]
pub const fn panic_const_rem_by_zero() -> ! { loop {} }
#[lang = "panic_const_shl_overflow"]
pub const fn panic_const_shl_overflow() -> ! { loop {} }
#[lang = "panic_const_shr_overflow"]
pub const fn panic_const_shr_overflow() -> ! { loop {} }

pub enum Void {}

#[repr(u32)]
pub enum Tag {
    Alpha = 3,
    Beta = 7,
}

#[repr(C)]
pub union Word {
    pub raw: u32,
    pub bytes: [u8; 4],
}

#[repr(C)]
pub union Sealed {
    secret: u32,
    pub open: u32,
}

pub const fn unary_and_shifts(x: i32, shift: u8) -> u32 {
    let neg = -x;
    let flipped = !0u32;
    let shifted = ((neg as u32) << shift) >> (shift as u16);
    (shifted % 100) + ((flipped != 0) as u32)
}

pub fn pointer_and_union(mut val: u32) -> u32 {
    let p: *mut u32 = &raw mut val;
    unsafe {
        *p = *p + 4;
    }
    let addr = p as usize;
    let back = addr as *const u32;
    let mut u = Word { raw: unsafe { *back } };
    unsafe {
        u.raw = u.raw + 2;
        u.raw
    }
}

pub const fn repeat_and_index<const N: usize>(seed: u32, idx: usize) -> u32 {
    let arr = [seed; N];
    let elem = arr[idx];
    let fixed = [10u32, 20u32, 30u32, 40u32];
    let [first, mid @ ..] = fixed;
    elem + first + mid[1]
}

pub const fn slice_sum(slice: &[u32]) -> u32 {
    match slice {
        [first, rest @ .., last] => *first + rest[0] + *last,
        _ => 0,
    }
}

pub fn match_text(label: &str) -> u32 {
    match label {
        "ok" => 5,
        _ => 1,
    }
}

pub fn thin_and_fn_cast(slice: &[u32]) -> u32 {
    let raw: *const [u32] = slice as *const [u32];
    let thin: *const u32 = raw as *const u32;
    let f: fn(u32) -> u32 = add_one as fn(u32) -> u32;
    let first = unsafe { *thin };
    f(first) + (Tag::Beta as u32)
}

const fn add_one(v: u32) -> u32 {
    v + 1
}

pub fn never_void(v: &Void) -> u32 {
    match *v {}
}

pub fn sealed_open() -> u32 {
    let s = Sealed { open: 9 };
    unsafe { s.open }
}
