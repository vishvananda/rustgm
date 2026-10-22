fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Marker{}trait K<R>{fn accepts(&self,r:R)->bool;}struct S<T>(T);impl<T:Marker>K<fn(&S<T>)->&S<T>>for&S<T>{fn accepts(&self,_:fn(&S<T>)->&S<T>)->bool{true}}fn id<T>(x:&S<T>)->&S<T>{x}fn main(){let a=S(3u8);check(K::accepts(&&a,id));}
