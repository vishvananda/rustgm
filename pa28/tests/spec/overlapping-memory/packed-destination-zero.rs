#[repr(C,packed)]struct P{a:u8,b:u32}const N:()={let a=7u32;let mut b=P{a:0,b:11};unsafe{core::ptr::copy(&raw const a,&raw mut b.b,0);}};fn main(){let _=N;}
