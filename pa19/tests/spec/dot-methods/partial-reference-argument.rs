fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn f<T>(&self,x:T)->T{x}}fn main(){let n=7u8;let p=S.f::<(&_,_)>((&n,517u16));check(*p.0==7&&p.1==517);}