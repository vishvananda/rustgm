union U{a:u32,b:u32}const X:u32={let u=U{a:7};unsafe{let r=&mut u.a;*r=37;u.b}};fn main(){let _=X;}
