struct W;unsafe impl Sync for W where for<'a> &'a W:Sync{}fn need<T:Sync>(){}fn f(){need::<W>();}
fn main(){}
