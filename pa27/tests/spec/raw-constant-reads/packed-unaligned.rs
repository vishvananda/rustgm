#[repr(C,packed)]struct P{x:u32}const X:u32={let x=P{x:37};let p=&raw const x.x;unsafe{*p}};fn main(){let _=X;}
