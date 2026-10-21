union U<T:Copy>{a:T,b:u8}const fn address<T:Copy>(u:&U<T>)->*const T{&raw const u.a}const X:u32={let u=U{a:83u32};let p=address(&u);unsafe{*p}};fn main(){let _=X;}
