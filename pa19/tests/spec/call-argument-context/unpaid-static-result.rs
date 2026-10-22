fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}struct S;impl K<fn(&S)->&'static S>for&S{fn accepts(&self,_:fn(&S)->&'static S)->bool{true}}fn id(x:&S)->&S{x}fn main(){let a=S;check(K::accepts(&&a,id));}
