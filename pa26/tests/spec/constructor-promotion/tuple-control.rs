struct Pair(u32,u32);fn f()->&'static Pair{&Pair(7,9)}fn main(){let _=f();}
