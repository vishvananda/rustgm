trait Apply:for<'a>Fn(&'a u8)->u8{}impl<F:for<'a>Fn(&'a u8)->u8>Apply for F{}fn run(f:&dyn Apply)->u8{f(&0)}fn main(){let f=move|_:&u8|{let x=7u8;&x};let _=f(&0);}
