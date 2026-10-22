struct S<T>(T);impl<T:Copy>S<T>{const C:u8={let n=7;n};}fn run<T>(n:u8)->bool{match n{S::<T>::C=>true,_=>false}}fn main(){}
