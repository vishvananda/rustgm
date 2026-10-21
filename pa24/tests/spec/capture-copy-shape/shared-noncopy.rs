fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D{n:u8}fn main(){let d=D{n:7};let r=&d;let f=move||r;check(f().n==7);check(f().n==7);}
