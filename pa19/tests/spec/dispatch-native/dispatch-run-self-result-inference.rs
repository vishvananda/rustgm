fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{fn f()->Self;}impl A for u8{fn f()->Self{7}}fn main(){let x:u8=A::f();check(x==7);}