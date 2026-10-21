fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
#[repr(packed)]struct P{a:u8,b:u32}fn main(){let p=P{a:1,b:7};let c=||&p.a;verify(*c()==1);}
