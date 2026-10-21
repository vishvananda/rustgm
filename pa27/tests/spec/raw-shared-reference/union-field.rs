union U{word:u32,byte:u8}const X:u32={let u=U{word:37};let p=unsafe{&raw const u.word};unsafe{let r=&*p;*r}};fn main(){let _=X;}
