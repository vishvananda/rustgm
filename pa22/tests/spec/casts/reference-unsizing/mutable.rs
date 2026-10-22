fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn cast<'a>(p:&'a mut [u8;3])->&'a mut [u8]{p as &'a mut [u8]}fn main(){let mut a=[3,5,7];let s=cast(&mut a);s[1]=9;check(match s{[_,_,_]=>true,_=>false});check(a[1]==9);}
