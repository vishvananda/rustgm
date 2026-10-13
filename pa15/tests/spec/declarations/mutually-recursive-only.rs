struct A<'a>{next:*const B<'a>}struct B<'b>{next:*const A<'b>}
fn main() {}
