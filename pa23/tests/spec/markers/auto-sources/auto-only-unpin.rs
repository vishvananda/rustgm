fn f<T:?Sized>(p:*const T){let _: &dyn Unpin=&p;}
fn main(){}
