unsafe fn f<T>(p:*mut T,x:T)->T{unsafe{core::ptr::write_volatile(p,x);core::ptr::read_volatile(p)}}fn main(){}
