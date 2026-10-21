fn f(p:*mut u32){unsafe{std::ptr::write_volatile(p,7);}}fn main(){}
