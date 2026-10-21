fn f<'a>(p:*const u8)->&'a u8{unsafe{&*p}}fn main(){}
