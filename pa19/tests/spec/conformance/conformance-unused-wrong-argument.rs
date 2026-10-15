fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f(&self,x:u8);}struct S;impl A for S{fn f(&self,x:u16){}}fn main(){}
