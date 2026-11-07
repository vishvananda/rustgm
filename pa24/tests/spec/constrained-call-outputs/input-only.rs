trait Apply{type Out;}
struct Call<F>(F);
impl<F,A>Apply for Call<F> where F:FnOnce(A)->u8{type Out=u8;}
fn main(){}
