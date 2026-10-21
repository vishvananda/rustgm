const X:u32={let x=37u32;let f:unsafe fn(*const u32)->u32=core::ptr::read_unaligned::<u32>;unsafe{f(&raw const x)}};fn main(){let _=X;}
