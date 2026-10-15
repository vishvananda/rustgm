fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<const N:usize>(&self);}struct S;impl A for S{fn f<T>(&self){}}fn main(){}
