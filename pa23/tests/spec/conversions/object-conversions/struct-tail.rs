trait T{}impl T for u8{}struct S<X:?Sized>{prefix:u8,tail:X}fn f(x:&S<u8>)->&S<dyn T>{x}
fn main(){}
