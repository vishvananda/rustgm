fn f(p:*const u32){unsafe{core::ptr::write_volatile(p,7);}}fn main(){}
