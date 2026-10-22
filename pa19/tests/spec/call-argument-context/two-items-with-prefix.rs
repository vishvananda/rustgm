fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,a:R,b:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,a:fn(&S)->&S,b:fn(&S)->&S)->bool{a(self).0==3&&b(self).0==3}}fn id(x:&S)->&S{x}fn other(x:&S)->&S{x}fn main(){let a=S(3);check(K::accepts(&&a,id,other));}
