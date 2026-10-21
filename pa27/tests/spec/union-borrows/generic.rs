union U<T:Copy>{a:T,b:u32}const fn f<T:Copy>(x:T)->T{let u=U{a:x};unsafe{let r=&u.a;*r}}const X:u32=f::<u32>(37);fn main(){let _=X;}
