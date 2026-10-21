fn f<'a>(p:*mut u8)->&'a mut u8{unsafe{&mut *p}}fn main(){}
