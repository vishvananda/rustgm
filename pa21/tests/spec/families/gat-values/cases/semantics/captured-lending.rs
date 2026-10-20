fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait L{type Out<'a>where Self:'a;fn get<'a>(&'a self)->Self::Out<'a>;}struct S<T>{v:T}impl<T>L for S<T>{type Out<'a>=&'a T where T:'a;fn get<'a>(&'a self)->Self::Out<'a>{&self.v}}fn main(){let s=S{v:7u8};check(*s.get()==7);}
