fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
fn f<const N:usize>()->[u8;N]{[7;N]}fn main(){let a=f::<3>();let b=f::<5>();check(a[2]==7&&b[4]==7);}
