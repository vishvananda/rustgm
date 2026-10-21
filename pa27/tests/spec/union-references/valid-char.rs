union U{bits:u32,value:char}const X:char={let u=U{bits:0x1f980};let p=&raw const u.value;unsafe{let r=&*p;*r}};fn main(){let _=X;}
