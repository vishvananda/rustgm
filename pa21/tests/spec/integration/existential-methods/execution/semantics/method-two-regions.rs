fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Make{fn make<'a,'b>(x:&'a u8,y:&'b u8)->impl Copy;}impl Make for u16{fn make<'x,'y>(x:&'x u8,y:&'y u8)->(&'y u8,&'x u8){(y,x)}}fn main(){let x=83;let y=89;let pair=<u16 as Make>::make(&x,&y);check(*pair.0==89);check(*pair.1==83);}
