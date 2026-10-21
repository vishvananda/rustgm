union U{a:u32,b:u32}static S:U=U{a:37};const R:&u32=unsafe{&S.b};const X:u32=*R;fn main(){let _=X;}
