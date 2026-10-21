fn f(p:*mut u32)->u32{unsafe{core::ptr::read_volatile(p)}}fn main(){}
