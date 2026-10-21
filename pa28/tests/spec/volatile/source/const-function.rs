const fn f(p:*const u32)->u32{unsafe{core::ptr::read_volatile(p)}}fn main(){}
