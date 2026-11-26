#[lang = "destruct"]
#[rustc_deny_explicit_impl]
#[rustc_dyn_incompatible_trait]
pub const trait Destruct: PointeeSized {}

#[lang = "freeze"]
pub unsafe auto trait Freeze {}

#[cfg(rustc_oracle)]
#[lang = "drop_glue"]
pub unsafe fn drop_glue<T: PointeeSized>(_: &mut T) {}

impl Clone for u32 {
    fn clone(&self) -> Self {
        *self
    }
}
impl Copy for u32 {}

pub struct Pair {
    pub left: u32,
    pub right: u32,
}

pub fn sum_pair((a, b): (u32, u32)) -> u32 {
    wrapping_add(a, b)
}

pub fn swap_pair<T, U>((a, b): (T, U)) -> (U, T) {
    (b, a)
}

pub fn sum_struct(Pair { left, right }: Pair) -> u32 {
    wrapping_add(left, right)
}

pub const fn const_sum_pair((a, b): (u32, u32)) -> u32 {
    wrapping_add(a, b)
}

#[repr(C)]
pub struct Lane(pub u32, pub u32);

unsafe extern "unadjusted" {
    #[allow(dead_code)]
    #[link_name = "llvm.x86.addcarry.32"]
    fn llvm_addcarry_u32(c: u8, a: u32, b: u32) -> (u8, u32);
    #[allow(dead_code)]
    #[link_name = "llvm.x86.lane.probe"]
    fn llvm_lane_probe(p: *const Lane) -> Lane;
}

#[target_feature(enable = "sse2")]
pub fn safe_add(a: u32, b: u32) -> u32 {
    wrapping_add(a, b)
}

pub fn call_with_block(value: u32) -> u32 {
    unsafe { safe_add(value, 4) }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub unsafe fn call_with_function_context(value: u32) -> u32 {
    safe_add(value, 9)
}

#[target_feature(enable = "sse2")]
pub const fn const_add(a: u32, b: u32) -> u32 {
    wrapping_add(a, b)
}

pub struct Calc;

impl Calc {
    #[target_feature(enable = "sse2")]
    pub fn scale(x: u32) -> u32 {
        safe_add(x, x)
    }

    #[target_feature(enable = "sse2")]
    pub const unsafe fn bias(x: u32) -> u32 {
        wrapping_add(x, 5)
    }
}

#[rustc_intrinsic]
pub const fn wrapping_add<T>(a: T, b: T) -> T;
