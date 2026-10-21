struct S;impl S{unsafe fn g(&self)->u8{7}}fn f(s:&S)->u8{s.g()}fn main(){}
