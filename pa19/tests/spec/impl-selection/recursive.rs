fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{}struct S<T>(T);impl A for u8{}impl<T:A>A for S<T>{}fn id<T:A>(x:T)->T{x}fn main(){let S(S(x))=id(S(S(9u8)));check(x==9);}