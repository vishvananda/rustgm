const fn f<T>(p:*const T)->T{unsafe{core::ptr::read_volatile(p)}}fn main(){}
