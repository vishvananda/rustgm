fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}impl A for u8{fn f(self)->u8{self+1}}fn pass<T>(x:T)->T{x}fn main(){let p=pass(A::f);let x=p(3u8);check(x==4);}