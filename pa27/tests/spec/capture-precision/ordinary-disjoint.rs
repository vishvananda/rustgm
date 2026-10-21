struct P{a:u8,b:u32}fn main(){let mut p=P{a:1,b:7};let c=||p.b;p.a=9;let _=c();}
