fn check(v:bool){if !v{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
struct Wrap<T>(T);impl<T>Wrap<T>{fn two<A,B,F:FnMut(A,B)->T>(mut f:F)->impl FnMut(A,B)->Wrap<T>{move|a,b|Wrap(f(a,b))}}fn drive<R,F:FnMut(u8,u8)->R>(mut f:F)->R{f(1,2)}fn project<T,F:FnMut(u8,u8)->T>(f:F)->T{drive(Wrap::two(f)).0}
fn main(){check(project(|a,b|a+b)==3);}
