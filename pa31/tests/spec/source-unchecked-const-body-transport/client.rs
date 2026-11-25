#![feature(no_core)]
#![no_core]

extern crate base;

const fn const_dead_branch(flag: bool) -> u32 {
    if flag {
        base::unchecked_free(1)
    } else {
        42
    }
}

const DEAD_RESULT: u32 = const_dead_branch(false);
const COMPTIME_RESULT: u32 = base::comptime_seed();

#[unsafe(no_mangle)]
pub extern "C" fn obs_free() -> u32 {
    base::unchecked_free(10)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_generic() -> u32 {
    let first = base::unchecked_generic(20u32, 30u32, true);
    let second = base::unchecked_generic(20u32, 30u32, false);
    match (first, second) {
        (20, 30) => 50,
        _ => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_method() -> u32 {
    base::Wrap(5).unchecked_method()
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_const_dead() -> u32 {
    DEAD_RESULT
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_comptime() -> u32 {
    COMPTIME_RESULT
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_cmp() -> u32 {
    let a = 1u32;
    let b = 2u32;
    let same = base::cmp_ptrs(&a as *const u32, &a as *const u32);
    let diff = base::cmp_ptrs(&a as *const u32, &b as *const u32);
    let eq_same = base::generic_ptr_eq(&a as *const u32, &a as *const u32);
    let eq_diff = base::generic_ptr_eq(&a as *const u32, &b as *const u32);
    match (same, diff, eq_same, eq_diff) {
        (1, 0, true, false) => 10,
        _ => 0,
    }
}
