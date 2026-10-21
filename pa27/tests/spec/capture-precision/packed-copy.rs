#[repr(packed)]struct P{a:u8,b:u32}fn shared<F:Fn()->u32>(_:&F){}fn main(){let p=P{a:1,b:7};let c=||p.b;shared(&c);let _=p.a;let _=c();}
