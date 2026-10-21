fn main(){let mut x=7u32;let mut y=3u32;let mut p=&raw mut x;let other=&raw mut y;let mut c=||{unsafe{*p=9;}p=other;unsafe{*p}};check(c()==3);check(x==9);check(y==3);check(c()==9);check(y==9);}
