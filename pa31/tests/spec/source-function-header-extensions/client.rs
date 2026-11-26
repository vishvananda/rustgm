#![feature(no_core)]
#![no_core]

extern crate base;

use base::{Calc, Pair, wrapping_add};

const SUM_CONST: u32 = base::const_sum_pair((11, 8));
const TF_CONST: u32 = unsafe { Calc::bias(14) };

#[target_feature(enable = "sse2")]
fn with_sse2() -> u32 {
    let direct = base::safe_add(12, 15);
    let ptr: fn(u32, u32) -> u32 = base::safe_add;
    let via_ptr = ptr(3, 4);
    let const_tf = base::const_add(5, 6);
    let scaled = Calc::scale(9);
    wrapping_add(wrapping_add(direct, via_ptr), wrapping_add(const_tf, scaled))
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_extensions(
    a: &mut u32,
    b: &mut u32,
    c: &mut u32,
    d: &mut u32,
    e: &mut u32,
    f: &mut u32,
    g: &mut u32,
) {
    *a = base::sum_pair((10, 22));
    let (y, x) = base::swap_pair((7_u32, 13_u32));
    *b = wrapping_add(wrapping_add(y, x), base::sum_struct(Pair { left: 4, right: 6 }));
    *c = SUM_CONST;
    *d = unsafe { with_sse2() };
    *e = TF_CONST;
    *f = base::call_with_block(13);
    *g = unsafe { base::call_with_function_context(29) };
}
