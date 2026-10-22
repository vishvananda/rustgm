fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait L{fn len(&self)->usize;}impl L for [u8;3]{fn len(&self)->usize{7}}type A=[u8;3];fn main(){check(A::len(&[1,2,3])==7);}
