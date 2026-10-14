fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct D<T>(T);impl<T> Drop for D<T>{fn drop(&mut self){grow::<T>();}}fn grow<T>(){grow::<(T,T)>();}fn main(){check(true);}
