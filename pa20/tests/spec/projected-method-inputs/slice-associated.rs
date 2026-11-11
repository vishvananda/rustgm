trait Family{type Out;}
fn probe<R:Family>(w:&[R::Out])->usize{w.len()}
fn main(){}
