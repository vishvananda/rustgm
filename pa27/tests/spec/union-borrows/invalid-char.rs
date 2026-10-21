union U{bits:u32,value:char}const X:char={let u=U{bits:0xd800};unsafe{let r=&u.value;*r}};fn main(){let _=X;}
