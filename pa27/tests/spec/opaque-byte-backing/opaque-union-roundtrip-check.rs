const fn make()->impl Copy {53u8}
union U<T:Copy>{a:T,b:T}
const fn round<T:Copy>(x:T)->T{let u=U{a:x};unsafe{u.b}}
const N:u8={let x=round(make());unsafe{*(&raw const x as *const u8)}};
fn main(){if N!=53{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
