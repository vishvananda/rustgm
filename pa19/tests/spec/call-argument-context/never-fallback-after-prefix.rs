fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S(u8);impl K<fn(&S)->&S>for&S{fn accepts(&self,r:fn(&S)->&S)->bool{r(self).0==self.0}}fn id(x:&S)->&S{x}fn plain<T>(_:T){}fn main(){let a=S(3);check(K::accepts(&&a,id));if false{plain(loop{});}check(true);}
