#[repr(C,packed)]union U{a:u32,b:u8}const X:u32={let mut u=U{a:7};let p=&raw mut u.a;unsafe{*p=37;}unsafe{u.a}};fn main(){let _=X;}
