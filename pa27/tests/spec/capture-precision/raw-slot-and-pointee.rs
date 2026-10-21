fn main(){let mut x=7u32;let mut y=3u32;let mut p=&raw mut x;let other=&raw mut y;let mut c=||{unsafe{*p=9;}p=other;unsafe{*p}};let _=c();}
