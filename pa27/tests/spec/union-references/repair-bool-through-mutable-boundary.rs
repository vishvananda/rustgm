union U{bits:u32,value:bool}const X:bool={let mut u=U{bits:2};let p=&raw mut u.value;unsafe{let r=&mut *p;*r=true;u.value}};fn main(){let _=X;}
