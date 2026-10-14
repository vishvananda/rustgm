fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T=u8,U=T>{a:T,b:U}fn main(){let s:S<u16>=S{a:300,b:500};check(s.a+s.b==800);}
