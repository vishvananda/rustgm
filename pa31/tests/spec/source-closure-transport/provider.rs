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
impl<T: PointeeSized> Clone for &T { fn clone(&self) -> Self { *self } }
impl<T: PointeeSized> Copy for &T {}

#[lang = "add"]
pub const trait Add<Rhs = Self> {
    type Output;
    fn add(self, rhs: Rhs) -> Self::Output;
}
const impl Add for u32 {
    type Output = u32;
    fn add(self, rhs: u32) -> u32 { self + rhs }
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

pub struct Held(pub u32);

pub const fn fold_three<F: [const] FnMut(u32, u32) -> u32 + [const] Destruct>(mut init: u32, mut f: F) -> u32 {
    init = f(init, 10);
    init = f(init, 20);
    f(init, 30)
}

pub const fn count_three() -> u32 {
    fold_three(0, const |acc, _| acc + 1)
}

pub fn apply_once<F: FnOnce(u32) -> u32>(f: F, x: u32) -> u32 {
    f(x)
}

fn apply_once_zero<F: FnOnce() -> u32>(f: F) -> u32 {
    f()
}

pub fn reify_closure(x: u32) -> u32 {
    let f: fn(u32) -> u32 = |v| v + 7;
    f(x)
}

pub fn capture_fn_and_adapt<T: Copy>(x: u32, _tag: T) -> u32 {
    let bias = x + 1;
    let f = |v: u32| v + bias;
    let a = f(2);
    let b = apply_once(f, 3);
    a + b
}

pub fn capture_unique_mut(mut acc: u32) -> u32 {
    let r = &mut acc;
    let mut bump = |d: u32| {
        *r = *r + d;
    };
    bump(4);
    bump(6);
    acc
}

fn take_held(h: Held) -> u32 {
    h.0
}

pub fn capture_once_move(h: Held) -> u32 {
    let f = move || take_held(h) + 9;
    apply_once_zero(f)
}

pub fn capture_ref_and_nested<'a, T>(a: &'a T, _b: &'a T) -> &'a T {
    let outer = || {
        let inner = || a;
        inner()
    };
    outer()
}

pub fn make_adder(k: u32) -> impl Fn(u32) -> u32 {
    move |x: u32| x + k
}
