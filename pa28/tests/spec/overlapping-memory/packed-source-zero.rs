#[repr(C,packed)]struct P{a:u8,b:u32}const N:()={let a=P{a:0,b:7};let mut b=11u32;unsafe{core::ptr::copy(&raw const a.b,&raw mut b,0);}};fn main(){let _=N;}
