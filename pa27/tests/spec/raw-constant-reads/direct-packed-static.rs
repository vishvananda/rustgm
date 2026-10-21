#[repr(C,packed)]struct P{x:u32}static S:P=P{x:37};const X:u32=unsafe{*(&raw const S.x)};fn main(){let _=X;}
