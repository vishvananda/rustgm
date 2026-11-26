#![feature(rustc_attrs)]
#![allow(internal_features)]

#[rustc_legacy_const_generics(0, 2)]
pub const fn select<const STEP: usize, const ADD: bool>(first: usize, second: usize) -> usize {
    let base = (first << 4) | second;
    if ADD { base ^ STEP } else { base & !STEP }
}
