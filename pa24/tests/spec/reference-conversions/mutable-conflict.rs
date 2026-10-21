fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn bump(x:&mut u8){*x+=1;}fn main(){let mut x=7u8;let r=&mut x;let mut f=||bump(r);let q=&*r;f();check(*q==8);}
