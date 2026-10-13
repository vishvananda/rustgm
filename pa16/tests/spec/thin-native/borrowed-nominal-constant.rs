fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[derive(PartialEq)]struct S(u8);const C:S=S(7);fn f(x:&S)->u8{match x{&C=>1,_=>2}}fn main(){check(f(&S(7))==1 && f(&S(9))==2);}
