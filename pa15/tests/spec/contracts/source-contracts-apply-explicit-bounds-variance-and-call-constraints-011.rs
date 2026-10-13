struct S<'a>(fn(&'a u8));fn f<'a:'b,'b>(x:S<'b>)->S<'a>{x}
fn main() {}
