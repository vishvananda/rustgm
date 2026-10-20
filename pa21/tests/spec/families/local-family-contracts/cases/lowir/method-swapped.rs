trait Q{type Out<T>;fn f()->(Self::Out<u8>,Self::Out<u16>);}impl Q for u8{type Out<T>=T;fn f()->(u16,u8){(1,2)}}
fn main(){}
