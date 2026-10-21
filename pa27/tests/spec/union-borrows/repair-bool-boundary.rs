union U{bits:u32,value:bool}const X:bool={let mut u=U{bits:2};unsafe{let r=&mut u.value;*r=true;u.value}};fn main(){let _=X;}
