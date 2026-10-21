fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct S(u8);fn take(s:S)->u8{s.0}fn main(){let s=S(7);let f=move||{let g=move||take(s);g()};verify(f()==7);}
