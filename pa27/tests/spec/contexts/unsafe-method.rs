struct S;impl S{unsafe fn g(&self)->u8{7}}fn f(s:&S)->u8{unsafe{s.g()}}fn main(){}
