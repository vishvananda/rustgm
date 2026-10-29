fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
const fn count()->usize{later()}const fn later()->usize{3}fn main(){let a=[7u8;count()];verify(a[2]==7);}