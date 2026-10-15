fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}struct S;impl A for S{fn f(self)->u8{7}}fn main(){let x=S::f(S);check(x==7);}