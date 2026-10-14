trait Has{type Item;}struct S;impl Has for S{type Item=u8;}fn main(){let _: <S as Has>::Item=7;}
