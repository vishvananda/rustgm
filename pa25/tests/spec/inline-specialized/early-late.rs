const fn f<const N:usize>()->usize{const{N+1}}type A=[u8;f::<2>()];const C:usize=f::<2>();pub fn run()->bool{let _:A=[0;3];C==3&&f::<2>()==3}
