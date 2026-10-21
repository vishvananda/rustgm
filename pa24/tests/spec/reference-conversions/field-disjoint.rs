fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn read(x:&u8)->u8{*x}fn main(){let mut a=3u8;let mut b=7u8;let t=(&mut a,&mut b);let f=||read(t.0);*t.1=9;check(f()==3&&b==9);}
