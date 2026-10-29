fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
fn make<const N:usize>()->[u8;N]{[7;N]}fn main(){let a=make::<3>();verify(a[2]==7);}