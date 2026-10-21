#[repr(packed)]struct P{a:u8,b:u32}fn shared<F:Fn()->u32>(_:&F){}fn main(){let p=P{a:1,b:7};let c=move||p.b;shared(&c);let _=c();let _=c();}
