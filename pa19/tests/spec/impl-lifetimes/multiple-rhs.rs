fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<A,B>{fn sum(&self,a:A,b:B)->u8;}struct S(u8);impl K<&S,&S>for&S{fn sum(&self,a:&S,b:&S)->u8{self.0+a.0+b.0}}fn main(){let a=S(1);let b=S(3);let c=S(5);check(K::sum(&&a,&b,&c)==9);}
