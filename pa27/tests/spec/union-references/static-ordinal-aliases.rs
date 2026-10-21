union U{a:u32,b:u32}static S:U=U{a:37};const A:&u32={let p=&raw const S.a;unsafe{&*p}};const B:&u32={let p=&raw const S.b;unsafe{&*p}};const X:u32=*A+*B;fn main(){let _=X;}
