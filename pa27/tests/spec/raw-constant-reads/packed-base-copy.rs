#[repr(C,packed)]struct P{prefix:u8,x:u32}const X:u32={let x=P{prefix:0,x:37};let p=&raw const x;unsafe{(*p).x}};fn main(){let _=X;}
