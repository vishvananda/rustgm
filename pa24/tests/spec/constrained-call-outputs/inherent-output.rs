struct Hold<F>(F);
impl<F,U>Hold<F> where F:Fn()->U{fn get(&self)->U{(self.0)()}}
fn main(){}
