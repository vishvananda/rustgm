#![feature(no_core, const_trait_impl, const_destruct)]
#![no_core]

extern crate base;

use base::{Acc, Counter, OverrideTracked, Tracked, wrapping_add};

const CONST_ACC: u32 = Acc { value: Counter(41) }.advance().value.0;

struct Local;

impl Tracked for Local {
    fn report_line(&self) -> u32 {
        99
    }
}

fn call_tracked<T: Tracked>(value: &T) -> u32 {
    value.report_line()
}

#[unsafe(no_mangle)]
pub extern "C" fn observe_inherent_and_tracked(
    a: &mut u32,
    b: &mut u32,
    c: &mut u32,
    d: &mut u32,
    e: &mut u32,
) {
    *a = CONST_ACC;
    let runtime_acc = Acc { value: Counter(9) }.advance().value.0;
    let text: &str = "hi";
    let bytes: &[u8] = &[9_u8, 2_u8];
    let arr = [1_u32, 2_u32, 3_u32, 4_u32];
    let mut cell = 0_u32;
    *b = wrapping_add(
        wrapping_add(runtime_acc, text.str_tag()),
        wrapping_add(
            bytes.slice_tag(),
            wrapping_add(
                arr.array_len() as u32,
                wrapping_add((&raw const cell).const_ptr_tag(), (&raw mut cell).mut_ptr_tag()),
            ),
        ),
    );
    let counter = Counter(0);
    *c = counter.report_line();
    *d = counter.ping_line();
    *e = call_tracked(&Local);
}
