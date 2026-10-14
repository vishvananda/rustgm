#![recursion_limit="64"]fn f<T>(){f::<(T,)>();}fn main(){f::<u8>();}
