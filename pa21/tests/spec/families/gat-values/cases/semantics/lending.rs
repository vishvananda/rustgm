fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait L{type Out<'a>where Self:'a;fn get<'a>(&'a self)->Self::Out<'a>;}struct S{v:u8}impl L for S{type Out<'a>=&'a u8;fn get<'a>(&'a self)->Self::Out<'a>{&self.v}}fn main(){let s=S{v:7};check(*s.get()==7);}
