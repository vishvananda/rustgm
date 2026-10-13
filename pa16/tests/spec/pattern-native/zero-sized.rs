fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(Clone,Copy)]struct Z;fn f(s:&[Z])->u8{match s{[]=>0,[Z]=>1,[Z,..,Z]=>2}}fn main(){check(f(&[])==0 && f(&[Z])==1 && f(&[Z;4])==2);}
