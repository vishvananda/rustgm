fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C,packed)]struct S{tag:u8,s:&'static [u8]}static V:S=S{tag:3,s:&[7,9]};fn main(){let n=match V.s{[a,..,b]=>*a+*b,_=>0};check(n==16 && V.tag==3);}
