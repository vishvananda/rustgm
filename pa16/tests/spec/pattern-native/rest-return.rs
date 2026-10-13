fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[u8])->&[u8]{match s{[_,mid @ ..,_]=>mid,_=>s}}fn main(){check(f(&[1,7,9,2])[1]==9 && f(&[7])[0]==7);let _=f(&[1,2]);}
