trait Apply{type Out;fn apply(self)->Self::Out;}
struct Call<F>(F,u8);
impl<'x,F,U>Apply for Call<F> where F:for<'a>FnOnce(&'a u8)->U{type Out=U;fn apply(self)->U{let v=self.1;(self.0)(&v)}}
fn main(){}
