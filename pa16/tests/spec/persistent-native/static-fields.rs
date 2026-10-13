fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S{a:u8,b:[u16;3]}static A:S=S{a:3,b:[4,5,6]};static B:S=S{a:3,b:[4,5,6]};fn main(){check(A.a==3 && A.b[2]==6 && B.b[0]==4);}
