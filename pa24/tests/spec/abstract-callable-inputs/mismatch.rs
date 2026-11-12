trait It{type Item;}
impl It for u8{type Item=u16;}
fn by<T,F:FnMut(&T)->u16>(x:T,mut f:F)->u16{f(&x)}
fn get(x:&u16)->u16{*x}
fn pick<I:It>(x:I::Item)->u16{by(x,get)}
fn main(){}
