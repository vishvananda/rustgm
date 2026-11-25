#![feature(no_core)]
#![no_core]
extern crate base;

const CONST_SEED: u32 = base::const_hrtb_passthrough::<base::Cell>(42);

#[unsafe(no_mangle)]
pub extern "C" fn obs_hrtb_a() -> u32 {
    let c = base::Cell(11);
    let o = 29u32;
    let (a, _, _) = base::apply_hrtb(&c, &o);
    a
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_hrtb_b() -> u32 {
    let c = base::Cell(11);
    let o = 29u32;
    let (_, b, _) = base::nested_hrtb(&c, &o);
    b
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_hrtb_c() -> u32 {
    let c = base::Cell(11);
    let o = 29u32;
    let (_, _, d) = base::nested_hrtb(&c, &o);
    d
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_fn_a() -> u32 {
    let x = 7u32;
    let y = 13u32;
    let (a, _) = base::call_higher_ranked(base::id_ref, &x, &y);
    a
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_fn_b() -> u32 {
    let x = 7u32;
    let y = 13u32;
    let (_, b) = base::call_higher_ranked(base::id_ref, &x, &y);
    b
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_const() -> u32 {
    CONST_SEED
}
