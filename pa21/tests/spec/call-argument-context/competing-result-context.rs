fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}impl K<fn(u8)->u8>for&S{fn accepts(&self,_:fn(u8)->u8)->bool{false}}fn id(x:&S)->&S{x}fn choose<R>(r:R)->R where for<'a> &'a S:K<R>{r}fn main(){let a=S(3);let f:fn(&S)->&S=choose(id);check(f(&a).0==3);}
