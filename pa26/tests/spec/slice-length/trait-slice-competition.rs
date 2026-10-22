fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait L{fn len(&self)->usize;}impl L for [u8]{fn len(&self)->usize{7}}fn main(){let a:&[u8]=&[1,2,3];check(a.len()==3);check(L::len(a)==7);}
