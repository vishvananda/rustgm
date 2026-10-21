fn f(p:*const unsafe fn(u8)->u8)->u8{let g=unsafe{core::ptr::read_volatile(p)};g(7)}fn main(){}
