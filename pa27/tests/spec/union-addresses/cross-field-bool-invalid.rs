union U{a:u8,b:bool}const X:bool={let mut u=U{b:false};let a=&raw mut u.a;let b=&raw const u.b;unsafe{*a=2;*b}};fn main(){let _=X;}
