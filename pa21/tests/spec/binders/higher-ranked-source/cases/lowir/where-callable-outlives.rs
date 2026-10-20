fn need<T:'static>(){}fn f<T>()where for<'a>fn(&'a T):'static{need::<T>();}fn main(){f::<u8>();}
