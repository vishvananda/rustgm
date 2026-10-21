fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const R:unsafe fn(*const u32)->u32=core::ptr::read_volatile::<u32>;fn main(){let mut x:u32=3;let w:unsafe fn(*mut u32,u32)=core::ptr::write_volatile::<u32>;unsafe{w(&raw mut x,23);check(R(&raw const x)==23);}}
