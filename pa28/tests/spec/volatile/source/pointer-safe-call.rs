fn f(p:*const u32)->u32{let r:unsafe fn(*const u32)->u32=core::ptr::read_volatile::<u32>;r(p)}fn main(){}
