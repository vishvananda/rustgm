fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait L{fn len(&self)->usize;}struct S;impl L for S{fn len(&self)->usize{7}}fn length<T:L>(a:&T)->usize{a.len()}fn main(){check(length(&S)==7);}
