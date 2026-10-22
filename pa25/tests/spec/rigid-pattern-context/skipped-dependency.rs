trait V{const C:u8;}struct S<T>(T);impl<T:V>S<T>{const C:u8=if false{T::C}else{7};}fn run<T:V>(n:u8)->bool{match n{S::<T>::C=>true,_=>false}}fn main(){}
