struct V(u8);trait A{const V:V=V(7);}impl A for u8{}fn main(){let _=<u8 as A>::V;}
