fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait K<R>{fn accepts(&self,r:R)->bool;}impl<T>K<&T>for&T{fn accepts(&self,_:&T)->bool{true}}fn main(){let a=3u8;let b=5u8;check(K::accepts(&&a,&b));}
