fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
const C:&[u8]=b"abc";fn f(s:&[u8])->u8{match s{C=>1,b""=>2,b"a"=>3,b"\0\xff"=>4,_=>0}}fn main(){check(f(b"abc")==1 && f(b"")==2 && f(b"a")==3 && f(b"\0\xff")==4 && f(b"abd")==0);}
