fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
union U{a:u32,b:u8}fn main(){let u=U{a:7};let c=||unsafe{&u.a};verify(*c()==7);}
