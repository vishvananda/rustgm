macro_rules! m{($name:ident)=>{type $name<T=$name<u8>>=T;}}m!(A);fn f(x:A)->u8{x}
fn main(){}
