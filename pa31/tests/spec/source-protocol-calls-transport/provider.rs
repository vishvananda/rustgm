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

impl Clone for bool { fn clone(&self) -> Self { *self } }
impl Copy for bool {}
impl Clone for u32 { fn clone(&self) -> Self { *self } }
impl Copy for u32 {}
impl Clone for usize { fn clone(&self) -> Self { *self } }
impl Copy for usize {}
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}
impl<T: Copy, const N: usize> Clone for [T; N] { fn clone(&self) -> Self { *self } }
impl<T: Copy, const N: usize> Copy for [T; N] {}

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
const impl PartialEq for u32 {
    fn eq(&self, other: &u32) -> bool { *self == *other }
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

#[lang = "panic_bounds_check"]
pub fn panic_bounds_check(_index: usize, _len: usize) -> ! { loop {} }
#[lang = "panic_const_add_overflow"]
pub const fn panic_const_add_overflow() -> ! { loop {} }

#[lang = "Option"]
pub enum Option<T> {
    #[lang = "Some"]
    Some(T),
    #[lang = "None"]
    None,
}

#[lang = "iterator"]
pub const trait Iterator {
    type Item;
    #[lang = "next"]
    fn next(&mut self) -> Option<Self::Item>;
}

#[rustc_skip_during_method_dispatch(array, boxed_slice)]
pub const trait IntoIterator {
    type Item;
    type IntoIter: Iterator<Item = Self::Item>;
    #[lang = "into_iter"]
    fn into_iter(self) -> Self::IntoIter;
}

const impl<I: [const] Iterator> IntoIterator for I {
    type Item = I::Item;
    type IntoIter = I;
    fn into_iter(self) -> I {
        self
    }
}

pub enum ControlFlow<B, C> {
    #[lang = "Continue"]
    Continue(C),
    #[lang = "Break"]
    Break(B),
}

pub const trait FromResidual<R = <Self as Try>::Residual> {
    #[lang = "from_residual"]
    fn from_residual(residual: R) -> Self;
}

#[lang = "Try"]
pub const trait Try: [const] FromResidual {
    type Output;
    type Residual;
    #[lang = "from_output"]
    fn from_output(output: Self::Output) -> Self;
    #[lang = "branch"]
    fn branch(self) -> ControlFlow<Self::Residual, Self::Output>;
}

pub const trait Residual<O>: Sized {
    type TryType: [const] Try<Output = O, Residual = Self>;
}

#[lang = "into_try_type"]
pub const fn residual_into_try_type<R: [const] Residual<O>, O>(value: R) -> <R as Residual<O>>::TryType {
    FromResidual::from_residual(value)
}

impl<T> FromResidual<Option<()>> for Option<T> {
    fn from_residual(_: Option<()>) -> Self {
        Option::None
    }
}

impl<T> Try for Option<T> {
    type Output = T;
    type Residual = Option<()>;
    fn from_output(output: T) -> Self {
        Option::Some(output)
    }
    fn branch(self) -> ControlFlow<Option<()>, T> {
        match self {
            Option::Some(v) => ControlFlow::Continue(v),
            Option::None => ControlFlow::Break(Option::None),
        }
    }
}

#[lang = "deref"]
pub const trait Deref: PointeeSized {
    #[lang = "deref_target"]
    type Target: ?Sized;
    fn deref(&self) -> &Self::Target;
}

#[lang = "index"]
pub const trait Index<Idx: ?Sized> {
    type Output: ?Sized;
    fn index(&self, index: Idx) -> &Self::Output;
}

const impl<T, const N: usize> Index<usize> for [T; N] {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self[index]
    }
}

#[rustc_intrinsic]
#[rustc_nounwind]
#[lang = "offset_of"]
pub const fn offset_of<T: PointeeSized>(variant: u32, field: u32) -> usize;

pub struct Counter {
    cur: u32,
    end: u32,
}

impl Counter {
    pub fn new(start: u32, end: u32) -> Self {
        Self { cur: start, end }
    }
}

impl Iterator for Counter {
    type Item = u32;
    fn next(&mut self) -> Option<u32> {
        if self.cur != self.end {
            let v = self.cur;
            self.cur = self.cur + 1;
            Option::Some(v)
        } else {
            Option::None
        }
    }
}

pub fn sum_up_to(end: u32) -> u32 {
    let mut total = 0u32;
    for x in Counter::new(1, end) {
        total = total + x;
    }
    total
}

pub fn sum_iter<I: IntoIterator<Item = u32>>(iter: I) -> u32 {
    let mut total = 0u32;
    for x in iter {
        total = total + x;
    }
    total
}

pub fn add_options(a: Option<u32>, b: Option<u32>) -> Option<u32> {
    let x = a?;
    let y = b?;
    Option::Some(x + y)
}

#[repr(C)]
pub struct Inner {
    pub first: u32,
    pub second: u32,
    pub items: [u32; 2],
}

impl Inner {
    pub fn read_second(&self) -> u32 {
        self.second
    }
}

impl Index<usize> for Inner {
    type Output = u32;
    fn index(&self, idx: usize) -> &u32 {
        &self.items[idx]
    }
}

pub struct Wrapper(pub Inner);

impl Deref for Wrapper {
    type Target = Inner;
    fn deref(&self) -> &Inner {
        &self.0
    }
}

fn take_inner_first_item(inner: &Inner) -> u32 {
    inner.items[0]
}

pub fn read_through_deref(w: &Wrapper) -> u32 {
    let a = w.first;
    let b = w.read_second();
    let c = take_inner_first_item(w);
    let d = w[1];
    a + b + c + d
}

pub const fn inner_offset<T>() -> usize {
    builtin # offset_of(Inner, items)
}
