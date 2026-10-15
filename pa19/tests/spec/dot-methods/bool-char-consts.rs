fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f<const B:bool,const C:char>(&self)->(bool,char){(B,C)}}fn main(){let p=S.f::<true,'x'>();check(p.0&&p.1=='x');}