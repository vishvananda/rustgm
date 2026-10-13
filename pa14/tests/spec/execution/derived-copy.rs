fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
#[derive(Clone,Copy)]struct S(u8);fn f(x:S)->u8{let a=x;let b=x;let c=[x;2];a.0+b.0+c[0].0+c[1].0}fn main(){check(f(S(3))==12);}
