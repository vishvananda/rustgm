fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct S<T>(T);impl<T:Copy> S<T>{fn get(&self)->T{self.0}}fn main(){let _=S(7u8).get();}
