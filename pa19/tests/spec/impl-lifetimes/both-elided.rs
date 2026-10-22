fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn sum(&self,r:R)->u8;}struct S(u8);impl K<&S> for &S{fn sum(&self,r:&S)->u8{self.0+r.0}}fn main(){let a=S(3);let b=S(5);check(K::sum(&&a,&b)==8);}
