#![feature(rustc_attrs)]
use core::mem::MaybeUninit;
// Don't let MIR inline this, because we really want it to keep its noalias metadata
#[rustc_no_mir_inline]
#[inline]
fn swap_chunk<const N: usize>(x: &mut MaybeUninit<[u8; N]>, y: &mut MaybeUninit<[u8; N]>) {
    let a = *x;
    let b = *y;
    *x = b;
    *y = a;
}

