#![feature(no_core, asm_native_fixture)]
#![no_core]
extern crate base;

fn split(dividend: u64, divisor: u64) -> (u64, u64) {
    let quotient: u64;
    let remainder: u64;
    unsafe {
        base::arch::asm!(
            "div {0}",
            in(reg) divisor,
            inlateout("rax") dividend => quotient,
            inlateout("rdx") 0u64 => remainder,
            options(pure, nomem, nostack),
        );
    }
    (quotient, remainder)
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_quotient(dividend: u64, divisor: u64) -> u64 {
    split(dividend, divisor).0
}

#[unsafe(no_mangle)]
pub extern "C" fn observed_remainder(dividend: u64, divisor: u64) -> u64 {
    split(dividend, divisor).1
}
