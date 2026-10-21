struct S{x:u8}fn f(p:*const S)->*const u8{&raw const (*p).x}fn main(){}
