struct S(u8);const F:fn(u8)->S=S;impl S{const C:Self=F(7);}fn main(){}
