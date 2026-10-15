fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{}impl<T:Copy>A for T{}fn id<T:A>(x:T)->T{x}fn main(){check(id(7u8)==7);check(id(9.0f32)==9.0);}