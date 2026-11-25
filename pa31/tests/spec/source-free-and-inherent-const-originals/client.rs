#![feature(no_core, const_trait_impl)]
#![no_core]

extern crate base;

use base::{ConstSeed, add4, wrapping_add};

struct Seven;

const impl ConstSeed for Seven {
    fn seed() -> u32 {
        7
    }
}

const C_TABLE: u8 = {
    let [a, b, c, d] = base::TABLE;
    add4(a, b, c, d)
};
const C_PAIR: u32 = wrapping_add(base::PAIR.a, base::PAIR.b);
const C_WEIGHTS: u8 = {
    let [a, b, c] = base::Pair::WEIGHTS;
    add4(a, b, c, 0)
};
const C_ONE: u32 = base::Wrap::<Seven>::ONE;
const C_LUT: u8 = {
    let [a, b, c, d] = *base::lut_ref();
    add4(a, b, c, d)
};
const C_FLAG: u8 = base::FLAG_INIT.into_inner();
const C_PROV_TABLE: u8 = {
    let [a, b, c, d] = base::table_copy();
    add4(a, b, c, d)
};
const C_TAG: &str = base::tag_ref();
const C_LABEL: &str = base::label_ref();

#[unsafe(no_mangle)]
pub extern "C" fn observe_const_originals(out: &mut [u32; 11]) {
    let runtime_flag = base::FLAG_INIT.into_inner();
    let runtime_pair = wrapping_add(base::PAIR.a, base::PAIR.b);
    let [l0, l1, l2, l3] = base::local_table();
    let runtime_local = add4(l0, l1, l2, l3);
    let runtime_tag = unsafe { *(base::TAG as *const str as *const u8) };
    let runtime_ctag = unsafe { *(C_TAG as *const str as *const u8) };
    let runtime_label = unsafe { *(C_LABEL as *const str as *const u8) };
    *out = [
        C_TABLE as u32,
        C_PAIR,
        C_WEIGHTS as u32,
        C_ONE,
        C_LUT as u32,
        wrapping_add(C_FLAG as u32, runtime_flag as u32),
        wrapping_add(C_PROV_TABLE as u32, runtime_pair),
        runtime_local as u32,
        runtime_tag as u32,
        runtime_label as u32,
        runtime_ctag as u32,
    ];
}
