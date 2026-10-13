fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
#[repr(align(64))]struct Z;fn f<'a>(z:&'a Z)->&'a Z{z}fn test(_: &Z)->u8{7}fn main(){let z=Z;let r=&z;check(test(f(r))==7);}
