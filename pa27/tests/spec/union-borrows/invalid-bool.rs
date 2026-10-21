union U{bits:u32,value:bool}const X:bool={let u=U{bits:2};unsafe{let r=&u.value;*r}};fn main(){let _=X;}
