fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn read(x:&[u8])->u8{x[1]}fn main(){let mut a=[3u8,7];let r=&mut a;let f=||read(r);check(f()==7&&f()==7);}
