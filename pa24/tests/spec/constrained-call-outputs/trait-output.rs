trait Apply{type Out;fn apply(self)->Self::Out;}
struct Call<F>(F,u8);
impl<F,U>Apply for Call<F> where F:FnOnce(u8)->U{type Out=U;fn apply(self)->U{(self.0)(self.1)}}
fn main(){}
