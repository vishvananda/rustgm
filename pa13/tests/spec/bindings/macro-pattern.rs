fn check(ok:bool){if !ok{let _=failure(0);}}fn failure(x:i32)->i32{1/x}
macro_rules! take{($p:pat,$e:expr)=>{let $p=$e;}}fn main(){take!((x,y),(1u8,2u16));check(x==1&&y==2);}