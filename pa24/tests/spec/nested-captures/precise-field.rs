fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct S(u8);fn read(s:&S)->u8{s.0}fn main(){let p=(S(7),S(9));let f=||{let g=||read(&p.0);g()};let other=p.1;verify(f()==7);verify(other.0==9);}
