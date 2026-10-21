fn f(p:*const unsafe fn(u8)->u8)->u8{unsafe{let g=core::ptr::read_volatile(p);g(7)}}fn main(){}
