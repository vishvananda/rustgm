fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn bump(x:&mut [u8]){x[1]+=1;}fn main(){let mut a=[3u8,7];let r=&mut a;let mut f=||bump(r);f();f();check(a[1]==9);}
