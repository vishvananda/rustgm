const fn f<T>(p:*const u32)->u32{if false{unsafe{core::ptr::read_volatile(p)}}else{0}}fn main(){}
