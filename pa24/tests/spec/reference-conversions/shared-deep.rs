fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}fn read(x:&u8)->u8{*x}fn main(){let x=7u8;let r=&x;let rr=&r;let f=||read(rr);check(f()==7);}
