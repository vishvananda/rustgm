fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct P(u8);fn take(x:u8)->u8{x}fn main(){let x=7u8;let p:fn(u8)->u8=take;let f=||{let local=((1,2),3);let pair=local.0;let _=pair;P(p(x))};verify(f().0==7);verify(f().0==7);}
