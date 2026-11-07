struct Hold<F>(F);
impl<F,A>Hold<F> where F:Fn(A){}
fn main(){}
