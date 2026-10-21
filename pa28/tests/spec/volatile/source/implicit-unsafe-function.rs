unsafe fn f(p:*const u32)->u32{core::ptr::read_volatile(p)}fn main(){}
