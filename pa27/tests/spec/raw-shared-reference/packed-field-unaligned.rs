#[repr(C,packed)]struct P{prefix:u8,x:u32}const X:u32={let x=P{prefix:0,x:37};let p=&raw const x.x;unsafe{let r=&*p;*r}};fn main(){let _=X;}
