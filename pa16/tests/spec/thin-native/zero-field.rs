fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(align(64))]struct Z;struct S{n:u8,z:Z}fn test(_: &Z)->u8{7}fn main(){let s=S{n:3,z:Z};check(test(&s.z)==7 && s.n==3);}
