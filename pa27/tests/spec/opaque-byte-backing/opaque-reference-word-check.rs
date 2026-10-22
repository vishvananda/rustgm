static X:u8=41;
const fn make()->impl Copy {&X}
union U<T:Copy>{a:T,b:&'static u8}
const fn value<T:Copy>(x:T)->u8{let u=U{a:x};unsafe{*u.b}}
const N:u8=value(make());
fn main(){if N!=41{let _=fail(0);}}
fn fail(x:u8)->u8{1/x}
