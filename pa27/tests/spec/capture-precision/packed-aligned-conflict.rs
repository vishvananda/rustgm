#[repr(packed)]struct P{a:u8,b:u8}fn main(){let mut p=P{a:1,b:7};let c=||p.a;p.b=9;let _=c();}
