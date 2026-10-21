fn f<'a>(p:*const u8)->&'a mut u8{unsafe{&mut *p}}fn main(){}
