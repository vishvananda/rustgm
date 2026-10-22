struct S<T>(T);impl<T>S<T>{const C:u8={let _:u16=false;7};}fn run<T>(n:u8)->bool{match n{S::<T>::C=>true,_=>false}}fn main(){}
