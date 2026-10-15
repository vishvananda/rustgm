fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
type A<T>=(u8,T);struct S;impl S{fn f<T>(&self,x:T)->T{x}}fn main(){let p=S.f::<A<_>>((7,517u16));check(p.0==7&&p.1==517);}