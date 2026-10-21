fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct S(u8);fn main(){let p=(S(7),S(9));let f=move||{let taken=p.0;let g=||{let _=p.0;};g();verify(taken.0==7);};f();}
