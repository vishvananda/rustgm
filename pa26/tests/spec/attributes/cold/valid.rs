#![allow(dead_code)]
#[cold]
#[inline(never)]
const fn observed()->u8{7}
const ANSWER:u8=observed();
