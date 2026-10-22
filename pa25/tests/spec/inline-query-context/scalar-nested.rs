fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait Pick<'a>{const V:u8;}struct Tag;impl<'a>Pick<'a> for Tag{const V:u8=7;}struct S<'a,T>(&'a u8,T);impl<'a,T:Pick<'a>>S<'a,T>{const C:u8={const{const{let n=T::V;n}}};}fn get<'a>(_:&'a u8)->u8{S::<'static,Tag>::C}fn main(){let n=7;check(get(&n)==7);}
