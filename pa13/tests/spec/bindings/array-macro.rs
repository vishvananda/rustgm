fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
macro_rules! take{($p:pat,$e:expr)=>{let $p=$e;}}fn main(){take!([a,tail@..],[2u8,3,5]);check(a==2&&tail[1]==5);}