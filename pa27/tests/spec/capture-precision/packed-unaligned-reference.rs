#[repr(packed)]struct P{a:u8,b:u32}fn main(){let p=P{a:1,b:7};let c=||&p.b;let _=*c();}
