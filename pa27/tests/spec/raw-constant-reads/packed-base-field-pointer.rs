#[repr(C,packed)]struct P{prefix:u8,x:u32}const X:u32={let x=P{prefix:0,x:37};let p=&raw const x;let q=unsafe{&raw const (*p).x};unsafe{*q}};fn main(){let _=X;}
