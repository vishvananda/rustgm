fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait P{type R;}struct Tag;impl P for Tag{type R=u8;}struct S(u8);impl PartialEq<u8> for S{fn eq(&self,r:&u8)->bool{self.0==*r}}fn equal<T:PartialEq<Q::R>,Q:P>(a:T,b:Q::R)->bool{a==b}fn main(){check(equal::<S,Tag>(S(4),4));}
