fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T:Copy>(T);impl<T:Copy>S<T>{fn get(&self)->T{self.0}}fn main(){check(S::get(&S(9u8))==9);}