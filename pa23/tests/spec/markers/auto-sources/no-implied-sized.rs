fn sized<T>(){}fn f<T:?Sized+Send>(){sized::<T>();}
fn main(){}
