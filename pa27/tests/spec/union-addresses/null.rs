union U{a:u32,b:u8}const X:u32={let p=0usize as *const U;let q=unsafe{&raw const (*p).a};unsafe{*q}};fn main(){let _=X;}
