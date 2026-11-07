trait Apply{type Out;}
struct Call<F>(F);
impl<F,A,U>Apply for Call<F> where F:FnOnce(A)->U{type Out=U;}
fn main(){}
