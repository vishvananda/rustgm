union U{a:u8,b:bool}const X:bool={let u=U{a:1};let p=&raw const u.b;unsafe{*p}};fn main(){let _=X;}
