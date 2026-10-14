fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn id<T,const N:usize>(x:[T;N])->[T;N]{x}fn main(){check(id::<u8,_>([1,7])[1]==7);}
