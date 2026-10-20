fn need<T:'static>(){}fn f<T>()where for<'a>T:'a{need::<T>();}fn main(){f::<u8>();}
