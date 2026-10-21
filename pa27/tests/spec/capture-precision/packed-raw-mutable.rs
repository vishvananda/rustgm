#[repr(packed)]struct P{a:u8,b:u32}fn main(){let mut p=P{a:1,b:7};let mut c=||&raw mut p.b;let _=c();}
