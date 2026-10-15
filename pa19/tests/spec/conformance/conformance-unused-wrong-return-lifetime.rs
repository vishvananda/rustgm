fn fail(x:u8)->u8{1/x}fn check(ok:bool){if !ok{let _=fail(0);}}trait A{fn f<'a,'b>(&self,x:&'a u8,y:&'b u8)->&'a u8;}struct S;impl A for S{fn f<'a,'b>(&self,x:&'a u8,y:&'b u8)->&'b u8{y}}fn main(){}
