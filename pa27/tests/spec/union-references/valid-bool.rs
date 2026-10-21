union U{bits:u32,value:bool}const X:bool={let u=U{bits:1};let p=&raw const u.value;unsafe{let r=&*p;*r}};fn main(){let _=X;}
