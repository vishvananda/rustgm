union U{small:u8,wide:u64}static S:U=U{small:7};const R:&u64={let p=&raw const S.wide;unsafe{&*p}};const X:u64=*R;fn main(){let _=X;}
