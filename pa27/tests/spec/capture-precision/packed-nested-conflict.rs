#[repr(packed)]struct P{a:u8,b:u32}struct Outer{p:P,free:u32}fn main(){let mut o=Outer{p:P{a:1,b:7},free:2};let c=||o.p.b;o.p.a=9;let _=c();}
