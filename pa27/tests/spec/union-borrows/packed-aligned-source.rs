#[repr(C,packed)]union U{a:u32,b:u32}#[repr(C)]struct W{prefix:u64,u:U}const X:u32={let w=W{prefix:0,u:U{a:37}};unsafe{let r=&w.u.a;*r}};fn main(){let _=X;}
