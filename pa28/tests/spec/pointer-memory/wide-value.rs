const S:[u32;3]=[3,7,37];const X:u32={let mut r:&[u32]=&S;unsafe{let s=core::ptr::read(&raw const r);core::ptr::write(&raw mut r,s);r[2]}};fn main(){let _=X;}
