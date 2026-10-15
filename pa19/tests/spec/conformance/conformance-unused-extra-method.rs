fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{}struct S;impl A for S{fn f(&self){}}fn main(){}
