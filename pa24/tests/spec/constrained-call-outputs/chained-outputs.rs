trait Apply{type Out;fn apply(self)->Self::Out;}
struct Pair<F,G>(F,G);
impl<F,G,U,V>Apply for Pair<F,G> where G:FnOnce(U)->V,F:FnOnce()->U{type Out=V;fn apply(self)->V{(self.1)((self.0)())}}
fn main(){}
