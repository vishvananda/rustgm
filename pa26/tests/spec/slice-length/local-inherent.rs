fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S;impl S{fn len(&self)->usize{7}}fn main(){check(S.len()==7);check([1u8,2,3].len()==3);}
