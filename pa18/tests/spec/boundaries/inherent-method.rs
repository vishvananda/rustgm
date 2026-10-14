struct S<T>(T);impl<T:Copy> S<T>{fn get(&self)->T{self.0}}fn main(){let _=S(7u8).get();}
