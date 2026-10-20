trait A{fn f()->impl Copy;}impl A for u8{fn f()->impl Copy{7u8}}fn main(){let _:u8=<u8 as A>::f();}
