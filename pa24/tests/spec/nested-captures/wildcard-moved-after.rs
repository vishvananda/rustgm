fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct S(u8);fn main(){let x=S(7);let f=||{let g=||{let _=x;};g();};let moved=x;f();verify(moved.0==7);}
