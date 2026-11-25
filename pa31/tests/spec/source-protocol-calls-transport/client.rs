#![feature(no_core)]
#![no_core]
extern crate base;

const CONST_OFFSET: u32 = base::inner_offset::<u32>() as u32;

#[unsafe(no_mangle)]
pub extern "C" fn obs_for_concrete() -> u32 {
    // 1 + 2 + 3 + 4 = 10
    base::sum_up_to(5)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_for_generic() -> u32 {
    // 1 + 2 + 3 + 4 + 5 = 15
    base::sum_iter(base::Counter::new(1, 6))
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_try_some() -> u32 {
    match base::add_options(base::Option::Some(20), base::Option::Some(25)) {
        base::Option::Some(v) => v,
        base::Option::None => 0,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_try_none() -> u32 {
    match base::add_options(base::Option::Some(20), base::Option::None) {
        base::Option::Some(v) => v,
        base::Option::None => 99,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_deref_all() -> u32 {
    let w = base::Wrapper(base::Inner {
        first: 10,
        second: 20,
        items: [11, 25],
    });
    // field (10) + method (20) + coercion (11) + index (25) = 66
    base::read_through_deref(&w)
}

#[unsafe(no_mangle)]
pub extern "C" fn obs_offset() -> u32 {
    CONST_OFFSET
}
