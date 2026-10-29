fn verify(ok:bool){if !ok{let _=fault(0);}}fn fault(x:u8)->u8{1/x}
struct S;impl S{const N:usize=3;}fn main(){let a=[7u8;S::N];verify(a[2]==7);}