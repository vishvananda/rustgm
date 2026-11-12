trait It{type Item;}
fn by<T,U,F:FnMut(&T,&U)->bool>(x:T,y:U,mut f:F)->bool{f(&x,&y)}
fn same<S>(_:&S,_:&S)->bool{true}
fn pick<I:It,J:It>(x:I::Item,y:J::Item)->bool{by(x,y,same)}
fn main(){}
