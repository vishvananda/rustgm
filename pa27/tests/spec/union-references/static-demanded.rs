union U{a:u32,b:u32}static S:U=U{a:37};const R:&u32={let p=&raw const S.b;unsafe{&*p}};const X:u32=*R;fn main(){let _=X;}
