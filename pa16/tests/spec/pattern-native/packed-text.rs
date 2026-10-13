fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C,packed)]struct S{tag:u8,s:&'static str}static V:S=S{tag:3,s:"héllo"};fn main(){let n=match V.s{"héllo"=>7,_=>0};check(n==7 && V.tag==3);}
