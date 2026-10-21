const fn borrow<'a>(r:&'a u32)->&'a u32{let p=r as *const u32;unsafe{&*p}}const X:u32={let x=37;*borrow(&x)};fn main(){let _=X;}
