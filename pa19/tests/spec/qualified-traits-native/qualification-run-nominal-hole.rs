fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f(self)->u8;}struct W<T>(T);impl A for W<u8>{fn f(self)->u8{self.0}}fn main(){let x=<W<_>as A>::f(W(3u8));check(x==3);}