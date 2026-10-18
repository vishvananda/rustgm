fn check(ok:bool){if !ok{let _=fail(0);}}fn fail(x:u8)->u8{1/x}
trait A{const C:usize=3;}impl A for u8{}fn f<const N:usize>()->usize{N}fn main(){check(f::<{<u8 as A>::C}>()==3);}