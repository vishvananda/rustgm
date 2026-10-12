trait A{const N:u8;} impl A for u8{const N:u8=1;} fn f(x:u8){match x{<u8 as A>::N=>(),_=>()}}
