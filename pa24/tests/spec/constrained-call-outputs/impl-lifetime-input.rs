trait Apply{type Out;}
struct Call<F>(F);
impl<'x,F,U>Apply for Call<F> where F:FnOnce(&'x u8)->U{type Out=U;}
fn main(){}
