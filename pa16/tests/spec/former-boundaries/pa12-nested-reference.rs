struct S([&'static u8;0]);fn f(s:S)->S{s}fn main(){let s=f(S([]));}
