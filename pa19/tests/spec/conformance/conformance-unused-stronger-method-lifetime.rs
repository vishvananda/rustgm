fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<'a>(&self,x:&'a u8);}struct S;impl A for S{fn f<'b:'static>(&self,x:&'b u8){}}fn main(){}
