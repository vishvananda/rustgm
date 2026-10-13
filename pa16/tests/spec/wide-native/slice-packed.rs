fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(C,packed)]struct S{x:u8,p:&'static [u8]}static A:S=S{x:3,p:&[7,9]};fn main(){let p=A.p;check(p[1]==9 && A.x==3);}
