fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{}impl<'a>A for &'a u8{}struct W<T:A>(T);fn value<'a>(x:W<&'a u8>)->u8{*x.0}fn main(){let x=9u8;check(value(W(&x))==9);}