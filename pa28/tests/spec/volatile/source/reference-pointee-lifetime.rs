unsafe fn f<'a>(p:*const &'a u8)->&'a u8{unsafe{core::ptr::read_volatile(p)}}fn main(){}
