fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<const N:usize,const M:usize=N>([u8;N],[u8;M]);fn main(){let s:S<2>=S([1,2],[3,4]);check(s.0[1]+s.1[1]==6);}
