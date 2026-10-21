#[repr(C,packed)]struct P{prefix:u8,x:u32}const X:u32={let mut x=P{prefix:0,x:7};let p=&raw mut x.x;unsafe{let r=&mut *p;*r}};fn main(){let _=X;}
