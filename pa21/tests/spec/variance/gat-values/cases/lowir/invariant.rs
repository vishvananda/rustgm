trait F{type Out<'a>;}struct S;impl F for S{type Out<'a>=&'a mut &'a u8;}fn shorten<'a:'b,'b>(x:<S as F>::Out<'a>)-><S as F>::Out<'b>{x}fn main(){}
