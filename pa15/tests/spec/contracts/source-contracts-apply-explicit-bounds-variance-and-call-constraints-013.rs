struct S<'a>(*mut &'a u8);fn f<'a:'b,'b>(x:S<'a>)->S<'b>{x}
fn main() {}
