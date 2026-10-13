fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f(s:&[&str])->u8{match s{["a",..,"🦀"]=>7,[..,"b"]=>9,_=>0}}fn main(){check(f(&["a","héllo","🦀"])==7 && f(&["b"])==9 && f(&[])==0);}
