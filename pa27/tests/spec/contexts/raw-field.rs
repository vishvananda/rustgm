struct S{x:u8}fn f(p:*const S)->u8{unsafe{(*p).x}}fn main(){}
