fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn read(x:&u8)->u8{*x}fn main(){let mut a=3u8;let mut b=7u8;let mut r=&mut a;let f=||read(r);r=&mut b;check(f()==3);}
